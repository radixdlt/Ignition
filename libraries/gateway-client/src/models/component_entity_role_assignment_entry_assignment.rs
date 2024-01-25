#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ComponentEntityRoleAssignmentEntryAssignment {
    #[serde(rename = "resolution")]
    pub resolution: crate::models::RoleAssignmentResolution,

    #[serde(rename = "explicit_rule", skip_serializing_if = "Option::is_none")]
    pub explicit_rule: Option<serde_json::Value>,
}

impl ComponentEntityRoleAssignmentEntryAssignment {
    pub fn new(
        resolution: crate::models::RoleAssignmentResolution,
    ) -> ComponentEntityRoleAssignmentEntryAssignment {
        ComponentEntityRoleAssignmentEntryAssignment {
            resolution,
            explicit_rule: None,
        }
    }
}
