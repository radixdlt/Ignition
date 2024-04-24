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
use crate::database_overlay::*;
use radix_engine::system::system_substates::*;
use radix_engine::transaction::*;
use radix_engine::vm::*;
use radix_engine_store_interface::db_key_mapper::*;
use scrypto_unit::*;
use state_manager::store::*;
use transaction::prelude::*;

/// A [`NetworkConnectionProvider`] that simulates the transaction execution on
/// any network so long as it can access the state manager's database. The most
/// common use of this is to simulate the transactions on mainnet prior to their
/// submission to ensure that they're all valid. The underlying database remains
/// unchanged since an overlay is used.
pub struct SimulatorNetworkConnector<'s> {
    /// The id of the network
    network_definition: NetworkDefinition,

    /// The simulator that transactions will be running against.
    ledger_simulator: TestRunner<
        NoExtension,
        UnmergeableSubstateDatabaseOverlay<'s, RocksDBStore>,
    >,
}

impl<'s> SimulatorNetworkConnector<'s> {
    pub fn new(
        database: &'s RocksDBStore,
        network_definition: NetworkDefinition,
    ) -> Self {
        let database = UnmergeableSubstateDatabaseOverlay::new(database);
        let test_runner = TestRunnerBuilder::new()
            .with_custom_database(database)
            .without_trace()
            .build_without_bootstrapping();
        Self {
            ledger_simulator: test_runner,
            network_definition,
        }
    }

    pub fn new_with_test_runner(
        ledger_simulator: TestRunner<
            NoExtension,
            UnmergeableSubstateDatabaseOverlay<'s, RocksDBStore>,
        >,
        network_definition: NetworkDefinition,
    ) -> Self {
        Self {
            ledger_simulator,
            network_definition,
        }
    }

    pub fn into_test_runner(
        self,
    ) -> TestRunner<
        NoExtension,
        UnmergeableSubstateDatabaseOverlay<'s, RocksDBStore>,
    > {
        self.ledger_simulator
    }
}

impl<'s> NetworkConnectionProvider for SimulatorNetworkConnector<'s> {
    type Error = MainnetSimulatorError;

    fn execute_transaction(
        &mut self,
        notarized_transaction: &NotarizedTransactionV1,
    ) -> Result<ExecutionReceipt, Self::Error> {
        let raw_transaction = notarized_transaction.to_raw().map_err(
            MainnetSimulatorError::NotarizedTransactionRawFormatError,
        )?;

        let transaction_receipt =
            self.ledger_simulator.execute_raw_transaction(
                &self.network_definition,
                &raw_transaction,
            );

        let execution_receipt = match transaction_receipt.result {
            TransactionResult::Commit(CommitResult {
                outcome: TransactionOutcome::Success(..),
                state_update_summary,
                ..
            }) => ExecutionReceipt::CommitSuccess(
                ExecutionReceiptSuccessContents {
                    new_entities: NewEntities {
                        new_component_addresses: state_update_summary
                            .new_components,
                        new_resource_addresses: state_update_summary
                            .new_resources,
                        new_package_addresses: state_update_summary
                            .new_packages,
                    },
                },
            ),
            TransactionResult::Commit(CommitResult {
                outcome: TransactionOutcome::Failure(reason),
                ..
            }) => ExecutionReceipt::CommitFailure {
                reason: format!("{:?}", reason),
            },
            TransactionResult::Reject(RejectResult { reason }) => {
                ExecutionReceipt::Rejection {
                    reason: format!("{:?}", reason),
                }
            }
            TransactionResult::Abort(AbortResult { reason }) => {
                ExecutionReceipt::Abort {
                    reason: format!("{:?}", reason),
                }
            }
        };
        Ok(execution_receipt)
    }

    fn preview_transaction(
        &mut self,
        preview_intent: PreviewIntentV1,
    ) -> Result<TransactionReceiptV1, Self::Error> {
        self.ledger_simulator
            .preview(preview_intent, &self.network_definition)
            .map_err(MainnetSimulatorError::PreviewError)
    }

    fn get_current_epoch(&mut self) -> Result<Epoch, Self::Error> {
        Ok(self.ledger_simulator.get_current_epoch())
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
        self.ledger_simulator
            .substate_db()
            .get_mapped::<SpreadPrefixKeyMapper, FieldSubstate<V>>(
                component_address.as_node_id(),
                MAIN_BASE_PARTITION,
                &SubstateKey::Field(ComponentField::State0.into()),
            )
            .ok_or(MainnetSimulatorError::CantReadComponentState(
                component_address,
            ))
            .map(|value| value.into_payload())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MainnetSimulatorError {
    NotarizedTransactionRawFormatError(EncodeError),
    PreviewError(PreviewError),
    CantReadComponentState(ComponentAddress),
}
