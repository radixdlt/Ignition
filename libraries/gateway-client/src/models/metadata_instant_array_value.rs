#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MetadataInstantArrayValue {
    #[serde(rename = "type")]
    pub r#type: crate::models::MetadataValueType,
    #[serde(rename = "values")]
    pub values: Vec<String>,
}

impl MetadataInstantArrayValue {
    pub fn new(
        r#type: crate::models::MetadataValueType,
        values: Vec<String>,
    ) -> MetadataInstantArrayValue {
        MetadataInstantArrayValue { r#type, values }
    }
}
