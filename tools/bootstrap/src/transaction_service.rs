use crate::error::*;
use gateway_client::apis::configuration::*;
use gateway_client::apis::status_api::*;
use gateway_client::apis::transaction_api::*;
use gateway_client::models::*;
use radix_engine::transaction::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::prelude::*;
use std::thread::*;
use std::time::*;
use transaction::manifest::*;
use transaction::prelude::*;

type NativePublicKey = radix_engine_interface::crypto::PublicKey;
type GatewayPublicKey = gateway_client::models::PublicKey;

/// A transaction service that provides a higher-level abstraction over the
/// gateway API.
pub struct TransactionService<'a> {
    /// The Bech32m encoders and decoders that the transaction service uses.
    bech32m_coders: &'a Bech32mCoders<'a>,

    /// The base url of the gateway API.
    gateway_api_base_url: String,

    /// Controls how often the transaction service should poll for the
    /// transaction status. This defaults to 5 seconds which is 5,000
    /// milliseconds.
    polling_frequency_in_milliseconds: u64,

    /// Controls how many polling attempts the transaction service should make
    /// before considering that to be an error. This defaults to 12 attempts.
    maximum_number_of_polling_attempts: u64,
}

impl<'a> TransactionService<'a> {
    pub fn new(
        bech32m_coders: &'a Bech32mCoders,
        gateway_api_base_url: impl Into<String>,
    ) -> Self {
        Self::new_configurable(bech32m_coders, gateway_api_base_url, 5_000, 12)
    }

    pub fn new_configurable(
        bech32m_coders: &'a Bech32mCoders,
        gateway_api_base_url: impl Into<String>,
        polling_frequency_in_milliseconds: u64,
        maximum_number_of_polling_attempts: u64,
    ) -> Self {
        Self {
            bech32m_coders,
            gateway_api_base_url: gateway_api_base_url.into(),
            polling_frequency_in_milliseconds,
            maximum_number_of_polling_attempts,
        }
    }

