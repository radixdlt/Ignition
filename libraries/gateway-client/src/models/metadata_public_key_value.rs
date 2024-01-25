#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MetadataPublicKeyValue {
    #[serde(rename = "type")]
    pub r#type: crate::models::MetadataValueType,
    #[serde(rename = "value")]
    pub value: Box<crate::models::PublicKey>,
}

impl MetadataPublicKeyValue {
    pub fn new(
        r#type: crate::models::MetadataValueType,
        value: crate::models::PublicKey,
    ) -> MetadataPublicKeyValue {
        MetadataPublicKeyValue {
            r#type,
            value: Box::new(value),
        }
    }
}
