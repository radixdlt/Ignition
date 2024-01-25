#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StateEntityDetailsResponseComponentDetails {
    #[serde(rename = "type")]
    pub r#type: crate::models::StateEntityDetailsResponseItemDetailsType,

    #[serde(
        rename = "package_address",
        skip_serializing_if = "Option::is_none"
    )]
    pub package_address: Option<String>,
    #[serde(rename = "blueprint_name")]
    pub blueprint_name: String,
    #[serde(rename = "blueprint_version")]
    pub blueprint_version: String,

    #[serde(rename = "state", skip_serializing_if = "Option::is_none")]
    pub state: Option<serde_json::Value>,
    #[serde(
        rename = "role_assignments",
        skip_serializing_if = "Option::is_none"
    )]
    pub role_assignments:
        Option<Box<crate::models::ComponentEntityRoleAssignments>>,

    #[serde(
        rename = "royalty_vault_balance",
        skip_serializing_if = "Option::is_none"
    )]
    pub royalty_vault_balance: Option<String>,
}

impl StateEntityDetailsResponseComponentDetails {
    pub fn new(
        r#type: crate::models::StateEntityDetailsResponseItemDetailsType,
        blueprint_name: String,
        blueprint_version: String,
    ) -> StateEntityDetailsResponseComponentDetails {
        StateEntityDetailsResponseComponentDetails {
            r#type,
            package_address: None,
            blueprint_name,
            blueprint_version,
            state: None,
            role_assignments: None,
            royalty_vault_balance: None,
        }
    }
}
