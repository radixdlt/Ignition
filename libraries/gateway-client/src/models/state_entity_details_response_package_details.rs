#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StateEntityDetailsResponsePackageDetails {
    #[serde(rename = "type")]
    pub r#type: crate::models::StateEntityDetailsResponseItemDetailsType,
    #[serde(rename = "vm_type")]
    pub vm_type: crate::models::PackageVmType,

    #[serde(rename = "code_hash_hex")]
    pub code_hash_hex: String,

    #[serde(rename = "code_hex")]
    pub code_hex: String,

    #[serde(
        rename = "royalty_vault_balance",
        skip_serializing_if = "Option::is_none"
    )]
    pub royalty_vault_balance: Option<String>,
    #[serde(rename = "blueprints", skip_serializing_if = "Option::is_none")]
    pub blueprints:
        Option<Box<crate::models::StateEntityDetailsResponsePackageDetailsBlueprintCollection>>,
    #[serde(rename = "schemas", skip_serializing_if = "Option::is_none")]
    pub schemas:
        Option<Box<crate::models::StateEntityDetailsResponsePackageDetailsSchemaCollection>>,
}

impl StateEntityDetailsResponsePackageDetails {
    pub fn new(
        r#type: crate::models::StateEntityDetailsResponseItemDetailsType,
        vm_type: crate::models::PackageVmType,
        code_hash_hex: String,
        code_hex: String,
    ) -> StateEntityDetailsResponsePackageDetails {
        StateEntityDetailsResponsePackageDetails {
            r#type,
            vm_type,
            code_hash_hex,
            code_hex,
            royalty_vault_balance: None,
            blueprints: None,
            schemas: None,
        }
    }
}
