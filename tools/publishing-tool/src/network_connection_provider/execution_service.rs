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

use itertools::*;
use radix_common::prelude::*;
use radix_engine::transaction::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::prelude::*;
use radix_transactions::manifest::*;
use radix_transactions::model::*;
use radix_transactions::prelude::*;

use super::*;

/// A simple execution service whose main responsibilities is to construct,
/// submit, and return the result of transactions.
pub struct ExecutionService<'e, E: NetworkConnectionProvider> {
    /// The executor that the service will use to execute transactions.
    executor: &'e mut E,
    /// The account to use for the payment of fees.
    fee_payer_account_address: ComponentAddress,
    /// The notary of the transaction
    notary_private_key: &'e PrivateKey,
    /// The set of private keys that should sign the transaction.
    signers_private_keys: &'e [&'e PrivateKey],
}

impl<'e, E: NetworkConnectionProvider> ExecutionService<'e, E> {
    pub fn new(
        executor: &'e mut E,
        fee_payer_account_address: ComponentAddress,
        notary_private_key: &'e PrivateKey,
        signers_private_keys: &'e [&'e PrivateKey],
    ) -> Self {
        Self {
            executor,
            fee_payer_account_address,
            notary_private_key,
            signers_private_keys,
        }
    }

    pub fn execute_manifest(
        &mut self,
        mut manifest: TransactionManifestV1,
    ) -> Result<
        SimplifiedReceiptSuccessContents,
        ExecutionServiceError<<E as NetworkConnectionProvider>::Error>,
    > {
        // If the manifest is empty (has no instructions) do no work
        if manifest.instructions.is_empty() {
            return Ok(SimplifiedReceiptSuccessContents {
                new_entities: Default::default(),
            });
        }

        // The signers for the transaction
        let notary_is_signatory =
            self.signers_private_keys.iter().any(|private_key| {
                private_key.public_key() == self.notary_private_key.public_key()
            });
        let signer_private_keys = self
            .signers_private_keys
            .iter()
            .filter(|private_key| {
                private_key.public_key() != self.notary_private_key.public_key()
            })
            .unique_by(|private_key| private_key.public_key());

        // Getting the current network definition
        let network_definition = self
            .executor
            .get_network_definition()
            .map_err(ExecutionServiceError::ExecutorError)?;

        // Constructing the header
        let current_epoch = self
            .executor
            .get_current_epoch()
            .map_err(ExecutionServiceError::ExecutorError)?;
        let header = TransactionHeaderV1 {
            network_id: network_definition.id,
            start_epoch_inclusive: current_epoch,
            end_epoch_exclusive: current_epoch
                .after(10)
                .expect("Not currently an issue"),
            nonce: rand::random(),
            notary_public_key: self.notary_private_key.public_key(),
            notary_is_signatory,
            tip_percentage: 0,
        };

        // Getting a preview of the transaction to determine the fees.
        let preview_receipt = self
            .executor
            .preview_transaction(PreviewIntentV1 {
                intent: IntentV1 {
                    header: header.clone(),
                    instructions: InstructionsV1(manifest.instructions.clone()),
                    blobs: BlobsV1 {
                        blobs: manifest
                            .blobs
                            .clone()
                            .into_values()
                            .map(BlobV1)
                            .collect(),
                    },
                    message: MessageV1::None,
                },
                signer_public_keys: self
                    .signers_private_keys
                    .iter()
                    .map(|private_key| private_key.public_key())
                    .unique()
                    .collect(),
                flags: PreviewFlags {
                    use_free_credit: true,
                    assume_all_signature_proofs: false,
                    skip_epoch_check: false,
                    disable_auth: false,
                },
            })
            .map_err(ExecutionServiceError::ExecutorError)?;

        if !preview_receipt.is_commit_success() {
            return Err(
                ExecutionServiceError::TransactionPreviewWasNotSuccessful(
                    manifest.clone(),
                    preview_receipt,
                ),
            );
        }
        let total_fees = preview_receipt.fee_summary.total_cost();
        let total_fees_plus_padding =
            total_fees + self.signers_private_keys.len() * dec!(0.5);
        let total_fees_plus_padding = total_fees_plus_padding * dec!(1.10);

        // Adding a lock fee instruction to the manifest.
        manifest.instructions.insert(
            0,
            InstructionV1::CallMethod {
                address: self.fee_payer_account_address.into(),
                method_name: ACCOUNT_LOCK_FEE_IDENT.to_string(),
                args: to_manifest_value(&AccountLockFeeInput {
                    amount: total_fees_plus_padding,
                })
                .expect("Can't fail!"),
            },
        );

        // Constructing the transaction.
        let mut transaction_builder = TransactionBuilder::new()
            .header(header)
            .manifest(manifest.clone());
        for signer_private_key in signer_private_keys {
            transaction_builder = transaction_builder.sign(*signer_private_key)
        }
        let transaction = transaction_builder
            .notarize(self.notary_private_key)
            .build();

        // Submitting the transaction
        let receipt = self
            .executor
            .execute_transaction(&transaction)
            .map_err(ExecutionServiceError::ExecutorError)?;

        // Do a match on the receipt and error out if execution failed. If it
        // did not, then return the success contents.
        match receipt {
            SimplifiedReceipt::CommitSuccess(success_contents) => {
                Ok(success_contents)
            }
            SimplifiedReceipt::CommitFailure { reason }
            | SimplifiedReceipt::Rejection { reason }
            | SimplifiedReceipt::Abort { reason } => {
                let decompiled_manifest =
                    decompile(&manifest.instructions, &network_definition)
                        .map_err(
                            ExecutionServiceError::ManifestDecompilationFailed,
                        )?;
                Err(
                    ExecutionServiceError::TransactionExecutionWasNotSuccessful {
                        manifest: decompiled_manifest,
                        reason,
                    },
                )
            }
        }
    }

    pub fn with_network_connection_provider<F, O>(&mut self, callback: F) -> O
    where
        F: Fn(&mut E) -> O,
    {
        callback(self.executor)
    }
}

#[derive(Debug)]
pub enum ExecutionServiceError<E> {
    ExecutorError(E),
    ManifestDecompilationFailed(DecompileError),
    TransactionExecutionWasNotSuccessful {
        manifest: String,
        reason: String,
    },
    TransactionPreviewWasNotSuccessful(
        TransactionManifestV1,
        TransactionReceipt,
    ),
}
