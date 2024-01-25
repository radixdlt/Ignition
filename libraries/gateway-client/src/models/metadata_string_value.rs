#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MetadataStringValue {
    #[serde(rename = "type")]
    pub r#type: crate::models::MetadataValueType,
    #[serde(rename = "value")]
    pub value: String,
}

impl MetadataStringValue {
    pub fn new(
        r#type: crate::models::MetadataValueType,
        value: String,
    ) -> MetadataStringValue {
        MetadataStringValue { r#type, value }
    }
}
