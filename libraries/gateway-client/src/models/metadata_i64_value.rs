#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MetadataI64Value {
    #[serde(rename = "type")]
    pub r#type: crate::models::MetadataValueType,
    #[serde(rename = "value")]
    pub value: String,
}

impl MetadataI64Value {
    pub fn new(
        r#type: crate::models::MetadataValueType,
        value: String,
    ) -> MetadataI64Value {
        MetadataI64Value { r#type, value }
    }
}
