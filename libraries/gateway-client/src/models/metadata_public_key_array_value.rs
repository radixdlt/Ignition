#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MetadataPublicKeyArrayValue {
    #[serde(rename = "type")]
    pub r#type: crate::models::MetadataValueType,
    #[serde(rename = "values")]
    pub values: Vec<crate::models::PublicKey>,
}

impl MetadataPublicKeyArrayValue {
    pub fn new(
        r#type: crate::models::MetadataValueType,
        values: Vec<crate::models::PublicKey>,
    ) -> MetadataPublicKeyArrayValue {
        MetadataPublicKeyArrayValue { r#type, values }
    }
}
