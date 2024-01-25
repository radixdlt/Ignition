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
pub enum TransactionStatus {
    #[serde(rename = "Unknown")]
    Unknown,
    #[serde(rename = "CommittedSuccess")]
    CommittedSuccess,
    #[serde(rename = "CommittedFailure")]
    CommittedFailure,
    #[serde(rename = "Pending")]
    Pending,
    #[serde(rename = "Rejected")]
    Rejected,
}

impl ToString for TransactionStatus {
    fn to_string(&self) -> String {
        match self {
            Self::Unknown => String::from("Unknown"),
            Self::CommittedSuccess => String::from("CommittedSuccess"),
            Self::CommittedFailure => String::from("CommittedFailure"),
            Self::Pending => String::from("Pending"),
            Self::Rejected => String::from("Rejected"),
        }
    }
}

impl Default for TransactionStatus {
    fn default() -> TransactionStatus {
        Self::Unknown
    }
}
