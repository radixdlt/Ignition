mod mainnet_testing;
mod stokenet_testing;

use clap::*;
use publishing_tool::publishing::*;
use transaction::prelude::*;

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum ConfigurationSelector {
    MainnetTesting,
    StokenetTesting,
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
            Self::StokenetTesting => {
                stokenet_testing::stokenet_testing(notary_private_key)
            }
        }
    }

    pub fn gateway_base_url(self) -> String {
        match self {
            Self::MainnetTesting => "https://mainnet.radixdlt.com".to_owned(),
            Self::StokenetTesting => "https://stokenet.radixdlt.com".to_owned(),
        }
    }

    pub fn network_definition(self) -> NetworkDefinition {
        match self {
            Self::MainnetTesting => NetworkDefinition::mainnet(),
            Self::StokenetTesting => NetworkDefinition::stokenet(),
        }
    }
}
