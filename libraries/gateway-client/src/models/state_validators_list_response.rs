#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StateValidatorsListResponse {
    #[serde(rename = "ledger_state")]
    pub ledger_state: Box<crate::models::LedgerState>,
    #[serde(rename = "validators")]
    pub validators: Box<crate::models::ValidatorCollection>,
}

impl StateValidatorsListResponse {
    pub fn new(
        ledger_state: crate::models::LedgerState,
        validators: crate::models::ValidatorCollection,
    ) -> StateValidatorsListResponse {
        StateValidatorsListResponse {
            ledger_state: Box::new(ledger_state),
            validators: Box::new(validators),
        }
    }
}
