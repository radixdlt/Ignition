#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MetadataI32Value {
    #[serde(rename = "type")]
    pub r#type: crate::models::MetadataValueType,
    #[serde(rename = "value")]
    pub value: String,
}

impl MetadataI32Value {
    pub fn new(
        r#type: crate::models::MetadataValueType,
        value: String,
    ) -> MetadataI32Value {
        MetadataI32Value { r#type, value }
    }
}
