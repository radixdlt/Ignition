#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StateEntityFungiblesPageRequest {
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

    #[serde(
        rename = "limit_per_page",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub limit_per_page: Option<Option<i32>>,

    #[serde(rename = "address")]
    pub address: String,
    #[serde(
        rename = "aggregation_level",
        skip_serializing_if = "Option::is_none"
    )]
    pub aggregation_level: Option<crate::models::ResourceAggregationLevel>,
    #[serde(rename = "opt_ins", skip_serializing_if = "Option::is_none")]
    pub opt_ins:
        Option<Box<crate::models::StateEntityFungiblesPageRequestOptIns>>,
}

impl StateEntityFungiblesPageRequest {
    pub fn new(address: String) -> StateEntityFungiblesPageRequest {
        StateEntityFungiblesPageRequest {
            at_ledger_state: None,
            cursor: None,
            limit_per_page: None,
            address,
            aggregation_level: None,
            opt_ins: None,
        }
    }
}
