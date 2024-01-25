#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LedgerStateMixin {
    #[serde(rename = "ledger_state")]
    pub ledger_state: Box<crate::models::LedgerState>,
}

impl LedgerStateMixin {
    pub fn new(ledger_state: crate::models::LedgerState) -> LedgerStateMixin {
        LedgerStateMixin {
            ledger_state: Box::new(ledger_state),
        }
    }
}
