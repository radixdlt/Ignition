#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TransactionCommittedDetailsResponse {
    #[serde(rename = "ledger_state")]
    pub ledger_state: Box<crate::models::LedgerState>,
    #[serde(rename = "transaction")]
    pub transaction: Box<crate::models::CommittedTransactionInfo>,
}

impl TransactionCommittedDetailsResponse {
    pub fn new(
        ledger_state: crate::models::LedgerState,
        transaction: crate::models::CommittedTransactionInfo,
    ) -> TransactionCommittedDetailsResponse {
        TransactionCommittedDetailsResponse {
            ledger_state: Box::new(ledger_state),
            transaction: Box::new(transaction),
        }
    }
}
