#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ValidatorsUptimeRequest {
    #[serde(
        rename = "at_ledger_state",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub at_ledger_state:
        Option<Option<Box<crate::models::LedgerStateSelector>>>,
    #[serde(
        rename = "from_ledger_state",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub from_ledger_state:
        Option<Option<Box<crate::models::LedgerStateSelector>>>,
    #[serde(
        rename = "validator_addresses",
        skip_serializing_if = "Option::is_none"
    )]
    pub validator_addresses: Option<Vec<String>>,
}

impl Default for ValidatorsUptimeRequest {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidatorsUptimeRequest {
    pub fn new() -> ValidatorsUptimeRequest {
        ValidatorsUptimeRequest {
            at_ledger_state: None,
            from_ledger_state: None,
            validator_addresses: None,
        }
    }
}
