#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StateNonFungibleDetailsResponseItem {
    #[serde(rename = "is_burned")]
    pub is_burned: bool,

    #[serde(rename = "non_fungible_id")]
    pub non_fungible_id: String,
    #[serde(rename = "data", skip_serializing_if = "Option::is_none")]
    pub data: Option<Box<crate::models::ScryptoSborValue>>,

    #[serde(rename = "last_updated_at_state_version")]
    pub last_updated_at_state_version: i64,
}

impl StateNonFungibleDetailsResponseItem {
    pub fn new(
        is_burned: bool,
        non_fungible_id: String,
        last_updated_at_state_version: i64,
    ) -> StateNonFungibleDetailsResponseItem {
        StateNonFungibleDetailsResponseItem {
            is_burned,
            non_fungible_id,
            data: None,
            last_updated_at_state_version,
        }
    }
}
