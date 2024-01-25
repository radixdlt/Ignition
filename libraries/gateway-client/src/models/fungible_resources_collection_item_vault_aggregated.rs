#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FungibleResourcesCollectionItemVaultAggregated {
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
    pub vaults:
        Box<crate::models::FungibleResourcesCollectionItemVaultAggregatedVault>,
}

impl FungibleResourcesCollectionItemVaultAggregated {
    pub fn new(
        aggregation_level: crate::models::ResourceAggregationLevel,
        resource_address: String,
        vaults: crate::models::FungibleResourcesCollectionItemVaultAggregatedVault,
    ) -> FungibleResourcesCollectionItemVaultAggregated {
        FungibleResourcesCollectionItemVaultAggregated {
            aggregation_level,
            resource_address,
            explicit_metadata: None,
            vaults: Box::new(vaults),
        }
    }
}
