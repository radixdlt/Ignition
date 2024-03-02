use radix_engine::transaction::TransactionReceiptV1;
use transaction::prelude::*;

/// A trait that can be implemented by various structs to execute transactions
/// and produce execution receipts. The executor could be object doing the
/// execution itself in case of a node or could delegate the execution to
/// another object like in the case of the gateway. This detail does not matter
/// for the executor.
pub trait Executor {
    type Error: Debug;

    fn execute_transaction(
        &mut self,
        notarized_transaction: &NotarizedTransactionV1,
    ) -> Result<ExecutionReceipt, Self::Error>;

    fn preview_transaction(
        &mut self,
        preview_intent: PreviewIntentV1,
    ) -> Result<TransactionReceiptV1, Self::Error>;

    fn get_current_epoch(&mut self) -> Result<Epoch, Self::Error>;

    fn get_network_definition(
        &mut self,
    ) -> Result<NetworkDefinition, Self::Error>;
}

/// A simplified transaction receipt containing the key pieces of information
/// that must be included in an execution receipt. This is limited by the data
/// that the node can give us.
#[derive(Clone, Debug, PartialEq, Eq, ScryptoSbor)]
pub enum ExecutionReceipt {
    CommitSuccess { new_entities: NewEntities },
    CommitFailure { reason: String },
    Rejection { reason: String },
    Abort { reason: String },
}

#[derive(Clone, Default, Debug, PartialEq, Eq, ScryptoSbor)]
pub struct NewEntities {
    pub new_component_addresses: IndexSet<ComponentAddress>,
    pub new_resource_addresses: IndexSet<ResourceAddress>,
    pub new_package_addresses: IndexSet<PackageAddress>,
}
