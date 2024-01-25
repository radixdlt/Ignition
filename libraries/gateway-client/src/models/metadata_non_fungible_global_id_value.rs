#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MetadataNonFungibleGlobalIdValue {
    #[serde(rename = "type")]
    pub r#type: crate::models::MetadataValueType,

    #[serde(rename = "resource_address")]
    pub resource_address: String,

    #[serde(rename = "non_fungible_id")]
    pub non_fungible_id: String,
}

impl MetadataNonFungibleGlobalIdValue {
    pub fn new(
        r#type: crate::models::MetadataValueType,
        resource_address: String,
        non_fungible_id: String,
    ) -> MetadataNonFungibleGlobalIdValue {
        MetadataNonFungibleGlobalIdValue {
            r#type,
            resource_address,
            non_fungible_id,
        }
    }
}
