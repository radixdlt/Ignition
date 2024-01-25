#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TransactionSubmitResponse {
    #[serde(rename = "duplicate")]
    pub duplicate: bool,
}

impl TransactionSubmitResponse {
    pub fn new(duplicate: bool) -> TransactionSubmitResponse {
        TransactionSubmitResponse { duplicate }
    }
}
