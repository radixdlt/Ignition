#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StateKeyValueStoreDataRequest {
    #[serde(
        rename = "at_ledger_state",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub at_ledger_state:
        Option<Option<Box<crate::models::LedgerStateSelector>>>,

    #[serde(rename = "key_value_store_address")]
    pub key_value_store_address: String,

    #[serde(rename = "keys")]
    pub keys: Vec<crate::models::StateKeyValueStoreDataRequestKeyItem>,
}

impl StateKeyValueStoreDataRequest {
    pub fn new(
        key_value_store_address: String,
        keys: Vec<crate::models::StateKeyValueStoreDataRequestKeyItem>,
    ) -> StateKeyValueStoreDataRequest {
        StateKeyValueStoreDataRequest {
            at_ledger_state: None,
            key_value_store_address,
            keys,
        }
    }
}
