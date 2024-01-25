#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct InvalidEntityError {
    #[serde(rename = "type")]
    pub r#type: String,

    #[serde(rename = "address")]
    pub address: String,
}

impl InvalidEntityError {
    pub fn new(r#type: String, address: String) -> InvalidEntityError {
        InvalidEntityError { r#type, address }
    }
}
