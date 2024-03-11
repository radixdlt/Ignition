#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StateEntityDetailsResponseItem {
    #[serde(rename = "address")]
    pub address: String,
    #[serde(
        rename = "fungible_resources",
        skip_serializing_if = "Option::is_none"
    )]
    pub fungible_resources:
        Option<Box<crate::models::FungibleResourcesCollection>>,
    #[serde(
        rename = "non_fungible_resources",
        skip_serializing_if = "Option::is_none"
    )]
    pub non_fungible_resources:
        Option<Box<crate::models::NonFungibleResourcesCollection>>,
    #[serde(
        rename = "ancestor_identities",
        skip_serializing_if = "Option::is_none"
    )]
    pub ancestor_identities: Option<
        Box<crate::models::StateEntityDetailsResponseItemAncestorIdentities>,
    >,
    #[serde(rename = "metadata")]
    pub metadata: Box<crate::models::EntityMetadataCollection>,
    #[serde(
        rename = "explicit_metadata",
        skip_serializing_if = "Option::is_none"
    )]
    pub explicit_metadata: Option<Box<crate::models::EntityMetadataCollection>>,
    #[serde(rename = "details", skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl StateEntityDetailsResponseItem {
    pub fn new(
        address: String,
        metadata: crate::models::EntityMetadataCollection,
    ) -> StateEntityDetailsResponseItem {
        StateEntityDetailsResponseItem {
            address,
            fungible_resources: None,
            non_fungible_resources: None,
            ancestor_identities: None,
            metadata: Box::new(metadata),
            explicit_metadata: None,
            details: None,
        }
    }
}
