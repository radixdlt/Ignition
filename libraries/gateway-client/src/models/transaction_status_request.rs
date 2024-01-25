#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TransactionStatusRequest {
    #[serde(rename = "intent_hash")]
    pub intent_hash: String,
}

impl TransactionStatusRequest {
    pub fn new(intent_hash: String) -> TransactionStatusRequest {
        TransactionStatusRequest { intent_hash }
    }
}
