#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StateKeyValueStoreDataResponseItem {
    #[serde(rename = "key")]
    pub key: Box<crate::models::ScryptoSborValue>,
    #[serde(rename = "value")]
    pub value: Box<crate::models::ScryptoSborValue>,

    #[serde(rename = "last_updated_at_state_version")]
    pub last_updated_at_state_version: i64,
    #[serde(rename = "is_locked")]
    pub is_locked: bool,
}

impl StateKeyValueStoreDataResponseItem {
    pub fn new(
        key: crate::models::ScryptoSborValue,
        value: crate::models::ScryptoSborValue,
        last_updated_at_state_version: i64,
        is_locked: bool,
    ) -> StateKeyValueStoreDataResponseItem {
        StateKeyValueStoreDataResponseItem {
            key: Box::new(key),
            value: Box::new(value),
            last_updated_at_state_version,
            is_locked,
        }
    }
}
