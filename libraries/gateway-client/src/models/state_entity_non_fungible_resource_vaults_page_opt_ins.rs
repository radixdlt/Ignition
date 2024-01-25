#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StateEntityNonFungibleResourceVaultsPageOptIns {
    #[serde(
        rename = "non_fungible_include_nfids",
        skip_serializing_if = "Option::is_none"
    )]
    pub non_fungible_include_nfids: Option<bool>,
}

impl Default for StateEntityNonFungibleResourceVaultsPageOptIns {
    fn default() -> Self {
        Self::new()
    }
}

impl StateEntityNonFungibleResourceVaultsPageOptIns {
    pub fn new() -> StateEntityNonFungibleResourceVaultsPageOptIns {
        StateEntityNonFungibleResourceVaultsPageOptIns {
            non_fungible_include_nfids: None,
        }
    }
}
