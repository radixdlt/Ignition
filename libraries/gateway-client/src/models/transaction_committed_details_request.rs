#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TransactionCommittedDetailsRequest {
    #[serde(
        rename = "at_ledger_state",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub at_ledger_state:
        Option<Option<Box<crate::models::LedgerStateSelector>>>,

    #[serde(rename = "intent_hash")]
    pub intent_hash: String,
    #[serde(rename = "opt_ins", skip_serializing_if = "Option::is_none")]
    pub opt_ins: Option<Box<crate::models::TransactionDetailsOptIns>>,
}

impl TransactionCommittedDetailsRequest {
    pub fn new(intent_hash: String) -> TransactionCommittedDetailsRequest {
        TransactionCommittedDetailsRequest {
            at_ledger_state: None,
            intent_hash,
            opt_ins: None,
        }
    }
}
