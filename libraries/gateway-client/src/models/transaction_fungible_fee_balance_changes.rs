#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TransactionFungibleFeeBalanceChanges {
    #[serde(rename = "type")]
    pub r#type: crate::models::TransactionFungibleFeeBalanceChangeType,

    #[serde(rename = "entity_address")]
    pub entity_address: String,

    #[serde(rename = "resource_address")]
    pub resource_address: String,

    #[serde(rename = "balance_change")]
    pub balance_change: String,
}

impl TransactionFungibleFeeBalanceChanges {
    pub fn new(
        r#type: crate::models::TransactionFungibleFeeBalanceChangeType,
        entity_address: String,
        resource_address: String,
        balance_change: String,
    ) -> TransactionFungibleFeeBalanceChanges {
        TransactionFungibleFeeBalanceChanges {
            r#type,
            entity_address,
            resource_address,
            balance_change,
        }
    }
}
