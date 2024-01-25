#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GatewayStatusResponse {
    #[serde(rename = "ledger_state")]
    pub ledger_state: Box<crate::models::LedgerState>,
    #[serde(rename = "release_info")]
    pub release_info: Box<crate::models::GatewayInfoResponseReleaseInfo>,
}

impl GatewayStatusResponse {
    pub fn new(
        ledger_state: crate::models::LedgerState,
        release_info: crate::models::GatewayInfoResponseReleaseInfo,
    ) -> GatewayStatusResponse {
        GatewayStatusResponse {
            ledger_state: Box::new(ledger_state),
            release_info: Box::new(release_info),
        }
    }
}
