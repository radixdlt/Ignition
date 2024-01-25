#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MetadataNonFungibleLocalIdValue {
    #[serde(rename = "type")]
    pub r#type: crate::models::MetadataValueType,
    #[serde(rename = "value")]
    pub value: String,
}

impl MetadataNonFungibleLocalIdValue {
    pub fn new(
        r#type: crate::models::MetadataValueType,
        value: String,
    ) -> MetadataNonFungibleLocalIdValue {
        MetadataNonFungibleLocalIdValue { r#type, value }
    }
}
