#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TransactionStatusResponse {
    #[serde(rename = "ledger_state")]
    pub ledger_state: Box<crate::models::LedgerState>,
    #[serde(rename = "status")]
    pub status: crate::models::TransactionStatus,
    #[serde(rename = "intent_status")]
    pub intent_status: crate::models::TransactionIntentStatus,

    #[serde(rename = "intent_status_description")]
    pub intent_status_description: String,
    #[serde(rename = "known_payloads")]
    pub known_payloads:
        Vec<crate::models::TransactionStatusResponseKnownPayloadItem>,

    #[serde(
        rename = "committed_state_version",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub committed_state_version: Option<Option<i64>>,

    #[serde(
        rename = "error_message",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub error_message: Option<Option<String>>,
}

impl TransactionStatusResponse {
    pub fn new(
        ledger_state: crate::models::LedgerState,
        status: crate::models::TransactionStatus,
        intent_status: crate::models::TransactionIntentStatus,
        intent_status_description: String,
        known_payloads: Vec<
            crate::models::TransactionStatusResponseKnownPayloadItem,
        >,
    ) -> TransactionStatusResponse {
        TransactionStatusResponse {
            ledger_state: Box::new(ledger_state),
            status,
            intent_status,
            intent_status_description,
            known_payloads,
            committed_state_version: None,
            error_message: None,
        }
    }
}
