#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FromLedgerStateMixin {
    #[serde(
        rename = "from_ledger_state",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub from_ledger_state:
        Option<Option<Box<crate::models::LedgerStateSelector>>>,
}

impl Default for FromLedgerStateMixin {
    fn default() -> Self {
        Self::new()
    }
}

impl FromLedgerStateMixin {
    pub fn new() -> FromLedgerStateMixin {
        FromLedgerStateMixin {
            from_ledger_state: None,
        }
    }
}
