#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MetadataU32ArrayValue {
    #[serde(rename = "type")]
    pub r#type: crate::models::MetadataValueType,
    #[serde(rename = "values")]
    pub values: Vec<String>,
}

impl MetadataU32ArrayValue {
    pub fn new(
        r#type: crate::models::MetadataValueType,
        values: Vec<String>,
    ) -> MetadataU32ArrayValue {
        MetadataU32ArrayValue { r#type, values }
    }
}
