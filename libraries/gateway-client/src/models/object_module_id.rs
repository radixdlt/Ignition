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
pub enum ObjectModuleId {
    #[serde(rename = "Main")]
    Main,
    #[serde(rename = "Metadata")]
    Metadata,
    #[serde(rename = "Royalty")]
    Royalty,
    #[serde(rename = "RoleAssignment")]
    RoleAssignment,
}

impl ToString for ObjectModuleId {
    fn to_string(&self) -> String {
        match self {
            Self::Main => String::from("Main"),
            Self::Metadata => String::from("Metadata"),
            Self::Royalty => String::from("Royalty"),
            Self::RoleAssignment => String::from("RoleAssignment"),
        }
    }
}

impl Default for ObjectModuleId {
    fn default() -> ObjectModuleId {
        Self::Main
    }
}
