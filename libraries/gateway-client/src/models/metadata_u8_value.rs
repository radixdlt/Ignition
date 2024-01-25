#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MetadataU8Value {
    #[serde(rename = "type")]
    pub r#type: crate::models::MetadataValueType,
    #[serde(rename = "value")]
    pub value: String,
}

impl MetadataU8Value {
    pub fn new(
        r#type: crate::models::MetadataValueType,
        value: String,
    ) -> MetadataU8Value {
        MetadataU8Value { r#type, value }
    }
}
