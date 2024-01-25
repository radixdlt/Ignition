#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StateEntityDetailsResponseFungibleVaultDetails {
    #[serde(rename = "type")]
    pub r#type: crate::models::StateEntityDetailsResponseItemDetailsType,

    #[serde(rename = "resource_address")]
    pub resource_address: String,
    #[serde(rename = "balance")]
    pub balance: Box<
        crate::models::FungibleResourcesCollectionItemVaultAggregatedVaultItem,
    >,
}

impl StateEntityDetailsResponseFungibleVaultDetails {
    pub fn new(
        r#type: crate::models::StateEntityDetailsResponseItemDetailsType,
        resource_address: String,
        balance: crate::models::FungibleResourcesCollectionItemVaultAggregatedVaultItem,
    ) -> StateEntityDetailsResponseFungibleVaultDetails {
        StateEntityDetailsResponseFungibleVaultDetails {
            r#type,
            resource_address,
            balance: Box::new(balance),
        }
    }
}
