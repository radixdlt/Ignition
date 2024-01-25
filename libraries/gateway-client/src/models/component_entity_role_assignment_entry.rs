#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ComponentEntityRoleAssignmentEntry {
    #[serde(rename = "role_key")]
    pub role_key: Box<crate::models::RoleKey>,
    #[serde(rename = "assignment")]
    pub assignment:
        Box<crate::models::ComponentEntityRoleAssignmentEntryAssignment>,
    #[serde(rename = "updater_roles", skip_serializing_if = "Option::is_none")]
    pub updater_roles: Option<Vec<crate::models::RoleKey>>,
}

impl ComponentEntityRoleAssignmentEntry {
    pub fn new(
        role_key: crate::models::RoleKey,
        assignment: crate::models::ComponentEntityRoleAssignmentEntryAssignment,
    ) -> ComponentEntityRoleAssignmentEntry {
        ComponentEntityRoleAssignmentEntry {
            role_key: Box::new(role_key),
            assignment: Box::new(assignment),
            updater_roles: None,
        }
    }
}
