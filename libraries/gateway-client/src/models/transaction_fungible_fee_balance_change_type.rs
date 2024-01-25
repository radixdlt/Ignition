#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum TransactionFungibleFeeBalanceChangeType {
    #[serde(rename = "FeePayment")]
    FeePayment,
    #[serde(rename = "FeeDistributed")]
    FeeDistributed,
    #[serde(rename = "TipDistributed")]
    TipDistributed,
    #[serde(rename = "RoyaltyDistributed")]
    RoyaltyDistributed,
}

impl ToString for TransactionFungibleFeeBalanceChangeType {
    fn to_string(&self) -> String {
        match self {
            Self::FeePayment => String::from("FeePayment"),
            Self::FeeDistributed => String::from("FeeDistributed"),
            Self::TipDistributed => String::from("TipDistributed"),
            Self::RoyaltyDistributed => String::from("RoyaltyDistributed"),
        }
    }
}

impl Default for TransactionFungibleFeeBalanceChangeType {
    fn default() -> TransactionFungibleFeeBalanceChangeType {
        Self::FeePayment
    }
}
