#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MetadataU64Value {
    #[serde(rename = "type")]
    pub r#type: crate::models::MetadataValueType,
    #[serde(rename = "value")]
    pub value: String,
}

impl MetadataU64Value {
    pub fn new(
        r#type: crate::models::MetadataValueType,
        value: String,
    ) -> MetadataU64Value {
        MetadataU64Value { r#type, value }
    }
}
