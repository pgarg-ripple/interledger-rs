use super::SettlementAccount;
use futures::{
    future::{err, Either},
    Future, Stream,
};
use interledger_packet::{Address, ErrorCode, FulfillBuilder, RejectBuilder};
use interledger_service::{BoxedIlpFuture, IncomingRequest, IncomingService};
use parking_lot::Mutex;
use reqwest::r#async::Client;
use serde_json::{self, Value};
use std::marker::PhantomData;

const PEER_FULFILLMENT: [u8; 32] = [0; 32];

#[derive(Clone)]
pub struct SettlementMessageService<I, A> {
    ilp_address: Address,
    next: I,
    http_client: Client,
    account_type: PhantomData<A>,
}

impl<I, A> SettlementMessageService<I, A>
where
    I: IncomingService<A>,
    A: SettlementAccount,
{
    pub fn new(ilp_address: Address, next: I) -> Self {
        SettlementMessageService {
            next,
            ilp_address,
            http_client: Client::new(),
            account_type: PhantomData,
        }
    }
}

impl<I, A> IncomingService<A> for SettlementMessageService<I, A>
where
    I: IncomingService<A>,
    A: SettlementAccount,
{
    type Future = BoxedIlpFuture;

    fn handle_request(&mut self, request: IncomingRequest<A>) -> Self::Future {
        // Only handle the request if the destination address matches the ILP address
        // of the settlement engine being used for this account
        let ilp_address = self.ilp_address.clone();
        if let Some(settlement_engine_details) = request.from.settlement_engine_details() {
            if request.prepare.destination() == settlement_engine_details.ilp_address {
                let ilp_address_clone = self.ilp_address.clone();
                let engine_address = settlement_engine_details.ilp_address;
                let mut settlement_engine_url = settlement_engine_details.url;

                match serde_json::from_slice(request.prepare.data()) {
                    Ok(Value::Object(mut message)) => {
                        let id = request.from.id();
                        message.insert("accountId".to_string(), Value::String(id.to_string()));
                        // TODO add auth
                        settlement_engine_url
                            .path_segments_mut()
                            .expect("Invalid settlement engine URL")
                            .push("accounts")
                            .push(&id.to_string())
                            .push("messages"); // Maybe set the idempotency flag here in the headers
                        return Box::new(self.http_client.post(settlement_engine_url)
                        .json(&message)
                        .send()
                        .map_err(move |error| {
                            error!("Error sending message to settlement engine: {:?}", error);
                            RejectBuilder {
                                code: ErrorCode::T00_INTERNAL_ERROR,
                                message: b"Error sending message to settlement engine",
                                data: &[],
                                triggered_by: Some(&ilp_address_clone),
                            }.build()
                        })
                        .and_then(move |response| {
                            let status = response.status();
                            if status.is_success() {
                                Either::A(response.into_body().concat2().map_err(move |err| {
                                    // When can this case be reached?  Unclear when `concat2` fails
                                    error!("Error concatenating settlement engine response body: {:?}", err);
                                    RejectBuilder {
                                        code: ErrorCode::T00_INTERNAL_ERROR,
                                        message: b"Error getting settlement engine response",
                                        data: &[],
                                        triggered_by: Some(&engine_address),
                                    }.build()
                                })
                                .and_then(|body| {
                                    Ok(FulfillBuilder {
                                        fulfillment: &PEER_FULFILLMENT,
                                        data: body.as_ref(),
                                    }.build())
                                }))
                            } else {
                                error!("Settlement engine rejected message with HTTP error code: {}", response.status());
                                let code = if status.is_client_error() {
                                    ErrorCode::F00_BAD_REQUEST
                                } else {
                                    ErrorCode::T00_INTERNAL_ERROR
                                };
                                Either::B(err(RejectBuilder {
                                    code,
                                    message: format!("Settlement engine rejected request with error code: {}", response.status()).as_str().as_ref(),
                                    data: &[],
                                    triggered_by: Some(&engine_address),
                                }.build()))
                            }
                        }));
                    }
                    Err(error) => {
                        error!(
                            "Got invalid JSON message from account {}: {:?}",
                            request.from.id(),
                            error
                        );
                        return Box::new(err(RejectBuilder {
                            code: ErrorCode::F00_BAD_REQUEST,
                            message: format!("Unable to parse message as JSON: {:?}", error)
                                .as_str()
                                .as_ref(),
                            data: &[],
                            triggered_by: Some(&ilp_address),
                        }
                        .build()));
                    }
                    _ => {
                        // What type of request causes this fallthrough case?
                        error!("Got invalid settlement message from account {} that could not be parsed as a JSON object", request.from.id());
                        return Box::new(err(RejectBuilder {
                            code: ErrorCode::F00_BAD_REQUEST,
                            message: b"Unable to parse message as a JSON object",
                            data: &[],
                            triggered_by: Some(&ilp_address),
                        }
                        .build()));
                    }
                }
            } else {
                error!("Got settlement packet from account {} but there is no settlement engine url configured for it", request.from.id());
                return Box::new(err(RejectBuilder {
                    code: ErrorCode::F02_UNREACHABLE,
                    message: format!("Got settlement packet from account {} but there is no settlement engine url configured for it", request.from.id()).as_str().as_ref(),
                    data: &[],
                    triggered_by: Some(&ilp_address),
                }
                .build()));
            }
        }
        Box::new(self.next.handle_request(request))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    // use futures::future::ok;
    use interledger_packet::{Fulfill, PrepareBuilder, Reject};
    // use interledger_service::outgoing_service_fn;
    use crate::SettlementEngineDetails;
    use interledger_service::{incoming_service_fn, Account, BoxedIlpFuture, IncomingService};
    use std::time::SystemTime;

    use mockito::mock;
    use mockito::Matcher;

    use std::str::FromStr;
    use tokio::runtime::Runtime;
    use url::Url;

    use std::str;

    #[test]
    fn settlement_ok_valid_json() {
        // happy case
        let m = mock_settle(200).create();
        let mut settlement = test_service();
        let destination = TEST_ACCOUNT_0.clone().ilp_address;
        let fulfill: Fulfill = block_on(
            settlement.handle_request(IncomingRequest {
                from: TEST_ACCOUNT_0.clone(),
                prepare: PrepareBuilder {
                    amount: 0,
                    expires_at: SystemTime::now(),
                    destination,
                    data: VALID_JSON.as_bytes(),
                    execution_condition: &[0; 32],
                }
                .build(),
            }),
        )
        .unwrap();

        m.assert();
        assert_eq!(fulfill.data(), BODY.as_bytes());
        assert_eq!(fulfill.fulfillment(), &[0; 32]);
    }

    #[test]
    fn settlement_ok_invalid_json() {
        // the request is rejected so the api should never be hit
        let m = mock_settle(200).create().expect(0);
        let mut settlement = test_service();
        let destination = TEST_ACCOUNT_0.clone().ilp_address;
        let reject: Reject = block_on(
            settlement.handle_request(IncomingRequest {
                from: TEST_ACCOUNT_0.clone(),
                prepare: PrepareBuilder {
                    amount: 0,
                    expires_at: SystemTime::now(),
                    destination,
                    data: INVALID_JSON.as_bytes(),
                    execution_condition: &[0; 32],
                }
                .build(),
            }),
        )
        .unwrap_err();

        m.assert();
        assert_eq!(reject.code(), ErrorCode::F00_BAD_REQUEST);
        assert_eq!(reject.triggered_by(), SERVICE_ADDRESS.clone());
        assert_eq!(
            reject.message(),
            "Unable to parse message as JSON: Error(\"expected value\", line: 1, column: 1)"
                .as_bytes()
        );
    }

    #[test]
    fn no_settlement_engine_configured_for_destination() {
        // happy case
        let m = mock_settle(200).create().expect(0);
        let mut settlement = test_service();
        let destination = Address::from_str("example.some.address").unwrap();
        let reject: Reject = block_on(
            settlement.handle_request(IncomingRequest {
                from: TEST_ACCOUNT_0.clone(),
                prepare: PrepareBuilder {
                    amount: 0,
                    expires_at: SystemTime::now(),
                    destination,
                    data: VALID_JSON.as_bytes(),
                    execution_condition: &[0; 32],
                }
                .build(),
            }),
        )
        .unwrap_err();

        m.assert();
        assert_eq!(reject.code(), ErrorCode::F02_UNREACHABLE);
        assert_eq!(reject.triggered_by(), SERVICE_ADDRESS.clone());
        assert_eq!(
            reject.message(),
            "Got settlement packet from account 0 but there is no settlement engine url configured for it".as_bytes(),
        );
    }

    #[test]
    fn account_does_not_have_settlement_engine() {
        // happy case
        let m = mock_settle(200).create().expect(0);
        let mut settlement = test_service();
        let mut acc = TEST_ACCOUNT_0.clone();
        acc.no_details = true; // Hide the settlement engine data from the account
        let reject: Reject = block_on(
            settlement.handle_request(IncomingRequest {
                from: acc.clone(),
                prepare: PrepareBuilder {
                    amount: 0,
                    expires_at: SystemTime::now(),
                    destination: acc.ilp_address,
                    data: VALID_JSON.as_bytes(),
                    execution_condition: &[0; 32],
                }
                .build(),
            }),
        )
        .unwrap_err();

        m.assert();
        assert_eq!(reject.code(), ErrorCode::F02_UNREACHABLE);
        assert_eq!(reject.triggered_by(), SERVICE_ADDRESS.clone());
        assert_eq!(reject.message(), "No other incoming handler!".as_bytes(),);
    }

    #[test]
    fn settlement_engine_rejects() {
        // for whatever reason the engine rejects our request with a 500 code
        let error_code = 500;
        let error_str = "Internal Server Error";
        let m = mock_settle(error_code).create();
        let destination = TEST_ACCOUNT_0.clone().ilp_address;
        let mut settlement = test_service();
        let reject: Reject = block_on(
            settlement.handle_request(IncomingRequest {
                from: TEST_ACCOUNT_0.clone(),
                prepare: PrepareBuilder {
                    amount: 0,
                    expires_at: SystemTime::now(),
                    destination: destination.clone(),
                    data: VALID_JSON.as_bytes(),
                    execution_condition: &[0; 32],
                }
                .build(),
            }),
        )
        .unwrap_err();

        m.assert();
        assert_eq!(reject.code(), ErrorCode::T00_INTERNAL_ERROR);
        // The engine rejected the message, not the connector's service,
        // so the triggered by should be the ilp address of th engine - I think.
        assert_eq!(reject.triggered_by(), destination);
        assert_eq!(
            reject.message(),
            format!(
                "Settlement engine rejected request with error code: {} {}",
                error_code, error_str
            )
            .as_bytes(),
        );
    }

    ////////// HELPERS (should they go to their own file? //////////

    impl TestAccount {
        pub fn new(id: u64, url: &str, ilp_address: &str) -> Self {
            Self {
                id,
                url: Url::parse(url).unwrap(),
                ilp_address: Address::from_str(ilp_address).unwrap(),
                no_details: false,
            }
        }
    }

    // todo: replace with actually valid data
    static VALID_JSON: &str = r#"
        {
            "name": "John Doe",
            "age": 43,
            "phones": [
                "+44 1234567",
                "+44 2345678"
            ]
        }"#;

    static INVALID_JSON: &str = "asdf";
    static BODY: &str = "hi";

    lazy_static! {
        static ref TEST_MUTEX: Mutex<()> = Mutex::new(());
        static ref TEST_ACCOUNT_0: TestAccount =
            TestAccount::new(0, "http://localhost:1234", "peer.settle.xrp-ledger");
        static ref SERVICE_ADDRESS: Address = Address::from_str("example.connector").unwrap();
        static ref SETTLEMENT_API: Matcher =
            Matcher::Regex(r"^/accounts/\d*/messages$".to_string());
    }

    fn mock_settle(status_code: usize) -> mockito::Mock {
        mock("POST", SETTLEMENT_API.clone())
            .with_status(status_code)
            .with_body(BODY)
    }

    // Futures helper taken from the store_helpers in interledger-store-redis.
    pub fn block_on<F>(f: F) -> Result<F::Item, F::Error>
    where
        F: Future + Send + 'static,
        F::Item: Send,
        F::Error: Send,
    {
        // Only run one test at a time
        let _ = env_logger::try_init();
        let lock = TEST_MUTEX.lock();
        let mut runtime = Runtime::new().unwrap();
        let result = runtime.block_on(f);
        drop(lock);
        result
    }

    fn test_service() -> SettlementMessageService<
        impl IncomingService<TestAccount, Future = BoxedIlpFuture> + Clone,
        TestAccount,
    > {
        SettlementMessageService::new(
            SERVICE_ADDRESS.clone(),
            incoming_service_fn(|_request| {
                Box::new(err(RejectBuilder {
                    code: ErrorCode::F02_UNREACHABLE,
                    message: b"No other incoming handler!",
                    data: &[],
                    triggered_by: Some(&SERVICE_ADDRESS),
                }
                .build()))
            }),
        )
    }

    /// PROBABLY COULD USE SOME UNIVERSAL TEST HELPERS FOR SERVICES SINCE THE
    /// TESTING STYLE IS VERY SIMILAR?

    #[derive(Debug, Clone)]
    struct TestAccount {
        pub id: u64,
        pub url: Url,
        pub ilp_address: Address,
        pub no_details: bool,
    }

    impl Account for TestAccount {
        type AccountId = u64;

        fn id(&self) -> u64 {
            self.id
        }
    }
    impl SettlementAccount for TestAccount {
        fn settlement_engine_details(&self) -> Option<SettlementEngineDetails> {
            if self.no_details {
                return None;
            }
            Some(SettlementEngineDetails {
                url: self.url.clone(),
                ilp_address: self.ilp_address.clone(),
                asset_scale: 9,
            })
        }
    }

}
