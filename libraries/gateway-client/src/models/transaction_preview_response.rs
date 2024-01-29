#[serde_with::serde_as]
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TransactionPreviewResponse {
    #[serde(rename = "encoded_receipt")]
    #[serde_as(as = "serde_with::hex::Hex")]
    pub encoded_receipt: Vec<u8>,
    #[serde(rename = "receipt")]
    pub receipt: serde_json::Value,
    #[serde(rename = "resource_changes")]
    pub resource_changes: Vec<serde_json::Value>,
    #[serde(rename = "logs")]
    pub logs: Vec<crate::models::TransactionPreviewResponseLogsInner>,
}

impl TransactionPreviewResponse {
    pub fn new(
        encoded_receipt: Vec<u8>,
        receipt: serde_json::Value,
        resource_changes: Vec<serde_json::Value>,
        logs: Vec<crate::models::TransactionPreviewResponseLogsInner>,
    ) -> TransactionPreviewResponse {
        TransactionPreviewResponse {
            encoded_receipt,
            receipt,
            resource_changes,
            logs,
        }
    }
}
