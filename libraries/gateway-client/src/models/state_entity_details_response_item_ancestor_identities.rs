#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StateEntityDetailsResponseItemAncestorIdentities {
    #[serde(
        rename = "parent_address",
        skip_serializing_if = "Option::is_none"
    )]
    pub parent_address: Option<String>,

    #[serde(rename = "owner_address", skip_serializing_if = "Option::is_none")]
    pub owner_address: Option<String>,

    #[serde(
        rename = "global_address",
        skip_serializing_if = "Option::is_none"
    )]
    pub global_address: Option<String>,
}

impl Default for StateEntityDetailsResponseItemAncestorIdentities {
    fn default() -> Self {
        Self::new()
    }
}

impl StateEntityDetailsResponseItemAncestorIdentities {
    pub fn new() -> StateEntityDetailsResponseItemAncestorIdentities {
        StateEntityDetailsResponseItemAncestorIdentities {
            parent_address: None,
            owner_address: None,
            global_address: None,
        }
    }
}
