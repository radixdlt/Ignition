#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StateEntityDetailsRequest {
    #[serde(
        rename = "at_ledger_state",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub at_ledger_state:
        Option<Option<Box<crate::models::LedgerStateSelector>>>,
    #[serde(rename = "opt_ins", skip_serializing_if = "Option::is_none")]
    pub opt_ins: Option<Box<crate::models::StateEntityDetailsOptIns>>,

    #[serde(rename = "addresses")]
    pub addresses: Vec<String>,
    #[serde(
        rename = "aggregation_level",
        skip_serializing_if = "Option::is_none"
    )]
    pub aggregation_level: Option<crate::models::ResourceAggregationLevel>,
}

impl StateEntityDetailsRequest {
    pub fn new(addresses: Vec<String>) -> StateEntityDetailsRequest {
        StateEntityDetailsRequest {
            at_ledger_state: None,
            opt_ins: None,
            addresses,
            aggregation_level: None,
        }
    }
}
