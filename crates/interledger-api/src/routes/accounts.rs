use crate::{AccountDetails, NodeStore, BEARER_TOKEN_START};
use futures::{
    future::{err, ok, result, Either},
    Future,
};
use hyper::Response;
use interledger_http::{Auth, HttpAccount, HttpStore};
use interledger_service::Account;
use interledger_service_util::BalanceStore;
use log::{debug, error, trace};
use reqwest::r#async::Client;
use serde::Serialize;
use serde_json::{json, Value};
use tokio_retry::{strategy::FixedInterval, Retry};
use tower_web::{impl_web, Response};
use url::Url;

#[derive(Serialize, Response, Debug)]
#[web(status = "200")]
struct AccountsResponse<A: Serialize> {
    accounts: Vec<A>,
}

#[derive(Response)]
#[web(status = "200")]
struct Success;

#[derive(Response, Debug)]
#[web(status = "200")]
struct BalanceResponse {
    balance: String,
}

#[derive(Clone)]
pub struct AccountsApi<T> {
    store: T,
    admin_api_token: String,
}

const MAX_RETRIES: usize = 10;

// Convenience function to clean up error handling and reduce unwrap quantity
trait ErrorStatus {
    fn error(code: u16) -> Self;
}

impl ErrorStatus for Response<()> {
    fn error(code: u16) -> Self {
        Response::builder().status(code).body(()).unwrap()
    }
}

