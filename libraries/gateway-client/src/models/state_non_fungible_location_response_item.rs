#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StateNonFungibleLocationResponseItem {
    #[serde(rename = "non_fungible_id")]
    pub non_fungible_id: String,

    #[serde(
        rename = "owning_vault_address",
        skip_serializing_if = "Option::is_none"
    )]
    pub owning_vault_address: Option<String>,
    #[serde(rename = "is_burned")]
    pub is_burned: bool,

    #[serde(rename = "last_updated_at_state_version")]
    pub last_updated_at_state_version: i64,
}

impl StateNonFungibleLocationResponseItem {
    pub fn new(
        non_fungible_id: String,
        is_burned: bool,
        last_updated_at_state_version: i64,
    ) -> StateNonFungibleLocationResponseItem {
        StateNonFungibleLocationResponseItem {
            non_fungible_id,
            owning_vault_address: None,
            is_burned,
            last_updated_at_state_version,
        }
    }
}
