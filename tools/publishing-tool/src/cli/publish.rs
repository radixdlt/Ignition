use super::default_configurations::*;
use crate::utils::*;
use crate::*;
use clap::Parser;
use radix_engine_common::prelude::*;
use state_manager::RocksDBStore;
use std::path::*;
use transaction::prelude::*;

#[derive(Parser, Debug)]
pub struct Publish {
    /// The configuration that the user wants to use when publishing.
    configuration_selector: ConfigurationSelector,

    /// The path to the state manager database.
    state_manager_database_path: PathBuf,

    /// The hex-encoded private key of the notary.
    notary_ed25519_private_key_hex: String,
}

impl Publish {
    pub fn run<O: std::io::Write>(self, f: &mut O) -> Result<(), Error> {
        // Loading the private key from the passed argument.
        let notary_private_key =
            hex::decode(self.notary_ed25519_private_key_hex)
                .ok()
                .and_then(|bytes| Ed25519PrivateKey::from_bytes(&bytes).ok())
                .map(PrivateKey::Ed25519)
                .ok_or(Error::PrivateKeyError)?;

        // Loading the configuration to use for the deployment
        let configuration = self
            .configuration_selector
            .configuration(&notary_private_key);

        // Creating the network connection providers to use for the deployments
        let network_definition =
            self.configuration_selector.network_definition();
        let gateway_base_url = self.configuration_selector.gateway_base_url();
        let database =
            RocksDBStore::new_read_only(self.state_manager_database_path)
                .unwrap();
        let mut simulator_network_provider = SimulatorNetworkConnector::new(
            &database,
            network_definition.clone(),
        );
        let mut gateway_network_provider = GatewayNetworkConnector::new(
            gateway_base_url,
            network_definition.clone(),
            PollingConfiguration {
                interval_in_seconds: 10,
                retries: 10,
            },
        );

        // Running a dry run of the publishing process against the simulator
        // network provider.
        log::info!("Publishing against the simulator");
        publish(&configuration, &mut simulator_network_provider)?;

        // Running the transactions against the network.
        log::info!("Publishing against the gateway");
        let receipt = publish(&configuration, &mut gateway_network_provider)?;
        writeln!(f, "{}", to_json(&receipt, &network_definition)).unwrap();
        Ok(())
    }
}
