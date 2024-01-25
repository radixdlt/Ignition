#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ValidatorUptimeCollectionItem {
    #[serde(rename = "address")]
    pub address: String,

    #[serde(
        rename = "proposals_made",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub proposals_made: Option<Option<i64>>,

    #[serde(
        rename = "proposals_missed",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub proposals_missed: Option<Option<i64>>,

    #[serde(rename = "epochs_active_in")]
    pub epochs_active_in: i64,
}

impl ValidatorUptimeCollectionItem {
    pub fn new(
        address: String,
        epochs_active_in: i64,
    ) -> ValidatorUptimeCollectionItem {
        ValidatorUptimeCollectionItem {
            address,
            proposals_made: None,
            proposals_missed: None,
            epochs_active_in,
        }
    }
}
