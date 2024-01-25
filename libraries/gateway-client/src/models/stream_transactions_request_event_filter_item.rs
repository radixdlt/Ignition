#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StreamTransactionsRequestEventFilterItem {
    #[serde(rename = "event")]
    pub event: Event,

    #[serde(
        rename = "emitter_address",
        skip_serializing_if = "Option::is_none"
    )]
    pub emitter_address: Option<String>,

    #[serde(
        rename = "resource_address",
        skip_serializing_if = "Option::is_none"
    )]
    pub resource_address: Option<String>,
}

impl StreamTransactionsRequestEventFilterItem {
    pub fn new(event: Event) -> StreamTransactionsRequestEventFilterItem {
        StreamTransactionsRequestEventFilterItem {
            event,
            emitter_address: None,
            resource_address: None,
        }
    }
}

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
pub enum Event {
    #[serde(rename = "Deposit")]
    Deposit,
    #[serde(rename = "Withdrawal")]
    Withdrawal,
}

impl Default for Event {
    fn default() -> Event {
        Self::Deposit
    }
}
