#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MetadataInstantValue {
    #[serde(rename = "type")]
    pub r#type: crate::models::MetadataValueType,
    #[serde(rename = "value")]
    pub value: String,
}

impl MetadataInstantValue {
    pub fn new(
        r#type: crate::models::MetadataValueType,
        value: String,
    ) -> MetadataInstantValue {
        MetadataInstantValue { r#type, value }
    }
}
