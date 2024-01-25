#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StateEntityFungiblesPageRequestOptIns {
    #[serde(
        rename = "explicit_metadata",
        skip_serializing_if = "Option::is_none"
    )]
    pub explicit_metadata: Option<Vec<String>>,
}

impl Default for StateEntityFungiblesPageRequestOptIns {
    fn default() -> Self {
        Self::new()
    }
}

impl StateEntityFungiblesPageRequestOptIns {
    pub fn new() -> StateEntityFungiblesPageRequestOptIns {
        StateEntityFungiblesPageRequestOptIns {
            explicit_metadata: None,
        }
    }
}
