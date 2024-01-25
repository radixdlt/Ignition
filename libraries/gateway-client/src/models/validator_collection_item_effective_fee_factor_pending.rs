#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ValidatorCollectionItemEffectiveFeeFactorPending {
    #[serde(rename = "fee_factor")]
    pub fee_factor: String,
    #[serde(rename = "effective_at_epoch")]
    pub effective_at_epoch: i64,
}

impl ValidatorCollectionItemEffectiveFeeFactorPending {
    pub fn new(
        fee_factor: String,
        effective_at_epoch: i64,
    ) -> ValidatorCollectionItemEffectiveFeeFactorPending {
        ValidatorCollectionItemEffectiveFeeFactorPending {
            fee_factor,
            effective_at_epoch,
        }
    }
}
