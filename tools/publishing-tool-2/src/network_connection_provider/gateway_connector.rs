// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use super::*;
use gateway_client::apis::configuration::*;
use gateway_client::apis::state_api::*;
use gateway_client::apis::status_api::GatewayStatusError;
use gateway_client::apis::status_api::*;
use gateway_client::apis::transaction_api::*;
use gateway_client::apis::Error as GatewayClientError;
use gateway_client::models::*;
use radix_common::prelude::*;
use radix_engine::transaction::*;
use radix_transactions::manifest::*;
use radix_transactions::prelude::*;

pub struct GatewayNetworkConnector {
    /// The configuration to use when making gateway HTTP requests.
    pub configuration: Configuration,
    /// The network definition of the network that the gateway talks to.
    pub network_definition: NetworkDefinition,
    /// The configuration to use when polling for the transaction status.
    pub polling_configuration: PollingConfiguration,
}

impl GatewayNetworkConnector {
    pub fn new(
        base_url: impl ToOwned<Owned = String>,
        network_definition: NetworkDefinition,
        polling_configuration: PollingConfiguration,
    ) -> Self {
        Self {
            configuration: Configuration {
                base_path: base_url.to_owned(),
                ..Default::default()
            },
            network_definition,
            polling_configuration,
        }
    }
}

impl NetworkConnectionProvider for GatewayNetworkConnector {
    type Error = GatewayExecutorError;

