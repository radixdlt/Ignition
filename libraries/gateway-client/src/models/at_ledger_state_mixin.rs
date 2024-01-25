#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AtLedgerStateMixin {
    #[serde(
        rename = "at_ledger_state",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub at_ledger_state:
        Option<Option<Box<crate::models::LedgerStateSelector>>>,
}

impl Default for AtLedgerStateMixin {
    fn default() -> Self {
        Self::new()
    }
}

impl AtLedgerStateMixin {
    pub fn new() -> AtLedgerStateMixin {
        AtLedgerStateMixin {
            at_ledger_state: None,
        }
    }
}
