#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NotSyncedUpError {
    #[serde(rename = "type")]
    pub r#type: String,

    #[serde(rename = "request_type")]
    pub request_type: String,

    #[serde(rename = "current_sync_delay_seconds")]
    pub current_sync_delay_seconds: i64,

    #[serde(rename = "max_allowed_sync_delay_seconds")]
    pub max_allowed_sync_delay_seconds: i64,
}

impl NotSyncedUpError {
    pub fn new(
        r#type: String,
        request_type: String,
        current_sync_delay_seconds: i64,
        max_allowed_sync_delay_seconds: i64,
    ) -> NotSyncedUpError {
        NotSyncedUpError {
            r#type,
            request_type,
            current_sync_delay_seconds,
            max_allowed_sync_delay_seconds,
        }
    }
}
