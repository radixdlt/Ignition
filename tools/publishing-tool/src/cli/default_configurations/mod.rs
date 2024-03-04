use crate::*;
use clap::*;
mod mainnet_testing;

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum ConfigurationSelector {
    MainnetTesting,
}

impl ConfigurationSelector {
    pub fn configuration(
        self,
        notary_private_key: &PrivateKey,
    ) -> PublishingConfiguration {
        match self {
            Self::MainnetTesting => {
                mainnet_testing::mainnet_testing(notary_private_key)
            }
        }
    }

    pub fn gateway_base_url(self) -> String {
        match self {
            Self::MainnetTesting => "https://mainnet.radixdlt.com".to_owned(),
        }
    }

    pub fn network_definition(self) -> NetworkDefinition {
        match self {
            Self::MainnetTesting => NetworkDefinition::mainnet(),
        }
    }
}
