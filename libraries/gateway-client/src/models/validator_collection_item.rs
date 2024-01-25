#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ValidatorCollectionItem {
    #[serde(rename = "address")]
    pub address: String,
    #[serde(rename = "stake_vault")]
    pub stake_vault: Box<crate::models::ValidatorVaultItem>,
    #[serde(rename = "pending_xrd_withdraw_vault")]
    pub pending_xrd_withdraw_vault: Box<crate::models::ValidatorVaultItem>,
    #[serde(rename = "locked_owner_stake_unit_vault")]
    pub locked_owner_stake_unit_vault: Box<crate::models::ValidatorVaultItem>,
    #[serde(rename = "pending_owner_stake_unit_unlock_vault")]
    pub pending_owner_stake_unit_unlock_vault:
        Box<crate::models::ValidatorVaultItem>,

    #[serde(rename = "state", deserialize_with = "Option::deserialize")]
    pub state: Option<serde_json::Value>,
    #[serde(
        rename = "active_in_epoch",
        skip_serializing_if = "Option::is_none"
    )]
    pub active_in_epoch:
        Option<Box<crate::models::ValidatorCollectionItemActiveInEpoch>>,
    #[serde(rename = "metadata")]
    pub metadata: Box<crate::models::EntityMetadataCollection>,
    #[serde(rename = "effective_fee_factor")]
    pub effective_fee_factor:
        Box<crate::models::ValidatorCollectionItemEffectiveFeeFactor>,
}

impl ValidatorCollectionItem {
    pub fn new(
        address: String,
        stake_vault: crate::models::ValidatorVaultItem,
        pending_xrd_withdraw_vault: crate::models::ValidatorVaultItem,
        locked_owner_stake_unit_vault: crate::models::ValidatorVaultItem,
        pending_owner_stake_unit_unlock_vault: crate::models::ValidatorVaultItem,
        state: Option<serde_json::Value>,
        metadata: crate::models::EntityMetadataCollection,
        effective_fee_factor: crate::models::ValidatorCollectionItemEffectiveFeeFactor,
    ) -> ValidatorCollectionItem {
        ValidatorCollectionItem {
            address,
            stake_vault: Box::new(stake_vault),
            pending_xrd_withdraw_vault: Box::new(pending_xrd_withdraw_vault),
            locked_owner_stake_unit_vault: Box::new(
                locked_owner_stake_unit_vault,
            ),
            pending_owner_stake_unit_unlock_vault: Box::new(
                pending_owner_stake_unit_unlock_vault,
            ),
            state,
            active_in_epoch: None,
            metadata: Box::new(metadata),
            effective_fee_factor: Box::new(effective_fee_factor),
        }
    }
}
