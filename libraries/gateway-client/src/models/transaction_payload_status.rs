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
pub enum TransactionPayloadStatus {
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
    #[serde(rename = "TemporarilyRejected")]
    TemporarilyRejected,
    #[serde(rename = "Pending")]
    Pending,
}

impl ToString for TransactionPayloadStatus {
    fn to_string(&self) -> String {
        match self {
            Self::Unknown => String::from("Unknown"),
            Self::CommittedSuccess => String::from("CommittedSuccess"),
            Self::CommittedFailure => String::from("CommittedFailure"),
            Self::CommitPendingOutcomeUnknown => {
                String::from("CommitPendingOutcomeUnknown")
            }
            Self::PermanentlyRejected => String::from("PermanentlyRejected"),
            Self::TemporarilyRejected => String::from("TemporarilyRejected"),
            Self::Pending => String::from("Pending"),
        }
    }
}

impl Default for TransactionPayloadStatus {
    fn default() -> TransactionPayloadStatus {
        Self::Unknown
    }
}
