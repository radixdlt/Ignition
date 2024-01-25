#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MetadataPublicKeyHashValue {
    #[serde(rename = "type")]
    pub r#type: crate::models::MetadataValueType,
    #[serde(rename = "value")]
    pub value: Box<crate::models::PublicKeyHash>,
}

impl MetadataPublicKeyHashValue {
    pub fn new(
        r#type: crate::models::MetadataValueType,
        value: crate::models::PublicKeyHash,
    ) -> MetadataPublicKeyHashValue {
        MetadataPublicKeyHashValue {
            r#type,
            value: Box::new(value),
        }
    }
}
