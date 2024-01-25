#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StateEntityNonFungiblesPageRequestOptIns {
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

impl Default for StateEntityNonFungiblesPageRequestOptIns {
    fn default() -> Self {
        Self::new()
    }
}

impl StateEntityNonFungiblesPageRequestOptIns {
    pub fn new() -> StateEntityNonFungiblesPageRequestOptIns {
        StateEntityNonFungiblesPageRequestOptIns {
            non_fungible_include_nfids: None,
            explicit_metadata: None,
        }
    }
}
