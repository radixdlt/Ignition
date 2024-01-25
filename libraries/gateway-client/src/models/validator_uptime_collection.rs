#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ValidatorUptimeCollection {
    #[serde(rename = "items")]
    pub items: Vec<crate::models::ValidatorUptimeCollectionItem>,
}

impl ValidatorUptimeCollection {
    pub fn new(
        items: Vec<crate::models::ValidatorUptimeCollectionItem>,
    ) -> ValidatorUptimeCollection {
        ValidatorUptimeCollection { items }
    }
}
