#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct InvalidRequestError {
    #[serde(rename = "type")]
    pub r#type: String,

    #[serde(rename = "validation_errors")]
    pub validation_errors: Vec<crate::models::ValidationErrorsAtPath>,
}

impl InvalidRequestError {
    pub fn new(
        r#type: String,
        validation_errors: Vec<crate::models::ValidationErrorsAtPath>,
    ) -> InvalidRequestError {
        InvalidRequestError {
            r#type,
            validation_errors,
        }
    }
}
