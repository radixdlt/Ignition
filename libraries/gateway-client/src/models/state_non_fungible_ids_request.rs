#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StateNonFungibleIdsRequest {
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

    #[serde(rename = "resource_address")]
    pub resource_address: String,
}

impl StateNonFungibleIdsRequest {
    pub fn new(resource_address: String) -> StateNonFungibleIdsRequest {
        StateNonFungibleIdsRequest {
            at_ledger_state: None,
            cursor: None,
            limit_per_page: None,
            resource_address,
        }
    }
}
