#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MetadataBoolArrayValue {
    #[serde(rename = "type")]
    pub r#type: crate::models::MetadataValueType,
    #[serde(rename = "values")]
    pub values: Vec<bool>,
}

impl MetadataBoolArrayValue {
    pub fn new(
        r#type: crate::models::MetadataValueType,
        values: Vec<bool>,
    ) -> MetadataBoolArrayValue {
        MetadataBoolArrayValue { r#type, values }
    }
}
