#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MetadataUrlArrayValue {
    #[serde(rename = "type")]
    pub r#type: crate::models::MetadataValueType,
    #[serde(rename = "values")]
    pub values: Vec<String>,
}

impl MetadataUrlArrayValue {
    pub fn new(
        r#type: crate::models::MetadataValueType,
        values: Vec<String>,
    ) -> MetadataUrlArrayValue {
        MetadataUrlArrayValue { r#type, values }
    }
}
