#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StateNonFungibleIdsResponse {
    #[serde(rename = "ledger_state")]
    pub ledger_state: Box<crate::models::LedgerState>,

    #[serde(rename = "resource_address")]
    pub resource_address: String,
    #[serde(rename = "non_fungible_ids")]
    pub non_fungible_ids: Box<crate::models::NonFungibleIdsCollection>,
}

impl StateNonFungibleIdsResponse {
    pub fn new(
        ledger_state: crate::models::LedgerState,
        resource_address: String,
        non_fungible_ids: crate::models::NonFungibleIdsCollection,
    ) -> StateNonFungibleIdsResponse {
        StateNonFungibleIdsResponse {
            ledger_state: Box::new(ledger_state),
            resource_address,
            non_fungible_ids: Box::new(non_fungible_ids),
        }
    }
}
