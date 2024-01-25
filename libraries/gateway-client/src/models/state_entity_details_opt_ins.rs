#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StateEntityDetailsOptIns {
    #[serde(
        rename = "ancestor_identities",
        skip_serializing_if = "Option::is_none"
    )]
    pub ancestor_identities: Option<bool>,

    #[serde(
        rename = "component_royalty_vault_balance",
        skip_serializing_if = "Option::is_none"
    )]
    pub component_royalty_vault_balance: Option<bool>,

    #[serde(
        rename = "package_royalty_vault_balance",
        skip_serializing_if = "Option::is_none"
    )]
    pub package_royalty_vault_balance: Option<bool>,

    #[serde(
        rename = "non_fungible_include_nfids",
        skip_serializing_if = "Option::is_none"
    )]
    pub non_fungible_include_nfids: Option<bool>,

    #[serde(
        rename = "explicit_metadata",
        skip_serializing_if = "Option::is_none"
    )]
    pub explicit_metadata: Option<Vec<String>>,
}

impl Default for StateEntityDetailsOptIns {
    fn default() -> Self {
        Self::new()
    }
}

impl StateEntityDetailsOptIns {
    pub fn new() -> StateEntityDetailsOptIns {
        StateEntityDetailsOptIns {
            ancestor_identities: None,
            component_royalty_vault_balance: None,
            package_royalty_vault_balance: None,
            non_fungible_include_nfids: None,
            explicit_metadata: None,
        }
    }
}
