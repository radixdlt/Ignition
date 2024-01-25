#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TransactionFungibleBalanceChanges {
    #[serde(rename = "entity_address")]
    pub entity_address: String,

    #[serde(rename = "resource_address")]
    pub resource_address: String,

    #[serde(rename = "balance_change")]
    pub balance_change: String,
}

impl TransactionFungibleBalanceChanges {
    pub fn new(
        entity_address: String,
        resource_address: String,
        balance_change: String,
    ) -> TransactionFungibleBalanceChanges {
        TransactionFungibleBalanceChanges {
            entity_address,
            resource_address,
            balance_change,
        }
    }
}
