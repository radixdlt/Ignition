#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StateKeyValueStoreDataResponse {
    #[serde(rename = "ledger_state")]
    pub ledger_state: Box<crate::models::LedgerState>,

    #[serde(rename = "key_value_store_address")]
    pub key_value_store_address: String,
    #[serde(rename = "entries")]
    pub entries: Vec<crate::models::StateKeyValueStoreDataResponseItem>,
}

impl StateKeyValueStoreDataResponse {
    pub fn new(
        ledger_state: crate::models::LedgerState,
        key_value_store_address: String,
        entries: Vec<crate::models::StateKeyValueStoreDataResponseItem>,
    ) -> StateKeyValueStoreDataResponse {
        StateKeyValueStoreDataResponse {
            ledger_state: Box::new(ledger_state),
            key_value_store_address,
            entries,
        }
    }
}
