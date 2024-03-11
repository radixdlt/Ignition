#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StateEntityNonFungibleResourceVaultsPageResponse {
    #[serde(rename = "ledger_state")]
    pub ledger_state: Box<crate::models::LedgerState>,

    #[serde(
        rename = "total_count",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub total_count: Option<Option<i64>>,

    #[serde(
        rename = "next_cursor",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub next_cursor: Option<Option<String>>,
    #[serde(rename = "items")]
    pub items: Vec<crate::models::NonFungibleResourcesCollectionItemVaultAggregatedVaultItem>,

    #[serde(rename = "address")]
    pub address: String,

    #[serde(rename = "resource_address")]
    pub resource_address: String,
}

impl StateEntityNonFungibleResourceVaultsPageResponse {
    pub fn new(
        ledger_state: crate::models::LedgerState,
        items: Vec<crate::models::NonFungibleResourcesCollectionItemVaultAggregatedVaultItem>,
        address: String,
        resource_address: String,
    ) -> StateEntityNonFungibleResourceVaultsPageResponse {
        StateEntityNonFungibleResourceVaultsPageResponse {
            ledger_state: Box::new(ledger_state),
            total_count: None,
            next_cursor: None,
            items,
            address,
            resource_address,
        }
    }
}
