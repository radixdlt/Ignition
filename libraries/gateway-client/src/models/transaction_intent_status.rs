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
pub enum TransactionIntentStatus {
    #[serde(rename = "Unknown")]
    Unknown,
    #[serde(rename = "CommittedSuccess")]
    CommittedSuccess,
    #[serde(rename = "CommittedFailure")]
    CommittedFailure,
    #[serde(rename = "CommitPendingOutcomeUnknown")]
    CommitPendingOutcomeUnknown,
    #[serde(rename = "PermanentlyRejected")]
    PermanentlyRejected,
    #[serde(rename = "LikelyButNotCertainRejection")]
    LikelyButNotCertainRejection,
    #[serde(rename = "Pending")]
    Pending,
}

impl ToString for TransactionIntentStatus {
    fn to_string(&self) -> String {
        match self {
            Self::Unknown => String::from("Unknown"),
            Self::CommittedSuccess => String::from("CommittedSuccess"),
            Self::CommittedFailure => String::from("CommittedFailure"),
            Self::CommitPendingOutcomeUnknown => {
                String::from("CommitPendingOutcomeUnknown")
            }
            Self::PermanentlyRejected => String::from("PermanentlyRejected"),
            Self::LikelyButNotCertainRejection => {
                String::from("LikelyButNotCertainRejection")
            }
            Self::Pending => String::from("Pending"),
        }
    }
}

impl Default for TransactionIntentStatus {
    fn default() -> TransactionIntentStatus {
        Self::Unknown
    }
}
