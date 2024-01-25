#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NonFungibleResourcesCollectionItemGloballyAggregated {
    #[serde(rename = "aggregation_level")]
    pub aggregation_level: crate::models::ResourceAggregationLevel,

    #[serde(rename = "resource_address")]
    pub resource_address: String,
    #[serde(
        rename = "explicit_metadata",
        skip_serializing_if = "Option::is_none"
    )]
    pub explicit_metadata: Option<Box<crate::models::EntityMetadataCollection>>,

    #[serde(rename = "amount")]
    pub amount: i64,

    #[serde(rename = "last_updated_at_state_version")]
    pub last_updated_at_state_version: i64,
}

impl NonFungibleResourcesCollectionItemGloballyAggregated {
    pub fn new(
        aggregation_level: crate::models::ResourceAggregationLevel,
        resource_address: String,
        amount: i64,
        last_updated_at_state_version: i64,
    ) -> NonFungibleResourcesCollectionItemGloballyAggregated {
        NonFungibleResourcesCollectionItemGloballyAggregated {
            aggregation_level,
            resource_address,
            explicit_metadata: None,
            amount,
            last_updated_at_state_version,
        }
    }
}
