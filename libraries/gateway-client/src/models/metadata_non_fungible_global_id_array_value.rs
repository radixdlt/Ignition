#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MetadataNonFungibleGlobalIdArrayValue {
    #[serde(rename = "type")]
    pub r#type: crate::models::MetadataValueType,
    #[serde(rename = "values")]
    pub values:
        Vec<crate::models::MetadataNonFungibleGlobalIdArrayValueAllOfValues>,
}

impl MetadataNonFungibleGlobalIdArrayValue {
    pub fn new(
        r#type: crate::models::MetadataValueType,
        values: Vec<
            crate::models::MetadataNonFungibleGlobalIdArrayValueAllOfValues,
        >,
    ) -> MetadataNonFungibleGlobalIdArrayValue {
        MetadataNonFungibleGlobalIdArrayValue { r#type, values }
    }
}
