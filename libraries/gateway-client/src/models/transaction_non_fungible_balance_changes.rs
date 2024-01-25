#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TransactionNonFungibleBalanceChanges {
    #[serde(rename = "entity_address")]
    pub entity_address: String,

    #[serde(rename = "resource_address")]
    pub resource_address: String,
    #[serde(rename = "added")]
    pub added: Vec<String>,
    #[serde(rename = "removed")]
    pub removed: Vec<String>,
}

impl TransactionNonFungibleBalanceChanges {
    pub fn new(
        entity_address: String,
        resource_address: String,
        added: Vec<String>,
        removed: Vec<String>,
    ) -> TransactionNonFungibleBalanceChanges {
        TransactionNonFungibleBalanceChanges {
            entity_address,
            resource_address,
            added,
            removed,
        }
    }
}
