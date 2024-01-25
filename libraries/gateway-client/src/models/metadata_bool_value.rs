#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MetadataBoolValue {
    #[serde(rename = "type")]
    pub r#type: crate::models::MetadataValueType,
    #[serde(rename = "value")]
    pub value: bool,
}

impl MetadataBoolValue {
    pub fn new(
        r#type: crate::models::MetadataValueType,
        value: bool,
    ) -> MetadataBoolValue {
        MetadataBoolValue { r#type, value }
    }
}
