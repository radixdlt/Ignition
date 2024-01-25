#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TransactionPreviewResponseLogsInner {
    #[serde(rename = "level")]
    pub level: String,
    #[serde(rename = "message")]
    pub message: String,
}

impl TransactionPreviewResponseLogsInner {
    pub fn new(
        level: String,
        message: String,
    ) -> TransactionPreviewResponseLogsInner {
        TransactionPreviewResponseLogsInner { level, message }
    }
}
