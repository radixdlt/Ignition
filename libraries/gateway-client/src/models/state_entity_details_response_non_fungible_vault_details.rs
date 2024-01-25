#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StateEntityDetailsResponseNonFungibleVaultDetails {
    #[serde(rename = "type")]
    pub r#type: crate::models::StateEntityDetailsResponseItemDetailsType,

    #[serde(rename = "resource_address")]
    pub resource_address: String,
    #[serde(rename = "balance")]
    pub balance: Box<crate::models::NonFungibleResourcesCollectionItemVaultAggregatedVaultItem>,
}

impl StateEntityDetailsResponseNonFungibleVaultDetails {
    pub fn new(
        r#type: crate::models::StateEntityDetailsResponseItemDetailsType,
        resource_address: String,
        balance: crate::models::NonFungibleResourcesCollectionItemVaultAggregatedVaultItem,
    ) -> StateEntityDetailsResponseNonFungibleVaultDetails {
        StateEntityDetailsResponseNonFungibleVaultDetails {
            r#type,
            resource_address,
            balance: Box::new(balance),
        }
    }
}
