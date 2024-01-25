#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum RoleAssignmentResolution {
    #[serde(rename = "Explicit")]
    Explicit,
    #[serde(rename = "Owner")]
    Owner,
}

impl ToString for RoleAssignmentResolution {
    fn to_string(&self) -> String {
        match self {
            Self::Explicit => String::from("Explicit"),
            Self::Owner => String::from("Owner"),
        }
    }
}

impl Default for RoleAssignmentResolution {
    fn default() -> RoleAssignmentResolution {
        Self::Explicit
    }
}