    pub fn submit_manifest(
        &self,
        mut manifest: TransactionManifestV1,
        notary_private_key: &PrivateKey,
        fee_handling: &FeeHandling<'_>,
    ) -> std::result::Result<SimplifiedTransactionReceipt, Error> {
        // Generating the nonce that will be used in submitting the transaction.
        let nonce = rand::random::<u32>();

        // Getting the epoch bounds of this transaction
        let current_epoch = self.current_epoch()?;
        let max_epoch = current_epoch.after(10).unwrap();

        let additional_signatures = if let FeeHandling::EstimateAndLock {
            fee_payer_private_key,
            ..
        } = fee_handling
        {
            let is_additional_fee_payer_signature_required =
                match (notary_private_key, fee_payer_private_key) {
                    (
                        PrivateKey::Secp256k1(notary),
                        PrivateKey::Secp256k1(fee_payer),
                    ) => notary.to_bytes() != fee_payer.to_bytes(),
                    (
                        PrivateKey::Ed25519(notary),
                        PrivateKey::Ed25519(fee_payer),
                    ) => notary.to_bytes() != fee_payer.to_bytes(),
                    (PrivateKey::Secp256k1(..), PrivateKey::Ed25519(..))
                    | (PrivateKey::Ed25519(..), PrivateKey::Secp256k1(..)) => {
                        true
                    }
                };

            if is_additional_fee_payer_signature_required {
                vec![fee_payer_private_key]
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        // If we need to estimate the fees then we must get a preview of the
        // manifest to estimate how much the fees will be.
        if let FeeHandling::EstimateAndLock {
            fee_payer_account,
            fee_payer_private_key,
        } = fee_handling
        {
            let decompiled_manifest = decompile(
                &manifest.instructions,
                self.bech32m_coders.network_definition,
            )?;

            let fees = {
                // Getting a preview of the manifest.
                let preview_response = transaction_preview(
                    &self.gateway_config(),
                    TransactionPreviewRequest {
                        manifest: decompiled_manifest.clone(),
                        blobs_hex: Some(
                            manifest.blobs.values().map(hex::encode).collect(),
                        ),
                        start_epoch_inclusive: current_epoch.number() as i64,
                        end_epoch_exclusive: max_epoch.number() as i64,
                        notary_public_key: match notary_private_key.public_key()
                        {
                            NativePublicKey::Secp256k1(pk) => Some(Box::new(
                                GatewayPublicKey::EcdsaSecp256k1 { key: pk.0 },
                            )),
                            NativePublicKey::Ed25519(pk) => {
                                Some(Box::new(GatewayPublicKey::EddsaEd25519 {
                                    key: pk.0,
                                }))
                            }
                        },
                        notary_is_signatory: Some(true),
                        tip_percentage: 0,
                        nonce: nonce as i64,
                        signer_public_keys: vec![match fee_payer_private_key {
                            PrivateKey::Secp256k1(pk) => {
                                GatewayPublicKey::EcdsaSecp256k1 {
                                    key: pk.public_key().0,
                                }
                            }
                            PrivateKey::Ed25519(pk) => {
                                GatewayPublicKey::EddsaEd25519 {
                                    key: pk.public_key().0,
                                }
                            }
                        }],
                        flags: Box::new(TransactionPreviewRequestFlags {
                            use_free_credit: true,
                            assume_all_signature_proofs: true,
                            skip_epoch_check: false,
                        }),
                    },
                )?;

                // Ensure that the transaction succeeded in preview. Getting the
                // fees of a transaction that failed or was rejected has no
                // point.
                let receipt = scrypto_decode::<VersionedTransactionReceipt>(
                    &preview_response.encoded_receipt,
                )
                .unwrap()
                .into_latest();

                if !receipt.is_commit_success() {
                    return Err(Error::PreviewFailed {
                        manifest: decompiled_manifest,
                        receipt,
                    });
                }

                receipt.fee_summary.total_execution_cost_in_xrd
                    + receipt.fee_summary.total_finalization_cost_in_xrd
                    + receipt.fee_summary.total_tipping_cost_in_xrd
                    + receipt.fee_summary.total_storage_cost_in_xrd
                    + receipt.fee_summary.total_royalty_cost_in_xrd
            };

            // Adding a 50% padding over the fees that were calculated.
            let fees_to_lock = fees * dec!(1.5);

            // Adding the instruction to lock fees.
            manifest.instructions.insert(
                0,
                InstructionV1::CallMethod {
                    address: (*fee_payer_account).into(),
                    method_name: ACCOUNT_LOCK_FEE_IDENT.to_owned(),
                    args: manifest_args!(fees_to_lock).into(),
                },
            );
        };

        // Constructing the transaction and submitting it.
        let mut builder = TransactionBuilder::new().manifest(manifest).header(
            TransactionHeaderV1 {
                network_id: self.bech32m_coders.network_definition.id,
                start_epoch_inclusive: current_epoch,
                end_epoch_exclusive: max_epoch,
                nonce,
                notary_public_key: notary_private_key.public_key(),
                notary_is_signatory: true,
                tip_percentage: 0,
            },
        );
        for key in additional_signatures {
            builder = builder.sign(*key);
        }
        let notarized_transaction =
            builder.notarize(notary_private_key).build();

        // Compiling the notarized transaction and submitting it to the gateway.
        let compiled_notarized_transaction =
            notarized_transaction.to_payload_bytes().unwrap();
        transaction_submit(
            &self.gateway_config(),
            TransactionSubmitRequest {
                notarized_transaction: compiled_notarized_transaction,
            },
        )?;

        // Getting the intent hash and starting to poll for the transaction.
        let intent_hash =
            notarized_transaction.prepare().unwrap().intent_hash();
        let bech32m_intent_hash = self
            .bech32m_coders
            .transaction_hash_encoder
            .encode(&intent_hash)
            .unwrap();
        println!("{bech32m_intent_hash}");

        for _ in 0..self.maximum_number_of_polling_attempts {
            match transaction_status(
                &self.gateway_config(),
                TransactionStatusRequest {
                    intent_hash: bech32m_intent_hash.clone(),
                },
            ) {
                Ok(TransactionStatusResponse {
                    status: TransactionStatus::CommittedSuccess,
                    ..
                }) => {
                    // The transaction has been committed successfully. We can
                    // now get the transaction committed details with no issues.
                    let committed_details = transaction_committed_details(
                        &self.gateway_config(),
                        TransactionCommittedDetailsRequest {
                            intent_hash: bech32m_intent_hash.clone(),
                            opt_ins: Some(Box::new(TransactionDetailsOptIns {
                                raw_hex: Some(true),
                                receipt_state_changes: Some(true),
                                receipt_fee_summary: Some(true),
                                receipt_fee_source: Some(true),
                                receipt_fee_destination: Some(true),
                                receipt_costing_parameters: Some(true),
                                receipt_events: Some(true),
                                receipt_output: Some(true),
                                affected_global_entities: Some(true),
                                balance_changes: Some(true),
                            })),
                            at_ledger_state: None,
                        },
                    )?;

                    let state_updates = committed_details
                        .transaction
                        .receipt
                        .unwrap()
                        .state_updates
                        .unwrap();

                    let mut simplified_receipt = SimplifiedTransactionReceipt {
                        new_component_addresses: Default::default(),
                        new_resource_addresses: Default::default(),
                        new_package_addresses: Default::default(),
                    };

                    for entity in state_updates.new_global_entities {
                        let address_string = entity.entity_address;

                        if let Some(address) = PackageAddress::try_from_bech32(
                            &self.bech32m_coders.address_decoder,
                            &address_string,
                        ) {
                            simplified_receipt
                                .new_package_addresses
                                .push(address)
                        } else if let Some(address) =
                            ResourceAddress::try_from_bech32(
                                &self.bech32m_coders.address_decoder,
                                &address_string,
                            )
                        {
                            simplified_receipt
                                .new_resource_addresses
                                .push(address)
                        } else if let Some(address) =
                            ComponentAddress::try_from_bech32(
                                &self.bech32m_coders.address_decoder,
                                &address_string,
                            )
                        {
                            simplified_receipt
                                .new_component_addresses
                                .push(address)
                        }
                    }

                    return Ok(simplified_receipt);
                }
                Ok(TransactionStatusResponse {
                    status:
                        TransactionStatus::CommittedFailure
                        | TransactionStatus::Rejected,
                    ..
                }) => {
                    return Err(Error::TransactionWasNotSuccessful {
                        intent_hash: bech32m_intent_hash,
                    })
                }
                _ => {}
            }
            sleep(Duration::from_millis(
                self.polling_frequency_in_milliseconds,
            ));
        }

        Err(Error::TransactionPollingYieldedNothing {
            intent_hash: bech32m_intent_hash,
        })
    }

    fn gateway_config(&self) -> Configuration {
        Configuration {
            base_path: self.gateway_api_base_url.clone(),
            ..Default::default()
        }
    }

    fn current_epoch(&self) -> std::result::Result<Epoch, Error> {
        Ok(Epoch::of(
            gateway_status(&self.gateway_config())?.ledger_state.epoch as u64,
        ))
    }
}

pub struct SimplifiedTransactionReceipt {
    pub new_component_addresses: Vec<ComponentAddress>,
    pub new_resource_addresses: Vec<ResourceAddress>,
    pub new_package_addresses: Vec<PackageAddress>,
}

pub enum FeeHandling<'a> {
    AlreadyHandled,
    EstimateAndLock {
        fee_payer_account: ComponentAddress,
        fee_payer_private_key: &'a PrivateKey,
    },
}

pub struct Bech32mCoders<'a> {
    pub network_definition: &'a NetworkDefinition,
    pub address_encoder: AddressBech32Encoder,
    pub address_decoder: AddressBech32Decoder,
    pub transaction_hash_encoder: TransactionHashBech32Encoder,
    pub transaction_hash_decoder: TransactionHashBech32Decoder,
}

impl<'a> Bech32mCoders<'a> {
    pub fn from_network_definition(
        network_definition: &'a NetworkDefinition,
    ) -> Self {
        Self {
            network_definition,
            address_encoder: AddressBech32Encoder::new(network_definition),
            address_decoder: AddressBech32Decoder::new(network_definition),
            transaction_hash_encoder: TransactionHashBech32Encoder::new(
                network_definition,
            ),
            transaction_hash_decoder: TransactionHashBech32Decoder::new(
                network_definition,
            ),
        }
    }
}