impl_web! {
    impl<T, A> AccountsApi<T>
    where T: NodeStore<Account = A> + HttpStore<Account = A> + BalanceStore<Account = A>,
    A: Account + HttpAccount + Serialize + 'static,

    {
        pub fn new(admin_api_token: String, store: T) -> Self {
            AccountsApi {
                store,
                admin_api_token,
            }
        }

        fn is_admin(&self, authorization: &str) -> bool {
            authorization[BEARER_TOKEN_START..] == self.admin_api_token
        }

        fn validate_admin(&self, authorization: String) -> impl Future<Item = T, Error = Response<()>> {
            if self.is_admin(&authorization) {
                ok(self.store.clone())
            } else {
                error!("Admin API endpoint called with non-admin API key");
                err(Response::error(401))
            }
        }

        #[post("/accounts")]
        #[content_type("application/json")]
        fn post_accounts(&self, body: AccountDetails, authorization: String) -> impl Future<Item = Value, Error = Response<()>> {
            // TODO don't allow accounts to be overwritten
            // TODO try connecting to that account's websocket server if it has
            // a btp_uri
            let se_url = body.settlement_engine_url.clone();
            self.validate_admin(authorization)
                .and_then(move |store| store.insert_account(body)
                .map_err(|_| Response::error(500))
                .and_then(|account| {
                    // if the account had a SE associated with it, then register
                    // the account in the SE.
                    if let Some(se_url)  = se_url {
                        let id = account.id();
                        Either::A(result(Url::parse(&se_url))
                        .map_err(|_| Response::error(500))
                        .and_then(move |mut se_url| {
                            se_url
                                .path_segments_mut()
                                .expect("Invalid settlement engine URL")
                                .push("accounts");
                            trace!(
                                "Sending account {} creation request to settlement engine: {:?}",
                                id,
                                se_url.clone()
                            );
                            let action = move || {
                                Client::new().post(se_url.clone())
                                .json(&json!({"id" : id.to_string()}))
                                .send()
                                .map_err(move |err| {
                                    error!("Error sending account creation command to the settlement engine: {:?}", err)
                                })
                                .and_then(move |response| {
                                    if response.status().is_success() {
                                        trace!("Account {} created on the SE", id);
                                        Ok(())
                                    } else {
                                        error!("Error creating account. Settlement engine responded with HTTP code: {}", response.status());
                                        Err(())
                                    }
                                })
                            };
                            Retry::spawn(FixedInterval::from_millis(2000).take(MAX_RETRIES), action)
                            .map_err(|_| Response::error(500))
                            .and_then(move |_| {
                                Ok(json!(account))
                            })
                        }))
                    } else {
                        Either::B(ok(json!(account)))
                    }
                }))
        }

        #[get("/accounts")]
        #[content_type("application/json")]
        fn get_accounts(&self, authorization: String) -> impl Future<Item = Value, Error = Response<()>> {
            let store = self.store.clone();
            if self.is_admin(&authorization) {
                Either::A(store.get_all_accounts()
                    .map_err(|_| Response::error(500))
                    .and_then(|accounts| Ok(json!(accounts))))
            } else {
                // Only allow the user to see their own account
                Either::B(result(Auth::parse(&authorization))
                    .map_err(move |_| {
                        error!("No account found with auth: {}", authorization);
                        Response::error(401)
                    })
                    .and_then(move |auth| {
                        let username = auth.username().to_owned();
                        let token = auth.password().to_owned();
                        store.get_account_from_http_token(&username, &token).map_err(|_| Response::error(404))
                        .and_then(|account| Ok(json!(vec![account])))
                    })
                )
            }
        }

        #[get("/accounts/:username")]
        #[content_type("application/json")]
        fn get_account(&self, username: String, authorization: String) -> impl Future<Item = Value, Error = Response<()>> {
            let store = self.store.clone();
            let is_admin = self.is_admin(&authorization);
            let username_clone = username.clone();
            let auth_clone = authorization.clone();
            store.get_account_id_from_username(username.clone())
            .map_err(move |_| {
                error!("Error getting account id from username: {}", username_clone);
                Response::builder().status(500).body(()).unwrap()
            })
            .and_then(move |id| {
                if is_admin  {
                    Either::A(store.get_accounts(vec![id])
                    .map_err(move |_| {
                        debug!("Account not found: {:?}", id);
                        Response::error(404)
                    })
                    .and_then(|mut accounts| Ok(json!(accounts.pop().unwrap()))))
                } else {
                    Either::B(result(Auth::parse(&auth_clone))
                        .map_err(move |_| {
                            error!("No account found with auth: {}", auth_clone);
                            Response::error(401)
                        })
                        .and_then(move |auth| {
                            let username = auth.username().to_owned();
                            let token = auth.password().to_owned();
                            store.get_account_from_http_token(&username, &token)
                            .map_err(move |_| {
                                debug!("No account found with auth: {}", authorization);
                                Response::error(401)
                            })
                            .and_then(move |account| {
                                if account.id() == id {
                                    Ok(json!(account))
                                } else {
                                    Err(Response::error(401))
                                }
                            })
                        })
                    )
                }
            })
        }

        #[delete("/accounts/:username")]
        #[content_type("application/json")]
        fn delete_account(&self, username: String, authorization: String) -> impl Future<Item = Value, Error = Response<()>> {
            let store_clone = self.store.clone();
            let self_clone = self.clone();
            store_clone.get_account_id_from_username(username.clone())
            .map_err(move |_| {
                error!("Error getting account id from username: {}", username);
                Response::builder().status(500).body(()).unwrap()
            })
            .and_then(move |id| {
                self_clone.validate_admin(authorization)
                .and_then(move |store| Ok((store, id)))
                .and_then(move |(store, id)|
                    store.remove_account(id)
                        .map_err(move |_| Response::error(500))
                        .and_then(move |account| {
                            // TODO: deregister from SE if url is present
                            Ok(json!(account))
                        })
                )
            })
        }

        // TODO should this be combined into the account record?
        #[get("/accounts/:username/balance")]
        #[content_type("application/json")]
        fn get_balance(&self, username: String, authorization: String) -> impl Future<Item = BalanceResponse, Error = Response<()>> {
            let store = self.store.clone();
            let store_clone = self.store.clone();
            let is_admin = self.is_admin(&authorization);
            let username_clone = username.clone();
            let auth_clone = authorization.clone();
            store_clone.get_account_id_from_username(username.clone())
            .map_err(move |_| {
                error!("Error getting account id from username: {}", username_clone);
                Response::builder().status(500).body(()).unwrap()
            })
            .and_then(move |id| {
                if is_admin  {
                    Either::A(store.get_accounts(vec![id])
                        .map_err(move |_| {
                            debug!("Account not found: {}", id);
                            Response::error(404)
                        })
                        .and_then(|mut accounts| Ok(accounts.pop().unwrap())))
                } else {
                    Either::B(result(Auth::parse(&auth_clone))
                        .map_err(move |_| {
                            error!("No account found with auth: {}", auth_clone);
                            Response::error(401)
                        })
                        .and_then(move |auth| {
                            let token = auth.password().to_owned();

                            store.get_account_from_http_token(&username, &token)
                            .map_err(move |_| {
                                error!("No account found with auth: {}", authorization);
                                Response::error(401)
                            })
                            .and_then(move |account| {
                                if account.id() == id {
                                    Ok(account)
                                } else {
                                    Err(Response::error(401))
                                }
                            })
                        })
                    )
                }
            })
            .and_then(move |account| store_clone.get_balance(account)
            .map_err(|_| Response::error(500)))
            .and_then(|balance| Ok(BalanceResponse {
                balance: balance.to_string(),
            }))
        }
    }
}
