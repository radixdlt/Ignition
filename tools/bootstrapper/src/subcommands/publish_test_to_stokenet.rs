use crate::constants::*;
use crate::error::*;
use crate::gateway::transaction_committed_details::{Emitter, Event};
use crate::gateway::*;
use crate::utils::*;

use ::bootstrap::*;
use package_loader::*;

use clap::Parser;
use serde::{Deserialize, Serialize};

use radix_engine_interface::blueprints::package::*;
use transaction::builder::*;
use transaction::prelude::*;

/// Publishes project Ignition to stokenet with the test configuration.
#[derive(Parser, Debug)]
pub struct PublishTestToStokenet {
    /// The path to write the addresses out to, this should be to a JSON file.
    pub output_path: String,
}

impl PublishTestToStokenet {
    pub fn run<O: std::io::Write>(self, _: &mut O) -> Result<(), Error> {
        let network_definition = NetworkDefinition::stokenet();
        let address_encoder = AddressBech32Encoder::new(&network_definition);

        let caviarnine_package = package_address!("package_tdx_2_1p57g523zj736u370z6g4ynrytn7t6r2hledvzkhl6tzpg3urn0707e");

        let public_key =
            "02e78ec7992207d7d814173ffd8d88d04a2153481477104b8008dc424a713ddb03";
        let public_key = Secp256k1PublicKey::from_str(public_key).unwrap();
        let account =
            ComponentAddress::virtual_account_from_public_key(&public_key);

        // Publishing all of the packages and getting their addresses.
        let (
            ignition_package,
            ociswap_adapter_v1_package,
            caviarnine_adapter_v1_package,
            test_oracle_package,
            bootstrap_package,
        ) = {
            let (ignition_code, ignition_package_definition) =
                PackageLoader::get(IGNITION_PACKAGE_NAME);
            let (
                ociswap_adapter_v1_code,
                ociswap_adapter_v1_package_definition,
            ) = PackageLoader::get(OCISWAP_ADAPTER_V1_PACKAGE_NAME);
            let (
                caviarnine_adapter_v1_code,
                caviarnine_adapter_v1_package_definition,
            ) = PackageLoader::get(CAVIARNINE_ADAPTER_V1_PACKAGE_NAME);
            let (test_oracle_code, test_oracle_package_definition) =
                PackageLoader::get(TEST_ORACLE_PACKAGE_NAME);
            let (bootstrap_code, bootstrap_package_definition) =
                PackageLoader::get(BOOTSTRAP_PACKAGE_NAME);

            let ignition_package =
                publish_package(ignition_code, ignition_package_definition);
            let ociswap_adapter_v1_package = publish_package(
                ociswap_adapter_v1_code,
                ociswap_adapter_v1_package_definition,
            );
            let caviarnine_adapter_v1_package = publish_package(
                caviarnine_adapter_v1_code,
                caviarnine_adapter_v1_package_definition,
            );
            let test_oracle_package = publish_package(
                test_oracle_code,
                test_oracle_package_definition,
            );
            let bootstrap_package =
                publish_package(bootstrap_code, bootstrap_package_definition);

            (
                ignition_package,
                ociswap_adapter_v1_package,
                caviarnine_adapter_v1_package,
                test_oracle_package,
                bootstrap_package,
            )
        };

        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(
                bootstrap_package,
                "Bootstrap",
                "bootstrap_for_testing",
                manifest_args!(
                    ignition_package,
                    test_oracle_package,
                    caviarnine_package,
                    ociswap_adapter_v1_package,
                    caviarnine_adapter_v1_package,
                ),
            )
            .try_deposit_entire_worktop_or_abort(account, None)
            .build();

        let (_, events) = submit_manifest(manifest)?;
        let testing_bootstrap_information = events
            .into_iter()
            .find_map(|event| {
                if event.name == EncodedTestingBootstrapInformation::EVENT_NAME
                {
                    match event.emitter {
                        Emitter::Function {
                            blueprint_name,
                            package_address,
                        } if blueprint_name == "Bootstrap"
                            && package_address
                                == address_encoder
                                    .encode(
                                        bootstrap_package
                                            .as_node_id()
                                            .as_bytes(),
                                    )
                                    .unwrap() =>
                        {
                            let hex = serde_json::from_value::<String>(
                                event
                                    .data
                                    .get("fields")
                                    .unwrap()
                                    .get(0)
                                    .unwrap()
                                    .get("hex")
                                    .unwrap()
                                    .clone(),
                            )
                            .unwrap();

                            let encoded_bootstrap_information =
                                hex::decode(hex).unwrap();
                            scrypto_decode::<TestingBootstrapInformation>(
                                &encoded_bootstrap_information,
                            )
                            .ok()
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            })
            .expect("Should not fail");

        let serializable_testing_bootstrap_information =
            SerializableTestingBootstrapInformation::new(
                testing_bootstrap_information,
                &address_encoder,
            );
        std::fs::write(
            self.output_path,
            serde_json::to_string(&serializable_testing_bootstrap_information)
                .unwrap(),
        )?;

        Ok(())
    }
}

fn publish_package(
    wasm: Vec<u8>,
    package_definition: PackageDefinition,
) -> PackageAddress {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            wasm,
            package_definition,
            MetadataInit::default(),
            OwnerRole::None,
        )
        .build();
    (*submit_manifest(manifest).unwrap().0.first().unwrap())
        .try_into()
        .unwrap()
}

/// Constructs a transaction from a manifest, submits it through the gateway,
/// and then pools the gateway for the status of the transaction. This fails
/// if the pooling time exceeds 45 seconds.
fn submit_manifest(
    manifest: TransactionManifestV1,
) -> Result<(Vec<GlobalAddress>, Vec<Event>), crate::error::Error> {
    let gateway_client = GatewayClient::stokenet();
    let network_definition = NetworkDefinition::stokenet();
    let transaction_hash_encoder =
        TransactionHashBech32Encoder::new(&network_definition);

    let private_key = Secp256k1PrivateKey::from_u64(1).expect("must succeed!");
    let public_key = private_key.public_key();

    let current_epoch = gateway_client.get_current_epoch()?;
    let transaction = TransactionBuilder::new()
        .header(TransactionHeaderV1 {
            network_id: 0x02,
            start_epoch_inclusive: current_epoch,
            end_epoch_exclusive: current_epoch.after(100).unwrap(),
            nonce: random_nonce(),
            notary_public_key: public_key.into(),
            notary_is_signatory: false,
            tip_percentage: 0,
        })
        .manifest(manifest)
        .notarize(&private_key)
        .build();
    let intent_hash = transaction.prepare().unwrap().intent_hash();
    assert!(!(gateway_client.submit_transaction(&transaction)?));

    let bech32m_intent_hash =
        transaction_hash_encoder.encode(&intent_hash).unwrap();
    println!("Submitting transaction: {bech32m_intent_hash}");

    // Check a total of 9 times at 5 second intervals = 45 seconds.
    for _ in 0..9 {
        use crate::gateway::transaction_committed_details::*;
        match gateway_client
            .transaction_committed_details(bech32m_intent_hash.clone())
        {
            Err(..)
            | Ok(Output {
                transaction:
                    Transaction {
                        transaction_status: TransactionStatus::Unknown,
                        receipt: Receipt { .. },
                    },
            }) => {
                // Do nothing, just get out of the match statement and let the
                // thread sleep.
            }
            Ok(Output {
                transaction:
                    Transaction {
                        transaction_status,
                        receipt:
                            Receipt {
                                state_updates,
                                events,
                            },
                    },
            }) => {
                return if let TransactionStatus::CommittedSuccess =
                    transaction_status
                {
                    Ok((
                        state_updates
                            .new_global_entities
                            .into_iter()
                            .map(|item| global_address!(item.entity_address))
                            .collect(),
                        events,
                    ))
                } else {
                    Err(Error::TransactionDidNotSucceed)
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
    Err(Error::TransactionPollingTimeOut)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializableTestingBootstrapInformation {
    pub resources: BTreeMap<String, SerializableResourceInformation>,
    pub protocol: SerializableProtocolEntities,
    pub caviarnine: SerializableDexEntities,
}

impl SerializableTestingBootstrapInformation {
    pub fn new(
        value: TestingBootstrapInformation,
        encoder: &AddressBech32Encoder,
    ) -> Self {
        Self {
            resources: value
                .resources
                .into_iter()
                .map(|(k, v)| {
                    (
                        encode(k, encoder),
                        SerializableResourceInformation::from(v),
                    )
                })
                .collect(),
            protocol: SerializableProtocolEntities::new(
                value.protocol,
                encoder,
            ),
            caviarnine: SerializableDexEntities::new(value.caviarnine, encoder),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializableProtocolEntities {
    /* Ignition */
    pub ignition_package_address: String,
    pub ignition: String,
    pub protocol_resource: String,
    /* Oracle */
    pub oracle_package_address: String,
    pub oracle: String,
}

impl SerializableProtocolEntities {
    pub fn new(
        value: ProtocolEntities,
        encoder: &AddressBech32Encoder,
    ) -> Self {
        Self {
            ignition_package_address: encode(
                value.ignition_package_address,
                encoder,
            ),
            ignition: encode(value.ignition, encoder),
            protocol_resource: encode(value.protocol_resource, encoder),
            oracle_package_address: encode(
                value.oracle_package_address,
                encoder,
            ),
            oracle: encode(value.oracle, encoder),
        }
    }
}

/// A struct that defines the entities that belong to a Decentralized Exchange.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializableDexEntities {
    /* Packages */
    pub package: String,
    /* Pools */
    pub pools: BTreeMap<String, String>,
    /* Adapter */
    pub adapter_package: String,
    pub adapter: String,
}

impl SerializableDexEntities {
    pub fn new(value: DexEntities, encoder: &AddressBech32Encoder) -> Self {
        Self {
            package: encode(value.package, encoder),
            pools: value
                .pools
                .into_iter()
                .map(|(k, v)| (encode(k, encoder), encode(v, encoder)))
                .collect(),
            adapter_package: encode(value.adapter_package, encoder),
            adapter: encode(value.adapter, encoder),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SerializableResourceInformation {
    pub divisibility: u8,
    pub name: String,
    pub symbol: String,
    pub icon_url: String,
}

impl From<ResourceInformation> for SerializableResourceInformation {
    fn from(value: ResourceInformation) -> Self {
        Self {
            divisibility: value.divisibility,
            name: value.name,
            symbol: value.symbol,
            icon_url: value.icon_url,
        }
    }
}

fn encode<T>(item: T, encoder: &AddressBech32Encoder) -> String
where
    T: Into<NodeId>,
{
    let node_id = item.into();
    encoder.encode(node_id.as_bytes()).unwrap()
}
