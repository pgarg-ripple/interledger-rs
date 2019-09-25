use crate::{
    http_retry::Client, number_or_string, AccountDetails, AccountSettings, ApiError, NodeStore,
};
use bytes::Bytes;
use futures::{
    future::{err, join_all, ok, Either},
    Future, Stream,
};
use interledger_btp::{connect_to_service_account, BtpAccount, BtpOutgoingService};
use interledger_ccp::{CcpRoutingAccount, Mode, RouteControlRequest, RoutingRelation};
use interledger_http::{HttpAccount, HttpStore};
use interledger_ildcp::IldcpRequest;
use interledger_ildcp::IldcpResponse;
use interledger_router::RouterStore;
use interledger_service::{
    Account, AddressStore, AuthToken, IncomingService, OutgoingRequest, OutgoingService, Username,
};
use interledger_service_util::{BalanceStore, ExchangeRateStore};
use interledger_settlement::SettlementAccount;
use interledger_spsp::{pay, SpspResponder};
use interledger_stream::{PaymentNotification, StreamNotificationsStore};
use log::{debug, error, trace};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::convert::TryFrom;
use std::time::Duration;
use warp::{self, Filter};

const MAX_RETRIES: usize = 10;
const DEFAULT_HTTP_TIMEOUT: Duration = Duration::from_millis(5000);

#[derive(Deserialize, Debug)]
struct SpspPayRequest {
    receiver: String,
    #[serde(deserialize_with = "number_or_string")]
    source_amount: u64,
}

