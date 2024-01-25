#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MetadataDecimalValue {
    #[serde(rename = "type")]
    pub r#type: crate::models::MetadataValueType,
    #[serde(rename = "value")]
    pub value: String,
}

impl MetadataDecimalValue {
    pub fn new(
        r#type: crate::models::MetadataValueType,
        value: String,
    ) -> MetadataDecimalValue {
        MetadataDecimalValue { r#type, value }
    }
}
