#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NonFungibleResourcesCollectionItemVaultAggregatedVaultItem {
    #[serde(rename = "total_count")]
    pub total_count: i64,

    #[serde(
        rename = "next_cursor",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub next_cursor: Option<Option<String>>,
    #[serde(rename = "items", skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<String>>,

    #[serde(rename = "vault_address")]
    pub vault_address: String,

    #[serde(rename = "last_updated_at_state_version")]
    pub last_updated_at_state_version: i64,
}

impl NonFungibleResourcesCollectionItemVaultAggregatedVaultItem {
    pub fn new(
        total_count: i64,
        vault_address: String,
        last_updated_at_state_version: i64,
    ) -> NonFungibleResourcesCollectionItemVaultAggregatedVaultItem {
        NonFungibleResourcesCollectionItemVaultAggregatedVaultItem {
            total_count,
            next_cursor: None,
            items: None,
            vault_address,
            last_updated_at_state_version,
        }
    }
}
