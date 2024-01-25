#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TransactionSubmitRequest {
    #[serde(rename = "notarized_transaction_hex")]
    pub notarized_transaction_hex: String,
}

impl TransactionSubmitRequest {
    pub fn new(notarized_transaction_hex: String) -> TransactionSubmitRequest {
        TransactionSubmitRequest {
            notarized_transaction_hex,
        }
    }
}
