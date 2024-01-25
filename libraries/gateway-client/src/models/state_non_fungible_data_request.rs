#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StateNonFungibleDataRequest {
    #[serde(
        rename = "at_ledger_state",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub at_ledger_state:
        Option<Option<Box<crate::models::LedgerStateSelector>>>,

    #[serde(rename = "resource_address")]
    pub resource_address: String,

    #[serde(rename = "non_fungible_ids")]
    pub non_fungible_ids: Vec<String>,
}

impl StateNonFungibleDataRequest {
    pub fn new(
        resource_address: String,
        non_fungible_ids: Vec<String>,
    ) -> StateNonFungibleDataRequest {
        StateNonFungibleDataRequest {
            at_ledger_state: None,
            resource_address,
            non_fungible_ids,
        }
    }
}
