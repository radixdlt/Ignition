#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TransactionBalanceChanges {
    #[serde(rename = "fungible_fee_balance_changes")]
    pub fungible_fee_balance_changes:
        Vec<crate::models::TransactionFungibleFeeBalanceChanges>,

    #[serde(rename = "fungible_balance_changes")]
    pub fungible_balance_changes:
        Vec<crate::models::TransactionFungibleBalanceChanges>,

    #[serde(rename = "non_fungible_balance_changes")]
    pub non_fungible_balance_changes:
        Vec<crate::models::TransactionNonFungibleBalanceChanges>,
}

impl TransactionBalanceChanges {
    pub fn new(
        fungible_fee_balance_changes: Vec<
            crate::models::TransactionFungibleFeeBalanceChanges,
        >,
        fungible_balance_changes: Vec<
            crate::models::TransactionFungibleBalanceChanges,
        >,
        non_fungible_balance_changes: Vec<
            crate::models::TransactionNonFungibleBalanceChanges,
        >,
    ) -> TransactionBalanceChanges {
        TransactionBalanceChanges {
            fungible_fee_balance_changes,
            fungible_balance_changes,
            non_fungible_balance_changes,
        }
    }
}
