#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StateNonFungibleLocationResponse {
    #[serde(rename = "ledger_state")]
    pub ledger_state: Box<crate::models::LedgerState>,

    #[serde(rename = "resource_address")]
    pub resource_address: String,
    #[serde(rename = "non_fungible_ids")]
    pub non_fungible_ids:
        Vec<crate::models::StateNonFungibleLocationResponseItem>,
}

impl StateNonFungibleLocationResponse {
    pub fn new(
        ledger_state: crate::models::LedgerState,
        resource_address: String,
        non_fungible_ids: Vec<
            crate::models::StateNonFungibleLocationResponseItem,
        >,
    ) -> StateNonFungibleLocationResponse {
        StateNonFungibleLocationResponse {
            ledger_state: Box::new(ledger_state),
            resource_address,
            non_fungible_ids,
        }
    }
}
