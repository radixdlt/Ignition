#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StateEntityDetailsResponseFungibleResourceDetails {
    #[serde(rename = "type")]
    pub r#type: crate::models::StateEntityDetailsResponseItemDetailsType,
    #[serde(rename = "role_assignments")]
    pub role_assignments: Box<crate::models::ComponentEntityRoleAssignments>,
    #[serde(rename = "divisibility")]
    pub divisibility: i32,

    #[serde(rename = "total_supply")]
    pub total_supply: String,

    #[serde(rename = "total_minted")]
    pub total_minted: String,

    #[serde(rename = "total_burned")]
    pub total_burned: String,
}

impl StateEntityDetailsResponseFungibleResourceDetails {
    pub fn new(
        r#type: crate::models::StateEntityDetailsResponseItemDetailsType,
        role_assignments: crate::models::ComponentEntityRoleAssignments,
        divisibility: i32,
        total_supply: String,
        total_minted: String,
        total_burned: String,
    ) -> StateEntityDetailsResponseFungibleResourceDetails {
        StateEntityDetailsResponseFungibleResourceDetails {
            r#type,
            role_assignments: Box::new(role_assignments),
            divisibility,
            total_supply,
            total_minted,
            total_burned,
        }
    }
}
