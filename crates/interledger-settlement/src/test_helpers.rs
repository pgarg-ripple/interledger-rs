use super::*;
use crate::SettlementEngineDetails;
use futures::{future::err, Future};
use interledger_service::{incoming_service_fn, Account, BoxedIlpFuture, IncomingService};
use parking_lot::Mutex;

use interledger_packet::{Address, ErrorCode, RejectBuilder};
use mockito::mock;
use mockito::Matcher;

use std::str::FromStr;
use tokio::runtime::Runtime;
use url::Url;

pub static DATA: &str = "DATA_FOR_SETTLEMENT_ENGINE";
pub static BODY: &str = "hi";

lazy_static! {
    pub static ref TEST_MUTEX: Mutex<()> = Mutex::new(());
    pub static ref TEST_ACCOUNT_0: TestAccount =
        TestAccount::new(0, "http://localhost:1234", "peer.settle.xrp-ledger");
    pub static ref SERVICE_ADDRESS: Address = Address::from_str("example.connector").unwrap();
    pub static ref SETTLEMENT_API: Matcher =
        Matcher::Regex(r"^/accounts/\d*/messages$".to_string());
}

#[derive(Debug, Clone)]
pub struct TestAccount {
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

pub fn mock_settle(status_code: usize) -> mockito::Mock {
    mock("POST", SETTLEMENT_API.clone())
        .match_header("content-type", "application/octet-stream")
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

pub fn test_service() -> SettlementMessageService<
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
