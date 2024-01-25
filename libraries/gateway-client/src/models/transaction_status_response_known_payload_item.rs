#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TransactionStatusResponseKnownPayloadItem {
    #[serde(rename = "payload_hash")]
    pub payload_hash: String,
    #[serde(rename = "status")]
    pub status: crate::models::TransactionStatus,
    #[serde(
        rename = "payload_status",
        skip_serializing_if = "Option::is_none"
    )]
    pub payload_status: Option<crate::models::TransactionPayloadStatus>,

    #[serde(
        rename = "payload_status_description",
        skip_serializing_if = "Option::is_none"
    )]
    pub payload_status_description: Option<String>,

    #[serde(
        rename = "error_message",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub error_message: Option<Option<String>>,

    #[serde(
        rename = "latest_error_message",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub latest_error_message: Option<Option<String>>,
    #[serde(
        rename = "handling_status",
        skip_serializing_if = "Option::is_none"
    )]
    pub handling_status:
        Option<crate::models::TransactionPayloadGatewayHandlingStatus>,

    #[serde(
        rename = "handling_status_reason",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub handling_status_reason: Option<Option<String>>,

    #[serde(
        rename = "submission_error",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub submission_error: Option<Option<String>>,
}

impl TransactionStatusResponseKnownPayloadItem {
    pub fn new(
        payload_hash: String,
        status: crate::models::TransactionStatus,
    ) -> TransactionStatusResponseKnownPayloadItem {
        TransactionStatusResponseKnownPayloadItem {
            payload_hash,
            status,
            payload_status: None,
            payload_status_description: None,
            error_message: None,
            latest_error_message: None,
            handling_status: None,
            handling_status_reason: None,
            submission_error: None,
        }
    }
}
