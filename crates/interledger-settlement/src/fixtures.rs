use interledger_packet::Address;
use mockito::Matcher;
use parking_lot::Mutex;
use std::str::FromStr;

use crate::api::SettlementDetails;
use crate::test_helpers::TestAccount;

pub static DATA: &str = "DATA_FOR_SETTLEMENT_ENGINE";
pub static BODY: &str = "hi";
pub static SETTLEMENT_BODY: SettlementDetails = SettlementDetails {
    amount: 100,
    scale: 9,
};

lazy_static! {
    pub static ref TEST_MUTEX: Mutex<()> = Mutex::new(());
    pub static ref TEST_ACCOUNT_0: TestAccount =
        TestAccount::new(0, "http://localhost:1234", "peer.settle.xrp-ledger");
    pub static ref SERVICE_ADDRESS: Address = Address::from_str("example.connector").unwrap();
    pub static ref MESSAGES_API: Matcher = Matcher::Regex(r"^/accounts/\d*/messages$".to_string());
    pub static ref SETTLEMENT_API: Matcher =
        Matcher::Regex(r"^/accounts/\d*/settlement$".to_string());
}
