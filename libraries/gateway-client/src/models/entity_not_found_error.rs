#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EntityNotFoundError {
    #[serde(rename = "type")]
    pub r#type: String,

    #[serde(rename = "address")]
    pub address: String,
}

impl EntityNotFoundError {
    pub fn new(r#type: String, address: String) -> EntityNotFoundError {
        EntityNotFoundError { r#type, address }
    }
}
