#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct InvalidTransactionError {
    #[serde(rename = "type")]
    pub r#type: String,
}

impl InvalidTransactionError {
    pub fn new(r#type: String) -> InvalidTransactionError {
        InvalidTransactionError { r#type }
    }
}
