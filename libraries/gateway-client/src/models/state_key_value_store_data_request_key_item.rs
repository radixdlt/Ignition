#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StateKeyValueStoreDataRequestKeyItem {
    #[serde(rename = "key_hex", skip_serializing_if = "Option::is_none")]
    pub key_hex: Option<String>,
    #[serde(rename = "key_json", skip_serializing_if = "Option::is_none")]
    pub key_json: Option<Box<crate::models::ProgrammaticScryptoSborValue>>,
}

impl Default for StateKeyValueStoreDataRequestKeyItem {
    fn default() -> Self {
        Self::new()
    }
}

impl StateKeyValueStoreDataRequestKeyItem {
    pub fn new() -> StateKeyValueStoreDataRequestKeyItem {
        StateKeyValueStoreDataRequestKeyItem {
            key_hex: None,
            key_json: None,
        }
    }
}
