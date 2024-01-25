#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MetadataU8ArrayValue {
    #[serde(rename = "type")]
    pub r#type: crate::models::MetadataValueType,
    #[serde(rename = "value_hex")]
    pub value_hex: String,
}

impl MetadataU8ArrayValue {
    pub fn new(
        r#type: crate::models::MetadataValueType,
        value_hex: String,
    ) -> MetadataU8ArrayValue {
        MetadataU8ArrayValue { r#type, value_hex }
    }
}
