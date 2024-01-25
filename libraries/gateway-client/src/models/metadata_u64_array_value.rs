#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MetadataU64ArrayValue {
    #[serde(rename = "type")]
    pub r#type: crate::models::MetadataValueType,
    #[serde(rename = "values")]
    pub values: Vec<String>,
}

impl MetadataU64ArrayValue {
    pub fn new(
        r#type: crate::models::MetadataValueType,
        values: Vec<String>,
    ) -> MetadataU64ArrayValue {
        MetadataU64ArrayValue { r#type, values }
    }
}
