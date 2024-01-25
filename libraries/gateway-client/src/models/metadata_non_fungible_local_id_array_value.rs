#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MetadataNonFungibleLocalIdArrayValue {
    #[serde(rename = "type")]
    pub r#type: crate::models::MetadataValueType,
    #[serde(rename = "values")]
    pub values: Vec<String>,
}

impl MetadataNonFungibleLocalIdArrayValue {
    pub fn new(
        r#type: crate::models::MetadataValueType,
        values: Vec<String>,
    ) -> MetadataNonFungibleLocalIdArrayValue {
        MetadataNonFungibleLocalIdArrayValue { r#type, values }
    }
}