    fn execute_transaction(
        &mut self,
        notarized_transaction: &NotarizedTransactionV1,
    ) -> Result<SimplifiedReceipt, Self::Error> {
        let notarized_transaction_payload_bytes = notarized_transaction
            .to_payload_bytes()
            .map_err(GatewayExecutorError::NotarizedTransactionEncodeError)?;

        transaction_submit(
            &self.configuration,
            TransactionSubmitRequest {
                notarized_transaction: notarized_transaction_payload_bytes,
            },
        )
        .map_err(GatewayExecutorError::TransactionSubmissionError)?;

        let intent_hash_string = {
            let intent_hash = notarized_transaction
                .prepare()
                .map_err(
                    GatewayExecutorError::NotarizedTransactionPrepareError,
                )?
                .intent_hash();
            let transaction_hash_encoder =
                TransactionHashBech32Encoder::new(&self.network_definition);
            transaction_hash_encoder.encode(&intent_hash).map_err(
                GatewayExecutorError::TransactionHashBech32mEncoderError,
            )?
        };

        for _ in 0..self.polling_configuration.retries {
            let transaction_status_response = transaction_status(
                &self.configuration,
                TransactionStatusRequest {
                    intent_hash: intent_hash_string.clone(),
                },
            )
            .map_err(GatewayExecutorError::TransactionStatusError)?;

            match transaction_status_response.intent_status {
                // Do nothing and keep on polling.
                TransactionIntentStatus::Unknown
                | TransactionIntentStatus::CommitPendingOutcomeUnknown
                | TransactionIntentStatus::Pending => {}
                TransactionIntentStatus::CommittedSuccess => {
                    // We must wait for some time before requesting the commit
                    // details as I've observed that doing this too quickly can
                    // result in us not getting commit results back.
                    std::thread::sleep(std::time::Duration::from_secs(5));

                    let transaction_committed_result_response = transaction_committed_details(
                        &self.configuration,
                        TransactionCommittedDetailsRequest {
                            intent_hash: intent_hash_string.clone(),
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
                    )
                    .map_err(GatewayExecutorError::TransactionCommittedDetailsError)?;

                    let new_entities = {
                        let mut new_entities = NewEntities::default();

                        let bech32m_address_decoder =
                            AddressBech32Decoder::new(&self.network_definition);
                        let new_global_entities = transaction_committed_result_response
                            .transaction
                            .receipt
                            .expect("We have opted into this")
                            .state_updates
                            .expect("We have opted into this")
                            .new_global_entities
                            .into_iter()
                            .map(|Entity { entity_address, .. }| {
                                bech32m_address_decoder
                                    .validate_and_decode(&entity_address)
                                    .map_err(|_| GatewayExecutorError::AddressBech32mDecodeError)
                                    .and_then(|(_, node_id)| {
                                        node_id.try_into().map(NodeId).map_err(|_| {
                                            GatewayExecutorError::AddressBech32mDecodeError
                                        })
                                    })
                            });

                        for node_id in new_global_entities {
                            let node_id = node_id?;
                            if let Ok(package_address) =
                                PackageAddress::try_from(node_id)
                            {
                                new_entities
                                    .new_package_addresses
                                    .insert(package_address);
                            } else if let Ok(resource_address) =
                                ResourceAddress::try_from(node_id)
                            {
                                new_entities
                                    .new_resource_addresses
                                    .insert(resource_address);
                            } else if let Ok(component_address) =
                                ComponentAddress::try_from(node_id)
                            {
                                new_entities
                                    .new_component_addresses
                                    .insert(component_address);
                            }
                        }

                        new_entities
                    };

                    return Ok(SimplifiedReceipt::CommitSuccess(
                        SimplifiedReceiptSuccessContents { new_entities },
                    ));
                }
                TransactionIntentStatus::CommittedFailure => {
                    return Ok(SimplifiedReceipt::CommitFailure {
                        reason: transaction_status_response
                            .intent_status_description,
                    })
                }
                TransactionIntentStatus::PermanentlyRejected
                | TransactionIntentStatus::LikelyButNotCertainRejection => {
                    return Ok(SimplifiedReceipt::Rejection {
                        reason: transaction_status_response
                            .intent_status_description,
                    })
                }
            }

            std::thread::sleep(std::time::Duration::from_secs(
                self.polling_configuration.interval_in_seconds,
            ))
        }

        Err(GatewayExecutorError::Timeout)
    }

    fn preview_transaction(
        &mut self,
        preview_intent: PreviewIntentV1,
    ) -> Result<TransactionReceiptV1, Self::Error> {
        let string_manifest = decompile(
            &preview_intent.intent.instructions.0,
            &self.network_definition,
        )
        .map_err(GatewayExecutorError::ManifestDecompileError)?;

        let blob_hex = preview_intent
            .intent
            .blobs
            .blobs
            .iter()
            .map(|blob| hex::encode(&blob.0))
            .collect::<Vec<_>>();

        let request = TransactionPreviewRequest {
            manifest: string_manifest,
            blobs_hex: Some(blob_hex),
            start_epoch_inclusive: preview_intent
                .intent
                .header
                .start_epoch_inclusive
                .number() as i64,
            end_epoch_exclusive: preview_intent
                .intent
                .header
                .end_epoch_exclusive
                .number() as i64,
            notary_public_key: Some(Box::new(
                native_public_key_to_gateway_public_key(
                    &preview_intent.intent.header.notary_public_key,
                ),
            )),
            notary_is_signatory: Some(
                preview_intent.intent.header.notary_is_signatory,
            ),
            tip_percentage: preview_intent.intent.header.tip_percentage as i32,
            nonce: preview_intent.intent.header.nonce as i64,
            signer_public_keys: preview_intent
                .signer_public_keys
                .iter()
                .map(native_public_key_to_gateway_public_key)
                .collect(),
            flags: Box::new(TransactionPreviewRequestFlags {
                assume_all_signature_proofs: preview_intent
                    .flags
                    .assume_all_signature_proofs,
                use_free_credit: preview_intent.flags.use_free_credit,
                skip_epoch_check: preview_intent.flags.skip_epoch_check,
            }),
        };
        let response = transaction_preview(&self.configuration, request)
            .map_err(GatewayExecutorError::TransactionPreviewError)?;

        scrypto_decode::<VersionedTransactionReceipt>(&response.encoded_receipt)
            .map_err(GatewayExecutorError::TransactionReceiptDecodeError)
            .map(|receipt| receipt.fully_update_and_into_latest_version())
    }

    fn get_current_epoch(&mut self) -> Result<Epoch, Self::Error> {
        Ok(Epoch::of(
            gateway_status(&self.configuration)
                .map_err(GatewayExecutorError::GatewayStatusError)?
                .ledger_state
                .epoch as u64,
        ))
    }

    fn get_network_definition(
        &mut self,
    ) -> Result<NetworkDefinition, Self::Error> {
        Ok(self.network_definition.clone())
    }

    fn read_component_state<V: ScryptoDecode>(
        &mut self,
        component_address: ComponentAddress,
    ) -> Result<V, Self::Error> {
        let encoder = AddressBech32Encoder::new(&self.network_definition);
        let encoded_component_address = encoder
            .encode(&component_address.as_node_id().0)
            .expect("Can't fail!");

        let request = StateEntityDetailsRequest {
            at_ledger_state: None,
            opt_ins: Some(Box::new(StateEntityDetailsOptIns {
                ancestor_identities: Some(true),
                component_royalty_vault_balance: Some(true),
                package_royalty_vault_balance: Some(true),
                non_fungible_include_nfids: Some(true),
                explicit_metadata: None,
            })),
            addresses: vec![encoded_component_address],
            aggregation_level: None,
        };

        let response = state_entity_details(&self.configuration, request)
            .map_err(GatewayExecutorError::StateEntityDetailsError)?;

        let details = serde_json::from_value::<
            sbor_json::scrypto::programmatic::value::ProgrammaticScryptoValue,
        >(
            response
                .items
                .first()
                .unwrap()
                .clone()
                .details
                .unwrap()
                .get("state")
                .unwrap()
                .clone(),
        )
        .unwrap();
        let encoded_details =
            scrypto_encode(&details.to_scrypto_value()).unwrap();

        scrypto_decode(&encoded_details)
            .map_err(GatewayExecutorError::StateScryptoDecodeError)
    }
}

fn native_public_key_to_gateway_public_key(
    native_public_key: &radix_common::prelude::PublicKey,
) -> gateway_client::models::PublicKey {
    match native_public_key {
        radix_common::prelude::PublicKey::Secp256k1(public_key) => {
            gateway_client::models::PublicKey::EcdsaSecp256k1 {
                key: public_key.0,
            }
        }
        radix_common::prelude::PublicKey::Ed25519(public_key) => {
            gateway_client::models::PublicKey::EddsaEd25519 {
                key: public_key.0,
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PollingConfiguration {
    pub interval_in_seconds: u64,
    pub retries: u64,
}

#[derive(Debug)]
pub enum GatewayExecutorError {
    ManifestDecompileError(DecompileError),
    TransactionReceiptDecodeError(DecodeError),
    NotarizedTransactionEncodeError(EncodeError),
    NotarizedTransactionPrepareError(PrepareError),
    TransactionHashBech32mEncoderError(TransactionHashBech32EncodeError),
    GatewayStatusError(GatewayClientError<GatewayStatusError>),
    TransactionStatusError(GatewayClientError<TransactionStatusError>),
    TransactionPreviewError(GatewayClientError<TransactionPreviewError>),
    StateEntityDetailsError(GatewayClientError<StateEntityDetailsError>),
    TransactionCommittedDetailsError(
        GatewayClientError<TransactionCommittedDetailsError>,
    ),
    TransactionSubmissionError(GatewayClientError<TransactionSubmitError>),
    StateScryptoDecodeError(DecodeError),
    AddressBech32mDecodeError,
    Timeout,
}
