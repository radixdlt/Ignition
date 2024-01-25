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
pub enum TransactionPayloadGatewayHandlingStatus {
    #[serde(rename = "HandlingSubmission")]
    HandlingSubmission,
    #[serde(rename = "Concluded")]
    Concluded,
}

impl ToString for TransactionPayloadGatewayHandlingStatus {
    fn to_string(&self) -> String {
        match self {
            Self::HandlingSubmission => String::from("HandlingSubmission"),
            Self::Concluded => String::from("Concluded"),
        }
    }
}

impl Default for TransactionPayloadGatewayHandlingStatus {
    fn default() -> TransactionPayloadGatewayHandlingStatus {
        Self::HandlingSubmission
    }
}
