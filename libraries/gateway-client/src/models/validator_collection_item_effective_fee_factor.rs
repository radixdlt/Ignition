#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ValidatorCollectionItemEffectiveFeeFactor {
    #[serde(rename = "current")]
    pub current:
        Box<crate::models::ValidatorCollectionItemEffectiveFeeFactorCurrent>,
    #[serde(
        rename = "pending",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub pending: Option<
        Option<
            Box<
                crate::models::ValidatorCollectionItemEffectiveFeeFactorPending,
            >,
        >,
    >,
}

impl ValidatorCollectionItemEffectiveFeeFactor {
    pub fn new(
        current: crate::models::ValidatorCollectionItemEffectiveFeeFactorCurrent,
    ) -> ValidatorCollectionItemEffectiveFeeFactor {
        ValidatorCollectionItemEffectiveFeeFactor {
            current: Box::new(current),
            pending: None,
        }
    }
}
