#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MetadataGlobalAddressValue {
    #[serde(rename = "type")]
    pub r#type: crate::models::MetadataValueType,
    #[serde(rename = "value")]
    pub value: String,
}

impl MetadataGlobalAddressValue {
    pub fn new(
        r#type: crate::models::MetadataValueType,
        value: String,
    ) -> MetadataGlobalAddressValue {
        MetadataGlobalAddressValue { r#type, value }
    }
}
