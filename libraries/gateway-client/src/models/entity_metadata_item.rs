#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EntityMetadataItem {
    #[serde(rename = "key")]
    pub key: String,
    #[serde(rename = "value")]
    pub value: Box<crate::models::EntityMetadataItemValue>,
    #[serde(rename = "is_locked")]
    pub is_locked: bool,

    #[serde(rename = "last_updated_at_state_version")]
    pub last_updated_at_state_version: i64,
}

impl EntityMetadataItem {
    pub fn new(
        key: String,
        value: crate::models::EntityMetadataItemValue,
        is_locked: bool,
        last_updated_at_state_version: i64,
    ) -> EntityMetadataItem {
        EntityMetadataItem {
            key,
            value: Box::new(value),
            is_locked,
            last_updated_at_state_version,
        }
    }
}
