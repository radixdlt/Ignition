use super::*;
use crate::database_overlay::*;
use radix_engine::transaction::*;
use radix_engine::vm::*;
use scrypto_unit::*;
use state_manager::store::*;
use transaction::prelude::*;

/// An [`Executor`] that simulates the transaction execution on mainnet state.
/// This requires having a mainnet database that the executor can read from. All
/// of the database changes from the transaction execution is written to an
/// overlay which means that the mainnet database's state remains unchanged.
pub struct MainnetSimulatorExecutor<'s>(
    TestRunner<
        NoExtension,
        UnmergeableSubstateDatabaseOverlay<'s, RocksDBStore>,
    >,
);

impl<'s> MainnetSimulatorExecutor<'s> {
    pub fn new(database: &'s RocksDBStore) -> Self {
        let database = UnmergeableSubstateDatabaseOverlay::new(database);
        let test_runner = TestRunnerBuilder::new()
            .with_custom_database(database)
            .without_trace()
            .build();
        Self(test_runner)
    }
}

impl<'s> Executor for MainnetSimulatorExecutor<'s> {
    type Error = MainnetSimulatorError;

    fn execute_transaction(
        &mut self,
        notarized_transaction: &NotarizedTransactionV1,
    ) -> Result<ExecutionReceipt, Self::Error> {
        let network_definition = NetworkDefinition::mainnet();
        let raw_transaction = notarized_transaction.to_raw().map_err(
            MainnetSimulatorError::NotarizedTransactionRawFormatError,
        )?;

        let transaction_receipt = self
            .0
            .execute_raw_transaction(&network_definition, &raw_transaction);

        let execution_receipt = match transaction_receipt.result {
            TransactionResult::Commit(CommitResult {
                outcome: TransactionOutcome::Success(..),
                state_update_summary,
                ..
            }) => ExecutionReceipt::CommitSuccess {
                new_entities: NewEntities {
                    new_component_addresses: state_update_summary
                        .new_components,
                    new_resource_addresses: state_update_summary.new_resources,
                    new_package_addresses: state_update_summary.new_packages,
                },
            },
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
        let network_definition = NetworkDefinition::mainnet();
        self.0
            .preview(preview_intent, &network_definition)
            .map_err(MainnetSimulatorError::PreviewError)
    }

    fn get_current_epoch(&mut self) -> Result<Epoch, Self::Error> {
        Ok(self.0.get_current_epoch())
    }

    fn get_network_definition(
        &mut self,
    ) -> Result<NetworkDefinition, Self::Error> {
        Ok(NetworkDefinition::mainnet())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MainnetSimulatorError {
    NotarizedTransactionRawFormatError(EncodeError),
    PreviewError(PreviewError),
}
