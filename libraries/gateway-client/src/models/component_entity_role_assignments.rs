#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ComponentEntityRoleAssignments {
    #[serde(rename = "owner")]
    pub owner: serde_json::Value,
    #[serde(rename = "entries")]
    pub entries: Vec<crate::models::ComponentEntityRoleAssignmentEntry>,
}

impl ComponentEntityRoleAssignments {
    pub fn new(
        owner: serde_json::Value,
        entries: Vec<crate::models::ComponentEntityRoleAssignmentEntry>,
    ) -> ComponentEntityRoleAssignments {
        ComponentEntityRoleAssignments { owner, entries }
    }
}
