#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RoleKey {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "module")]
    pub module: crate::models::ObjectModuleId,
}

impl RoleKey {
    pub fn new(name: String, module: crate::models::ObjectModuleId) -> RoleKey {
        RoleKey { name, module }
    }
}
