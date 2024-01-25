#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StateEntityDetailsResponse {
    #[serde(rename = "ledger_state")]
    pub ledger_state: Box<crate::models::LedgerState>,
    #[serde(rename = "items")]
    pub items: Vec<crate::models::StateEntityDetailsResponseItem>,
}

impl StateEntityDetailsResponse {
    pub fn new(
        ledger_state: crate::models::LedgerState,
        items: Vec<crate::models::StateEntityDetailsResponseItem>,
    ) -> StateEntityDetailsResponse {
        StateEntityDetailsResponse {
            ledger_state: Box::new(ledger_state),
            items,
        }
    }
}
