#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LedgerStateSelector {
    #[serde(
        rename = "state_version",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub state_version: Option<Option<i64>>,

    #[serde(
        rename = "timestamp",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub timestamp: Option<Option<String>>,

    #[serde(
        rename = "epoch",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub epoch: Option<Option<i64>>,

    #[serde(
        rename = "round",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub round: Option<Option<i64>>,
}

impl Default for LedgerStateSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl LedgerStateSelector {
    pub fn new() -> LedgerStateSelector {
        LedgerStateSelector {
            state_version: None,
            timestamp: None,
            epoch: None,
            round: None,
        }
    }
}
