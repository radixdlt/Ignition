#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StateValidatorsListRequest {
    #[serde(
        rename = "at_ledger_state",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub at_ledger_state:
        Option<Option<Box<crate::models::LedgerStateSelector>>>,

    #[serde(
        rename = "cursor",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub cursor: Option<Option<String>>,
}

impl Default for StateValidatorsListRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl StateValidatorsListRequest {
    pub fn new() -> StateValidatorsListRequest {
        StateValidatorsListRequest {
            at_ledger_state: None,
            cursor: None,
        }
    }
}
