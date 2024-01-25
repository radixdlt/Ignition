#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct InternalServerError {
    #[serde(rename = "type")]
    pub r#type: String,

    #[serde(rename = "exception")]
    pub exception: String,

    #[serde(rename = "cause")]
    pub cause: String,
}

impl InternalServerError {
    pub fn new(
        r#type: String,
        exception: String,
        cause: String,
    ) -> InternalServerError {
        InternalServerError {
            r#type,
            exception,
            cause,
        }
    }
}
