#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MetadataPublicKeyHashArrayValue {
    #[serde(rename = "type")]
    pub r#type: crate::models::MetadataValueType,
    #[serde(rename = "values")]
    pub values: Vec<crate::models::PublicKeyHash>,
}

impl MetadataPublicKeyHashArrayValue {
    pub fn new(
        r#type: crate::models::MetadataValueType,
        values: Vec<crate::models::PublicKeyHash>,
    ) -> MetadataPublicKeyHashArrayValue {
        MetadataPublicKeyHashArrayValue { r#type, values }
    }
}
