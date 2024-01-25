#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TransactionNotFoundError {
    #[serde(rename = "type")]
    pub r#type: String,

    #[serde(rename = "intent_hash")]
    pub intent_hash: String,
}

impl TransactionNotFoundError {
    pub fn new(
        r#type: String,
        intent_hash: String,
    ) -> TransactionNotFoundError {
        TransactionNotFoundError {
            r#type,
            intent_hash,
        }
    }
}
