#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ValidatorCollectionItemActiveInEpoch {
    #[serde(rename = "stake")]
    pub stake: String,
    #[serde(rename = "stake_percentage")]
    pub stake_percentage: f64,
    #[serde(rename = "key")]
    pub key: Box<crate::models::PublicKey>,
}

impl ValidatorCollectionItemActiveInEpoch {
    pub fn new(
        stake: String,
        stake_percentage: f64,
        key: crate::models::PublicKey,
    ) -> ValidatorCollectionItemActiveInEpoch {
        ValidatorCollectionItemActiveInEpoch {
            stake,
            stake_percentage,
            key: Box::new(key),
        }
    }
}
