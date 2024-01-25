#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ValidatorVaultItem {
    #[serde(rename = "balance")]
    pub balance: String,
    #[serde(rename = "last_changed_at_state_version")]
    pub last_changed_at_state_version: i64,

    #[serde(rename = "address")]
    pub address: String,
}

impl ValidatorVaultItem {
    pub fn new(
        balance: String,
        last_changed_at_state_version: i64,
        address: String,
    ) -> ValidatorVaultItem {
        ValidatorVaultItem {
            balance,
            last_changed_at_state_version,
            address,
        }
    }
}
