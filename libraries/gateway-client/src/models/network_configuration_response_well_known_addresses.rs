#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NetworkConfigurationResponseWellKnownAddresses {
    #[serde(rename = "xrd")]
    pub xrd: String,

    #[serde(rename = "secp256k1_signature_virtual_badge")]
    pub secp256k1_signature_virtual_badge: String,

    #[serde(rename = "ed25519_signature_virtual_badge")]
    pub ed25519_signature_virtual_badge: String,

    #[serde(rename = "package_of_direct_caller_virtual_badge")]
    pub package_of_direct_caller_virtual_badge: String,

    #[serde(rename = "global_caller_virtual_badge")]
    pub global_caller_virtual_badge: String,

    #[serde(rename = "system_transaction_badge")]
    pub system_transaction_badge: String,

    #[serde(rename = "package_owner_badge")]
    pub package_owner_badge: String,

    #[serde(rename = "validator_owner_badge")]
    pub validator_owner_badge: String,

    #[serde(rename = "account_owner_badge")]
    pub account_owner_badge: String,

    #[serde(rename = "identity_owner_badge")]
    pub identity_owner_badge: String,

    #[serde(rename = "package_package")]
    pub package_package: String,

    #[serde(rename = "resource_package")]
    pub resource_package: String,

    #[serde(rename = "account_package")]
    pub account_package: String,

    #[serde(rename = "identity_package")]
    pub identity_package: String,

    #[serde(rename = "consensus_manager_package")]
    pub consensus_manager_package: String,

    #[serde(rename = "access_controller_package")]
    pub access_controller_package: String,

    #[serde(rename = "transaction_processor_package")]
    pub transaction_processor_package: String,

    #[serde(rename = "metadata_module_package")]
    pub metadata_module_package: String,

    #[serde(rename = "royalty_module_package")]
    pub royalty_module_package: String,

    #[serde(rename = "access_rules_package")]
    pub access_rules_package: String,

    #[serde(rename = "genesis_helper_package")]
    pub genesis_helper_package: String,

    #[serde(rename = "faucet_package")]
    pub faucet_package: String,

    #[serde(rename = "consensus_manager")]
    pub consensus_manager: String,

    #[serde(rename = "genesis_helper")]
    pub genesis_helper: String,

    #[serde(rename = "faucet")]
    pub faucet: String,

    #[serde(rename = "pool_package")]
    pub pool_package: String,
}

impl NetworkConfigurationResponseWellKnownAddresses {
    pub fn new(
        xrd: String,
        secp256k1_signature_virtual_badge: String,
        ed25519_signature_virtual_badge: String,
        package_of_direct_caller_virtual_badge: String,
        global_caller_virtual_badge: String,
        system_transaction_badge: String,
        package_owner_badge: String,
        validator_owner_badge: String,
        account_owner_badge: String,
        identity_owner_badge: String,
        package_package: String,
        resource_package: String,
        account_package: String,
        identity_package: String,
        consensus_manager_package: String,
        access_controller_package: String,
        transaction_processor_package: String,
        metadata_module_package: String,
        royalty_module_package: String,
        access_rules_package: String,
        genesis_helper_package: String,
        faucet_package: String,
        consensus_manager: String,
        genesis_helper: String,
        faucet: String,
        pool_package: String,
    ) -> NetworkConfigurationResponseWellKnownAddresses {
        NetworkConfigurationResponseWellKnownAddresses {
            xrd,
            secp256k1_signature_virtual_badge,
            ed25519_signature_virtual_badge,
            package_of_direct_caller_virtual_badge,
            global_caller_virtual_badge,
            system_transaction_badge,
            package_owner_badge,
            validator_owner_badge,
            account_owner_badge,
            identity_owner_badge,
            package_package,
            resource_package,
            account_package,
            identity_package,
            consensus_manager_package,
            access_controller_package,
            transaction_processor_package,
            metadata_module_package,
            royalty_module_package,
            access_rules_package,
            genesis_helper_package,
            faucet_package,
            consensus_manager,
            genesis_helper,
            faucet,
            pool_package,
        }
    }
}
