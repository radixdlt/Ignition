#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TransactionConstructionResponse {
    #[serde(rename = "ledger_state")]
    pub ledger_state: Box<crate::models::LedgerState>,
}

impl TransactionConstructionResponse {
    pub fn new(
        ledger_state: crate::models::LedgerState,
    ) -> TransactionConstructionResponse {
        TransactionConstructionResponse {
            ledger_state: Box::new(ledger_state),
        }
    }
}
