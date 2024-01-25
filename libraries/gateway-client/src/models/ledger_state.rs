#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LedgerState {
    #[serde(rename = "network")]
    pub network: String,

    #[serde(rename = "state_version")]
    pub state_version: i64,

    #[serde(rename = "proposer_round_timestamp")]
    pub proposer_round_timestamp: String,

    #[serde(rename = "epoch")]
    pub epoch: i64,

    #[serde(rename = "round")]
    pub round: i64,
}

impl LedgerState {
    pub fn new(
        network: String,
        state_version: i64,
        proposer_round_timestamp: String,
        epoch: i64,
        round: i64,
    ) -> LedgerState {
        LedgerState {
            network,
            state_version,
            proposer_round_timestamp,
            epoch,
            round,
        }
    }
}
