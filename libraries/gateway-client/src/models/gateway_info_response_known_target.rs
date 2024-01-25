#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GatewayInfoResponseKnownTarget {
    #[serde(rename = "state_version")]
    pub state_version: i64,
}

impl GatewayInfoResponseKnownTarget {
    pub fn new(state_version: i64) -> GatewayInfoResponseKnownTarget {
        GatewayInfoResponseKnownTarget { state_version }
    }
}
