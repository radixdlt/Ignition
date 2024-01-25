#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StateNonFungibleDataResponse {
    #[serde(rename = "ledger_state")]
    pub ledger_state: Box<crate::models::LedgerState>,

    #[serde(rename = "resource_address")]
    pub resource_address: String,
    #[serde(rename = "non_fungible_id_type")]
    pub non_fungible_id_type: crate::models::NonFungibleIdType,
    #[serde(rename = "non_fungible_ids")]
    pub non_fungible_ids:
        Vec<crate::models::StateNonFungibleDetailsResponseItem>,
}

impl StateNonFungibleDataResponse {
    pub fn new(
        ledger_state: crate::models::LedgerState,
        resource_address: String,
        non_fungible_id_type: crate::models::NonFungibleIdType,
        non_fungible_ids: Vec<
            crate::models::StateNonFungibleDetailsResponseItem,
        >,
    ) -> StateNonFungibleDataResponse {
        StateNonFungibleDataResponse {
            ledger_state: Box::new(ledger_state),
            resource_address,
            non_fungible_id_type,
            non_fungible_ids,
        }
    }
}
