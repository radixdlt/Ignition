#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ValidationErrorsAtPath {
    #[serde(rename = "path")]
    pub path: String,
    #[serde(rename = "errors")]
    pub errors: Vec<String>,
}

impl ValidationErrorsAtPath {
    pub fn new(path: String, errors: Vec<String>) -> ValidationErrorsAtPath {
        ValidationErrorsAtPath { path, errors }
    }
}
