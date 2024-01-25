#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MetadataU32Value {
    #[serde(rename = "type")]
    pub r#type: crate::models::MetadataValueType,
    #[serde(rename = "value")]
    pub value: String,
}

impl MetadataU32Value {
    pub fn new(
        r#type: crate::models::MetadataValueType,
        value: String,
    ) -> MetadataU32Value {
        MetadataU32Value { r#type, value }
    }
}
