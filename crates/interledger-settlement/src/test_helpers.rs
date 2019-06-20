use super::*;
use crate::SettlementEngineDetails;
use futures::{
    future::{err, ok},
    Future,
};
use interledger_ildcp::IldcpAccount;
use interledger_service::{
    incoming_service_fn, outgoing_service_fn, Account, AccountStore, IncomingService,
    OutgoingService,
};

use interledger_packet::{Address, ErrorCode, FulfillBuilder, RejectBuilder};
use mockito::mock;

use crate::fixtures::{BODY, MESSAGES_API, SERVICE_ADDRESS, SETTLEMENT_API, TEST_ACCOUNT_0};
use std::str::FromStr;
use std::sync::Arc;
use tokio::runtime::Runtime;
use url::Url;

// Test account that implements settlement + ildcp info

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

impl IldcpAccount for TestAccount {
    fn asset_code(&self) -> &str {
        "XYZ"
    }

    fn asset_scale(&self) -> u8 {
        9
    }

    fn client_address(&self) -> &Address {
        &self.ilp_address
    }
}

// Test Store
#[derive(Clone)]
pub struct TestStore {
    pub accounts: Arc<Vec<TestAccount>>,
    pub should_fail: bool,
}

impl SettlementStore for TestStore {
    type Account = TestAccount;

    fn update_balance_for_incoming_settlement(
        &self,
        _account_id: <Self::Account as Account>::AccountId,
        _amount: u64,
    ) -> Box<Future<Item = (), Error = ()> + Send> {
        // Do we need to do anything here?
        // Maybe add some cache for the idempotency flag later
        let ret = if self.should_fail { err(()) } else { ok(()) };
        Box::new(ret)
    }
}

impl AccountStore for TestStore {
    type Account = TestAccount;

    fn get_accounts(
        &self,
        account_ids: Vec<<<Self as AccountStore>::Account as Account>::AccountId>,
    ) -> Box<Future<Item = Vec<Self::Account>, Error = ()> + Send> {
        let accounts: Vec<TestAccount> = self
            .accounts
            .iter()
            .filter_map(|account| {
                if account_ids.contains(&account.id) {
                    Some(account.clone())
                } else {
                    None
                }
            })
            .collect();
        if accounts.len() == account_ids.len() {
            Box::new(ok(accounts))
        } else {
            Box::new(err(()))
        }
    }
}

// Test Service

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

#[allow(dead_code)]
pub fn mock_settlement(status_code: usize) -> mockito::Mock {
    mock("POST", SETTLEMENT_API.clone())
        .match_header("content-type", "application/octet-stream")
        .with_status(status_code)
        .with_body(BODY)
}

pub fn mock_message(status_code: usize) -> mockito::Mock {
    mock("POST", MESSAGES_API.clone())
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
    let mut runtime = Runtime::new().unwrap();
    runtime.block_on(f)
}

pub fn test_service(
) -> SettlementMessageService<impl IncomingService<TestAccount> + Clone, TestAccount> {
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

pub fn test_store(store_fails: bool, account_has_engine: bool) -> TestStore {
    let mut acc = TEST_ACCOUNT_0.clone();
    acc.no_details = !account_has_engine;

    TestStore {
        accounts: Arc::new(vec![acc]),
        should_fail: store_fails,
    }
}

pub fn test_api(
    test_store: TestStore,
    should_fulfill: bool,
) -> SettlementApi<TestStore, impl OutgoingService<TestAccount> + Clone + Send + Sync, TestAccount>
{
    let outgoing = outgoing_service_fn(move |_| Box::new(
        if should_fulfill {
            ok(FulfillBuilder {
                fulfillment: &[0; 32],
                data: b"hello!",
            }
            .build())
        } else {
            err(RejectBuilder {
                code: ErrorCode::F02_UNREACHABLE,
                message: b"No other outgoing handler!",
                data: &[],
                triggered_by: Some(&SERVICE_ADDRESS),
            }
            .build())
        }
    ));
    SettlementApi::new(test_store, outgoing)
}