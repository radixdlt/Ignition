#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ValidatorCollectionItemEffectiveFeeFactorCurrent {
    #[serde(rename = "fee_factor")]
    pub fee_factor: String,
}

impl ValidatorCollectionItemEffectiveFeeFactorCurrent {
    pub fn new(
        fee_factor: String,
    ) -> ValidatorCollectionItemEffectiveFeeFactorCurrent {
        ValidatorCollectionItemEffectiveFeeFactorCurrent { fee_factor }
    }
}
