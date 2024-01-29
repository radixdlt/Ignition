#[serde_with::serde_as]
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TransactionSubmitRequest {
    #[serde(rename = "notarized_transaction_hex")]
    #[serde_as(as = "serde_with::hex::Hex")]
    pub notarized_transaction: Vec<u8>,
}

impl TransactionSubmitRequest {
    pub fn new(notarized_transaction: Vec<u8>) -> TransactionSubmitRequest {
        TransactionSubmitRequest {
            notarized_transaction,
        }
    }
}
