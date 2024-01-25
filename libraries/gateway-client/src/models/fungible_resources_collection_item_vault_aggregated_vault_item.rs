#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FungibleResourcesCollectionItemVaultAggregatedVaultItem {
    #[serde(rename = "vault_address")]
    pub vault_address: String,

    #[serde(rename = "amount")]
    pub amount: String,

    #[serde(rename = "last_updated_at_state_version")]
    pub last_updated_at_state_version: i64,
}

impl FungibleResourcesCollectionItemVaultAggregatedVaultItem {
    pub fn new(
        vault_address: String,
        amount: String,
        last_updated_at_state_version: i64,
    ) -> FungibleResourcesCollectionItemVaultAggregatedVaultItem {
        FungibleResourcesCollectionItemVaultAggregatedVaultItem {
            vault_address,
            amount,
            last_updated_at_state_version,
        }
    }
}
