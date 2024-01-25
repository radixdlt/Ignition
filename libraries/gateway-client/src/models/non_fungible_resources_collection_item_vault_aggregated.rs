#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NonFungibleResourcesCollectionItemVaultAggregated {
    #[serde(rename = "aggregation_level")]
    pub aggregation_level: crate::models::ResourceAggregationLevel,

    #[serde(rename = "resource_address")]
    pub resource_address: String,
    #[serde(
        rename = "explicit_metadata",
        skip_serializing_if = "Option::is_none"
    )]
    pub explicit_metadata: Option<Box<crate::models::EntityMetadataCollection>>,
    #[serde(rename = "vaults")]
    pub vaults: Box<
        crate::models::NonFungibleResourcesCollectionItemVaultAggregatedVault,
    >,
}

impl NonFungibleResourcesCollectionItemVaultAggregated {
    pub fn new(
        aggregation_level: crate::models::ResourceAggregationLevel,
        resource_address: String,
        vaults: crate::models::NonFungibleResourcesCollectionItemVaultAggregatedVault,
    ) -> NonFungibleResourcesCollectionItemVaultAggregated {
        NonFungibleResourcesCollectionItemVaultAggregated {
            aggregation_level,
            resource_address,
            explicit_metadata: None,
            vaults: Box::new(vaults),
        }
    }
}
