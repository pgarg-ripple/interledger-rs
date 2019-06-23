use super::{SettlementAccount, SettlementStore};
use bytes::Bytes;
use futures::{
    future::{ok, result, Either},
    Future,
};
use hyper::{Response, StatusCode};
use interledger_ildcp::IldcpAccount;
use interledger_packet::PrepareBuilder;
use interledger_service::{AccountStore, OutgoingRequest, OutgoingService};
use interledger_service_util::Convert;
use std::{
    marker::PhantomData,
    str::{self, FromStr},
    time::{Duration, SystemTime},
};

static PEER_PROTOCOL_CONDITION: [u8; 32] = [
    102, 104, 122, 173, 248, 98, 189, 119, 108, 143, 193, 139, 142, 159, 142, 32, 8, 151, 20, 133,
    110, 226, 51, 179, 144, 42, 89, 29, 13, 95, 41, 37,
];

pub struct SettlementApi<S, O, A> {
    outgoing_handler: O,
    store: S,
    account_type: PhantomData<A>,
}

// TODO add authentication
impl_web! {
    impl<S, O, A> SettlementApi<S, O, A>
    where
        S: SettlementStore<Account = A> + AccountStore<Account = A> + Clone + Send + Sync + 'static,
        O: OutgoingService<A> + Clone + Send + Sync + 'static,
        A: SettlementAccount + IldcpAccount + Send + Sync + 'static,
    {
        pub fn new(store: S, outgoing_handler: O) -> Self {
            SettlementApi {
                store,
                outgoing_handler,
                account_type: PhantomData,
            }
        }

        pub fn store(&self) -> S {
            self.store.clone()
        }

        // TODO: Can the Response<()> be converted to a Response<String>? It'd
        // be nice if we could include the full error message body (currently
        // it's just the header)
        // TODO: Verify that the idempotency_key is seen as a "Idempotency-Key"
        // header by impl_web!
        #[post("/accounts/:account_id/settlement")]
        fn receive_settlement(&self, account_id: String, body: u64, idempotency_key: String) -> impl Future<Item = Response<Bytes>, Error = Response<String>> {
            let amount = body;
            let store = self.store.clone();
            let mut store_clone = store.clone();
            let mut store_clone2 = store.clone();
            let idempotency_key_clone = idempotency_key.clone();

            // Check store for idempotency key. If exists, return cached data
            store_clone.load_idempotent_data(idempotency_key.clone())
            .map_err(move |err| {
                let err = format!("Couldn't connect to store {:?}", err);
                error!("{}", err);
                Response::builder().status(500).body(err).unwrap()
            }).and_then(move |data: Option<(StatusCode, Bytes)>| {
                if let Some(d) = data {
                    return Either::A(ok(Response::builder().status(d.0).body(d.1).unwrap()))
                }

            Either::B(
            result(A::AccountId::from_str(&account_id)
                .map_err(move |_err| {
                    let err = format!("Unable to parse account id: {}", account_id);
                    error!("{}", err);
                    Response::builder().status(400).body(err).unwrap()
                }))
                .and_then(move |account_id| store.get_accounts(vec![account_id]).map_err(move |_err| {
                    let err = format!("Error getting account: {}", account_id);
                    error!("{}", err);
                    Response::builder().status(404).body(err).unwrap()
                }))
                .and_then(|accounts| {
                    let account = &accounts[0];
                    if let Some(settlement_engine) = account.settlement_engine_details() {
                        Ok((account.clone(), settlement_engine))
                    } else {
                        let err = format!("Account {} does not have settlement engine details configured. Cannot handle incoming settlement", account.id());
                        error!("{}", err);
                        Err(Response::builder().status(404).body(err).unwrap())
                    }
                })
                .and_then(move |(account, settlement_engine)| {
                    let account_id = account.id();
                    let amount = amount.normalize_scale(account.asset_scale(), settlement_engine.asset_scale);

                    store_clone.update_balance_for_incoming_settlement(account_id, amount, idempotency_key_clone)
                        .map_err(move |_| {
                            let err = format!("Error updating balance of account: {} for incoming settlement of amount: {}", account_id, amount);
                            error!("{}", err);
                            Response::builder().status(500).body(err).unwrap()
                        })
                })
                .and_then(move |_| {
                    let ret = Bytes::from("Success");
                    store_clone2.save_idempotent_data(idempotency_key.clone(), StatusCode::OK, ret.clone())
                    .map_err(move |err| {
                        let err = format!("Couldn't connect to store {:?}", err);
                        error!("{}", err);
                        Response::builder().status(500).body(err).unwrap()
                    })
                    .and_then(|_| Ok(Response::builder().status(StatusCode::OK).body(ret).unwrap()))
                })
            )
            })
        }

        // Gets called by our settlement engine, forwards the request outwards
        // until it reaches the peer's settlement engine. Extract is not
        // implemented for Bytes unfortunately.
        #[post("/accounts/:account_id/messages")]
        fn send_outgoing_message(&self, account_id: String, body: Vec<u8>, idempotency_key: String)-> impl Future<Item = Response<Bytes>, Error = Response<String>> {
            let store = self.store.clone();
            let mut store_clone = self.store.clone(); // TODO: Can we avoid these clones?
            let mut outgoing_handler = self.outgoing_handler.clone();

            // Check store for idempotency key. If exists, return cached data
            store_clone.load_idempotent_data(idempotency_key.clone())
            .map_err(move |err| {
                let err = format!("Couldn't connect to store {:?}", err);
                error!("{}", err);
                Response::builder().status(500).body(err).unwrap()
            }).and_then(move |data: Option<(StatusCode, Bytes)>| {
                if let Some(d) = data {
                    return Either::A(ok(Response::builder().status(d.0).body(d.1).unwrap()))
                }

                Either::B(
                result(A::AccountId::from_str(&account_id)
                .map_err(move |_err| {
                    let err = format!("Unable to parse account id: {}", account_id);
                    error!("{}", err);
                    Response::builder().status(400).body(err).unwrap()
                }))
                .and_then(move |account_id| store.get_accounts(vec![account_id]).map_err(move |_| {
                    let err = format!("Error getting account: {}", account_id);
                    error!("{}", err);
                    Response::builder().status(404).body(err).unwrap()
                }))
                .and_then(|accounts| {
                    let account = &accounts[0];
                    if let Some(settlement_engine) = account.settlement_engine_details() {
                        Ok((account.clone(), settlement_engine))
                    } else {
                        let err = format!("Account {} has no settlement engine details configured, cannot send a settlement engine message to that account", accounts[0].id());
                        error!("{}", err);
                        Err(Response::builder().status(404).body(err).unwrap())
                    }
                })
                .and_then(move |(account, settlement_engine)| {
                    // Send the message to the peer's settlement engine.
                    // Note that we use dummy values for the `from` and `original_amount`
                    // because this `OutgoingRequest` will bypass the router and thus will not
                    // use either of these values. Including dummy values in the rare case where
                    // we do not need them seems easier than using
                    // `Option`s all over the place.
                    outgoing_handler.send_request(OutgoingRequest {
                        from: account.clone(),
                        to: account.clone(),
                        original_amount: 0,
                        prepare: PrepareBuilder {
                            destination: settlement_engine.ilp_address,
                            amount: 0,
                            expires_at: SystemTime::now() + Duration::from_secs(30),
                            data: &body,
                            execution_condition: &PEER_PROTOCOL_CONDITION,
                        }.build()
                    })
                    .map_err(|reject| {
                        let err = format!("Error sending message to peer settlement engine. Packet rejected with code: {}, message: {}", reject.code(), str::from_utf8(reject.message()).unwrap_or_default());
                        error!("{}", err);
                        Response::builder().status(502).body(err).unwrap()
                    })
                })
                .and_then(move |fulfill| {
                    let data = Bytes::from(fulfill.data());
                    store_clone.save_idempotent_data(idempotency_key, StatusCode::OK, data.clone())
                    .map_err(move |err| {
                        let err = format!("Couldn't connect to store {:?}", err);
                        error!("{}", err);
                        Response::builder().status(500).body(err).unwrap()
                    })
                    .and_then(|_| Ok(
                        Response::builder().status(200).body(data).unwrap()
                    ))
                })
            )})
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::*;
    use crate::test_helpers::*;

    // Settlement Tests

    mod settlement_tests {
        use super::*;

        #[test]
        fn settlement_ok() {
            let id = TEST_ACCOUNT_0.clone().id.to_string();
            let store = test_store(false, true);
            let api = test_api(store, false);

            let ret = api
                .receive_settlement(id, SETTLEMENT_BODY, IDEMPOTENCY.to_string())
                .wait();
            assert!(ret.is_ok());
        }

        #[test]
        fn account_has_no_engine_configured() {
            let id = TEST_ACCOUNT_0.clone().id.to_string();
            let store = test_store(false, false);
            let api = test_api(store, false);

            let ret = api
                .receive_settlement(id, SETTLEMENT_BODY, IDEMPOTENCY.to_string())
                .wait()
                .unwrap_err();
            assert_eq!(ret.status().as_u16(), 404);
        }

        #[test]
        fn update_balance_for_incoming_settlement_fails() {
            let id = TEST_ACCOUNT_0.clone().id.to_string();
            let store = test_store(true, true);
            let api = test_api(store, false);

            let ret: Response<_> = api
                .receive_settlement(id, SETTLEMENT_BODY, IDEMPOTENCY.to_string())
                .wait()
                .unwrap_err();
            assert_eq!(ret.status().as_u16(), 500);
        }

        #[test]
        fn invalid_account_id() {
            // the api is configured to take an accountId type
            // supplying an id that cannot be parsed to that type must fail
            let id = "a".to_string();
            let store = test_store(false, true);
            let api = test_api(store, false);

            let ret: Response<_> = api
                .receive_settlement(id, SETTLEMENT_BODY, IDEMPOTENCY.to_string())
                .wait()
                .unwrap_err();
            assert_eq!(ret.status().as_u16(), 400);
        }

        #[test]
        fn account_not_in_store() {
            let id = TEST_ACCOUNT_0.clone().id.to_string();
            let store = TestStore::new(vec![], false);
            let api = test_api(store, false);

            let ret: Response<_> = api
                .receive_settlement(id, SETTLEMENT_BODY, IDEMPOTENCY.to_string())
                .wait()
                .unwrap_err();
            assert_eq!(ret.status().as_u16(), 404);
        }
    }

    mod message_tests {
        use super::*;

        #[test]
        fn message_ok() {
            let id = TEST_ACCOUNT_0.clone().id.to_string();
            let store = test_store(false, true);
            let api = test_api(store, true);

            let ret: Response<_> = api
                .send_outgoing_message(id, vec![], IDEMPOTENCY.to_string())
                .wait()
                .unwrap();
            assert_eq!(ret.status(), StatusCode::OK);
            assert_eq!(ret.body(), &Bytes::from("hello!"));
        }

        // #[test] -- Disable until we figure out how to do with Arc<RwLock>>
        fn message_idempotent() {
            let id = TEST_ACCOUNT_0.clone().id.to_string();
            let store = test_store(false, true);
            let api = test_api(store, true);

            let ret: Response<_> = api
                .send_outgoing_message(id.clone(), vec![], IDEMPOTENCY.to_string())
                .wait()
                .unwrap();
            assert_eq!(ret.status(), StatusCode::OK);
            assert_eq!(ret.body(), &Bytes::from("hello!"));
            // This test fails because the store passed in the API is not the
            // same store that's used by the send_outgoing_message function,
            // since it clones inside. We'd have to make the API take a
            // reference to the store, such that it's able to mutate it
            // internally. Maybe an Arc<RwLock>?
            // Note that this wouldn't fail if the store is some external
            // process, and it only fails if the store is being run in memory,
            // such as in this case.
            // This call should hit the `load_idempotent_data` call, and
            // increase the test store's cache hits.
            let ret2 = api
                .send_outgoing_message(id, vec![], IDEMPOTENCY.to_string())
                .wait()
                .unwrap();
            assert_eq!(ret2.status(), StatusCode::OK);
            assert_eq!(ret2.body(), &Bytes::from("hello!"));

            assert_eq!(api.store().cache_hits, 1);
            let store = api.store();
            let cache = store.cache;
            let cached_data = cache.get(&IDEMPOTENCY.to_string()).unwrap();
            assert_eq!(cached_data.0, StatusCode::OK);
            assert_eq!(cached_data.1, &Bytes::from("hello!"));
        }

        #[test]
        fn message_gets_rejected() {
            let id = TEST_ACCOUNT_0.clone().id.to_string();
            let store = test_store(false, true);
            let api = test_api(store, false);

            let ret = api
                .send_outgoing_message(id, vec![], IDEMPOTENCY.to_string())
                .wait()
                .unwrap_err();
            assert_eq!(ret.status().as_u16(), 502);
        }

        #[test]
        fn invalid_account_id() {
            // the api is configured to take an accountId type
            // supplying an id that cannot be parsed to that type must fail
            let id = "a".to_string();
            let store = test_store(false, true);
            let api = test_api(store, true);

            let ret: Response<_> = api
                .send_outgoing_message(id, vec![], IDEMPOTENCY.to_string())
                .wait()
                .unwrap_err();
            assert_eq!(ret.status().as_u16(), 400);
        }

        #[test]
        fn account_not_in_store() {
            let id = TEST_ACCOUNT_0.clone().id.to_string();
            let store = TestStore::new(vec![], false);
            let api = test_api(store, true);

            let ret: Response<_> = api
                .send_outgoing_message(id, vec![], IDEMPOTENCY.to_string())
                .wait()
                .unwrap_err();
            assert_eq!(ret.status().as_u16(), 404);
        }

    }
}
