#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ValidatorsUptimeResponse {
    #[serde(rename = "ledger_state")]
    pub ledger_state: Box<crate::models::LedgerState>,
    #[serde(rename = "validators")]
    pub validators: Box<crate::models::ValidatorUptimeCollection>,
}

impl ValidatorsUptimeResponse {
    pub fn new(
        ledger_state: crate::models::LedgerState,
        validators: crate::models::ValidatorUptimeCollection,
    ) -> ValidatorsUptimeResponse {
        ValidatorsUptimeResponse {
            ledger_state: Box::new(ledger_state),
            validators: Box::new(validators),
        }
    }
}