pub fn accounts_api<I, O, S, A, B>(
    server_secret: Bytes,
    admin_api_token: String,
    default_spsp_account: Option<Username>,
    incoming_handler: I,
    outgoing_handler: O,
    btp: BtpOutgoingService<B, A>,
    store: S,
) -> warp::filters::BoxedFilter<(impl warp::Reply,)>
where
    I: IncomingService<A> + Clone + Send + Sync + 'static,
    O: OutgoingService<A> + Clone + Send + Sync + 'static,
    B: OutgoingService<A> + Clone + Send + Sync + 'static,
    S: NodeStore<Account = A>
        + HttpStore<Account = A>
        + BalanceStore<Account = A>
        + StreamNotificationsStore<Account = A>
        + ExchangeRateStore
        + RouterStore,
    A: BtpAccount
        + CcpRoutingAccount
        + SettlementAccount
        + Account
        + HttpAccount
        + Serialize
        + Send
        + Sync
        + 'static,
{
    // TODO can we make any of the Filters const or put them in lazy_static?

    // Helper filters
    let admin_auth_header = format!("Bearer {}", admin_api_token);
    let admin_only = warp::header::<String>("authorization")
        .and_then(move |authorization| -> Result<(), warp::Rejection> {
            if authorization == admin_auth_header {
                Ok(())
            } else {
                Err(warp::reject::custom(ApiError::Unauthorized))
            }
        })
        // This call makes it so we do not pass on a () value on
        // success to the next filter, it just gets rid of it
        .untuple_one()
        .boxed();
    let with_store = warp::any().map(move || store.clone()).boxed();
    let with_incoming_handler = warp::any().map(move || incoming_handler.clone()).boxed();
    let accounts = warp::path("accounts");
    let accounts_index = accounts.and(warp::path::end());
    let account_username = accounts.and(warp::path::param2::<Username>());
    let account_username_to_id = account_username
        .and(with_store.clone())
        .and_then(|username: Username, store: S| {
            store
                .get_account_id_from_username(&username)
                .map_err(move |_| {
                    // TODO differentiate between server error and not found
                    error!("Error getting account id from username: {}", username);
                    warp::reject::custom(ApiError::AccountNotFound)
                })
        })
        .boxed();
    let valid_account_authorization = warp::header::<AuthToken>("authorization")
        .and(with_store.clone())
        .and_then(|auth: AuthToken, store: S| {
            store
                .get_account_from_http_auth(auth.username(), auth.password())
                .map_err(move |_| {
                    error!(
                        "Invalid authorization provided for user: {}",
                        auth.username()
                    );
                    warp::reject::custom(ApiError::Unauthorized)
                })
        })
        .boxed();
    let authorized_account_from_path = account_username
        .and(valid_account_authorization.clone())
        .and_then(
            |path_username: Username, authorized_account: A| -> Result<A, warp::Rejection> {
                // Check that the user is authorized for this route
                if &path_username == authorized_account.username() {
                    Ok(authorized_account)
                } else {
                    Err(warp::reject::custom(ApiError::Unauthorized))
                }
            },
        )
        .boxed();
    let admin_or_authorized_account = admin_only
        .clone()
        .and(account_username_to_id.clone())
        .clone()
        .or(authorized_account_from_path
            .clone()
            .map(|account: A| account.id()))
        .unify()
        .boxed();

    // POST /accounts
    let btp_clone = btp.clone();
    let outgoing_handler_clone = outgoing_handler.clone();
    let post_accounts = warp::post2()
        .and(accounts_index)
        .and(admin_only.clone())
        .and(warp::body::json())
        .and(with_store.clone())
        .and_then(move |account_details: AccountDetails, store: S| {
            let store_clone = store.clone();
            let handler = outgoing_handler_clone.clone();
            let btp = btp_clone.clone();
            store
                .insert_account(account_details)
                .map_err(|_| warp::reject::custom(ApiError::InternalServerError))
                .and_then(move |account| {
                    connect_to_external_services(handler, account, store_clone, btp)
                })
                .and_then(|account: A| Ok(warp::reply::json(&account)))
        })
        .boxed();

    // GET /accounts
    let get_accounts = warp::get2()
        .and(accounts_index)
        .and(admin_only.clone())
        .and(with_store.clone())
        .and_then(|store: S| {
            store
                .get_all_accounts()
                .map_err(|_| warp::reject::custom(ApiError::InternalServerError))
                .and_then(|accounts| Ok(warp::reply::json(&accounts)))
        })
        .boxed();

    // PUT /accounts/:username
    let put_account = warp::put2()
        .and(account_username_to_id.clone())
        .and(warp::path::end())
        .and(admin_only.clone())
        .and(warp::body::json())
        .and(with_store.clone())
        .and_then(
            move |id: A::AccountId, account_details: AccountDetails, store: S| {
                let store_clone = store.clone();
                let handler = outgoing_handler.clone();
                let btp = btp.clone();
                store
                    .update_account(id, account_details)
                    .map_err(move |_| warp::reject::custom(ApiError::InternalServerError))
                    .and_then(move |account| {
                        connect_to_external_services(handler, account, store_clone, btp)
                    })
                    .and_then(|account: A| Ok(warp::reply::json(&account)))
            },
        )
        .boxed();

    // GET /accounts/:username
    let get_account = warp::get2()
        .and(admin_or_authorized_account.clone())
        .and(warp::path::end())
        .and(with_store.clone())
        .and_then(|id: A::AccountId, store: S| {
            store
                .get_accounts(vec![id])
                .map_err(|_| warp::reject::not_found())
                .and_then(|accounts| Ok(warp::reply::json(&accounts[0])))
        })
        .boxed();

    // GET /accounts/:username/balance
    let get_account_balance = warp::get2()
        .and(admin_or_authorized_account.clone())
        .and(warp::path("balance"))
        .and(warp::path::end())
        .and(with_store.clone())
        .and_then(|id: A::AccountId, store: S| {
            // TODO reduce the number of store calls it takes to get the balance
            store
                .get_accounts(vec![id])
                .map_err(|_| warp::reject::not_found())
                .and_then(move |mut accounts| {
                    store
                        .get_balance(accounts.pop().unwrap())
                        .map_err(move |_| {
                            error!("Error getting balance for account: {}", id);
                            warp::reject::custom(ApiError::InternalServerError)
                        })
                })
                .and_then(|balance: i64| {
                    Ok(warp::reply::json(&json!({
                        "balance": balance.to_string(),
                    })))
                })
        })
        .boxed();

    // DELETE /accounts/:username
    let delete_account = warp::delete2()
        .and(admin_only.clone())
        .and(account_username_to_id.clone())
        .and(warp::path::end())
        .and(with_store.clone())
        .and_then(|id: A::AccountId, store: S| {
            store
                .delete_account(id)
                .map_err(move |_| {
                    error!("Error deleting account {}", id);
                    warp::reject::custom(ApiError::InternalServerError)
                })
                .and_then(|account| Ok(warp::reply::json(&account)))
        })
        .boxed();

    // PUT /accounts/:username/settings
    let put_account_settings = warp::put2()
        .and(admin_or_authorized_account.clone())
        .and(warp::path("settings"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with_store.clone())
        .and_then(|id: A::AccountId, settings: AccountSettings, store: S| {
            store
                .modify_account_settings(id, settings)
                .map_err(move |_| {
                    error!("Error updating account settings {}", id);
                    warp::reject::custom(ApiError::InternalServerError)
                })
                .and_then(|settings| Ok(warp::reply::json(&settings)))
        })
        .boxed();

    // (Websocket) /accounts/:username/payments/incoming
    let incoming_payment_notifications = warp::ws2()
        .and(admin_or_authorized_account.clone())
        .and(warp::path("payments"))
        .and(warp::path("incoming"))
        .and(warp::path::end())
        .and(with_store.clone())
        .map(|ws: warp::ws::Ws2, id: A::AccountId, store: S| {
            ws.on_upgrade(move |ws: warp::ws::WebSocket| {
                let (tx, rx) = futures::sync::mpsc::unbounded::<PaymentNotification>();
                store.add_payment_notification_subscription(id, tx);
                rx.map_err(|_| -> warp::Error { unreachable!("unbounded rx never errors") })
                    .map(|notification| {
                        warp::ws::Message::text(serde_json::to_string(&notification).unwrap())
                    })
                    .forward(ws)
                    .map(|_| ())
                    .map_err(|err| error!("Error forwarding notifications to websocket: {:?}", err))
            })
        })
        .boxed();

    // POST /accounts/:username/payments
    let post_payments = warp::post2()
        .and(authorized_account_from_path.clone())
        .and(warp::path("payments"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(with_incoming_handler.clone())
        .and_then(
            |account: A, pay_request: SpspPayRequest, incoming_handler: I| {
                pay(
                    incoming_handler,
                    account,
                    &pay_request.receiver,
                    pay_request.source_amount,
                )
                .and_then(|delivered_amount| {
                    debug!(
                        "Sent SPSP payment and delivered: {} of the receiver's units",
                        delivered_amount
                    );
                    Ok(warp::reply::json(&json!({
                        "delivered_amount": delivered_amount
                    })))
                })
                .map_err(|err| {
                    error!("Error sending SPSP payment: {:?}", err);
                    // TODO give a different error message depending on what type of error it is
                    warp::reject::custom(ApiError::InternalServerError)
                })
            },
        )
        .boxed();

    // GET /accounts/:username/spsp
    let server_secret_clone = server_secret.clone();
    let get_spsp = warp::get2()
        .and(account_username_to_id.clone())
        .and(warp::path("spsp"))
        .and(warp::path::end())
        .and(with_store.clone())
        .and_then(move |id: A::AccountId, store: S| {
            let server_secret_clone = server_secret_clone.clone();
            store
                .get_accounts(vec![id])
                .map_err(|_| warp::reject::custom(ApiError::InternalServerError))
                .and_then(move |accounts| {
                    // TODO return the response without instantiating an SpspResponder (use a simple fn)
                    Ok(SpspResponder::new(
                        accounts[0].ilp_address().clone(),
                        server_secret_clone.clone(),
                    )
                    .generate_http_response())
                })
        })
        .boxed();

    // GET /.well-known/pay
    // This is the endpoint a [Payment Pointer](https://github.com/interledger/rfcs/blob/master/0026-payment-pointers/0026-payment-pointers.md)
    // with no path resolves to
    let server_secret_clone = server_secret.clone();
    let get_spsp_well_known = warp::get2()
        .and(warp::path(".well-known"))
        .and(warp::path("pay"))
        .and(warp::path::end())
        .and(with_store.clone())
        .and_then(move |store: S| {
            // TODO don't clone this
            if let Some(username) = default_spsp_account.clone() {
                let server_secret_clone = server_secret_clone.clone();
                Either::A(
                    store
                        .get_account_id_from_username(&username)
                        .map_err(move |_| {
                            error!("Account not found: {}", username);
                            warp::reject::not_found()
                        })
                        .and_then(move |id| {
                            // TODO this shouldn't take multiple store calls
                            store
                                .get_accounts(vec![id])
                                .map_err(|_| warp::reject::custom(ApiError::InternalServerError))
                                .map(|mut accounts| accounts.pop().unwrap())
                        })
                        .and_then(move |account| {
                            // TODO return the response without instantiating an SpspResponder (use a simple fn)
                            Ok(SpspResponder::new(
                                account.ilp_address().clone(),
                                server_secret_clone.clone(),
                            )
                            .generate_http_response())
                        }),
                )
            } else {
                Either::B(err(warp::reject::not_found()))
            }
        })
        .boxed();

    get_spsp
        .or(get_spsp_well_known)
        .or(post_accounts)
        .or(get_accounts)
        .or(put_account)
        .or(get_account)
        .or(get_account_balance)
        .or(delete_account)
        .or(put_account_settings)
        .or(incoming_payment_notifications)
        .or(post_payments)
        .boxed()
}

fn get_address_from_parent_and_update_routes<S, A, T>(
    mut service: S,
    parent: A,
    store: T,
) -> impl Future<Item = (), Error = ()>
where
    S: OutgoingService<A> + Clone + Send + Sync + 'static,
    A: CcpRoutingAccount + Clone + Send + Sync + 'static,
    T: AddressStore + Clone + Send + Sync + 'static,
{
    let prepare = IldcpRequest {}.to_prepare();
    service
        .send_request(OutgoingRequest {
            from: parent.clone(), // Does not matter what we put here, they will get the account from the HTTP/BTP credentials
            to: parent.clone(),
            prepare,
            original_amount: 0,
        })
        .map_err(|err| error!("Error getting ILDCP info: {:?}", err))
        .and_then(|fulfill| {
            let response = IldcpResponse::try_from(fulfill.into_data().freeze()).map_err(|err| {
                error!(
                    "Unable to parse ILDCP response from fulfill packet: {:?}",
                    err
                );
            });
            debug!("Got ILDCP response: {:?}", response);
            let ilp_address = match response {
                Ok(info) => info.ilp_address(),
                Err(_) => return err(()),
            };
            ok(ilp_address)
        })
        .and_then(move |ilp_address| {
            // TODO we may want to make this trigger the CcpRouteManager to request
            let prepare = RouteControlRequest {
                mode: Mode::Sync,
                last_known_epoch: 0,
                last_known_routing_table_id: [0; 16],
                features: Vec::new(),
            }
            .to_prepare();
            debug!("Asking for routes from {:?}", parent.clone());
            join_all(vec![
                // Update our store's address
                store.set_ilp_address(ilp_address),
                // Get the parent's routes for us
                Box::new(
                    service
                        .send_request(OutgoingRequest {
                            from: parent.clone(),
                            to: parent.clone(),
                            original_amount: prepare.amount(),
                            prepare: prepare.clone(),
                        })
                        .and_then(move |_| Ok(()))
                        .map_err(move |err| {
                            error!("Got error when trying to update routes {:?}", err)
                        }),
                ),
            ])
        })
        .and_then(move |_| Ok(()))
}

// Helper function which gets called whenever a new account is added or
// modified.
// Performed actions:
// 1. If they have a BTP uri configured: connect to their BTP socket
// 2. If they are a parent:
// 2a. Perform an ILDCP Request to get the address assigned to us by them, and
// update our store's address to that value
// 2b. Perform a RouteControl Request to make them send us any new routes
// 3. If they have a settlement engine endpoitn configured: Make a POST to the
//    engine's account creation endpoint with the account's id
fn connect_to_external_services<S, A, T, B>(
    service: S,
    account: A,
    store: T,
    btp: BtpOutgoingService<B, A>,
) -> impl Future<Item = A, Error = warp::reject::Rejection>
where
    S: OutgoingService<A> + Clone + Send + Sync + 'static,
    A: CcpRoutingAccount + BtpAccount + SettlementAccount + Clone + Send + Sync + 'static,
    T: AddressStore + Clone + Send + Sync + 'static,
    B: OutgoingService<A> + Clone + 'static,
{
    // Try to connect to the account's BTP socket if they have
    // one configured
    let btp_connect_fut = if account.get_ilp_over_btp_url().is_some() {
        Either::A(
            connect_to_service_account(account.clone(), true, btp)
                .map_err(|_| warp::reject::custom(ApiError::InternalServerError)),
        )
    } else {
        Either::B(ok(()))
    };

    btp_connect_fut.and_then(move |_| {
    // If we added a parent, get the address assigned to us by
    // them and update all of our routes
    let get_ilp_address_fut = if account.routing_relation() == RoutingRelation::Parent {
        Either::A(
            get_address_from_parent_and_update_routes(service, account.clone(), store)
            .map_err(|_| {
                warp::reject::custom(ApiError::InternalServerError)
            })
        )
    } else {
        Either::B(ok(()))
    };

    // Register the account with the settlement engine
    // if a settlement_engine_url was configured on the account
    get_ilp_address_fut.and_then(move |_|
    if let Some(se_details) = account.settlement_engine_details() {
        let se_url = se_details.url;
        let id = account.id();
        let http_client = Client::new(DEFAULT_HTTP_TIMEOUT, MAX_RETRIES);
        trace!(
            "Sending account {} creation request to settlement engine: {:?}",
            id,
            se_url.clone()
        );
        Either::A(
            http_client.create_engine_account(se_url, id)
            .map_err(|_| {
                warp::reject::custom(ApiError::InternalServerError)
            })
            .and_then(move |status_code| {
                if status_code.is_success() {
                    trace!("Account {} created on the SE", id);
                } else {
                    error!("Error creating account. Settlement engine responded with HTTP code: {}", status_code);
                }
                Ok(())
            })
            .and_then(move |_| {
                Ok(account)
            }))
    } else {
        Either::B(ok(account))
    })})
}
