use radix_engine::transaction::*;
use transaction::manifest::*;

type TransactionPreviewError = gateway_client::apis::Error<
    gateway_client::apis::transaction_api::TransactionPreviewError,
>;
type TransactionCommittedDetailsError = gateway_client::apis::Error<
    gateway_client::apis::transaction_api::TransactionCommittedDetailsError,
>;
type TransactionSubmitError = gateway_client::apis::Error<
    gateway_client::apis::transaction_api::TransactionSubmitError,
>;
type GatewayStatusError = gateway_client::apis::Error<
    gateway_client::apis::status_api::GatewayStatusError,
>;

#[derive(Debug)]
pub enum Error {
    ManifestDecompilation(DecompileError),
    TransactionPreviewError(TransactionPreviewError),
    TransactionSubmitError(TransactionSubmitError),
    TransactionCommittedDetailsError(TransactionCommittedDetailsError),
    GatewayStatusError(GatewayStatusError),
    PreviewFailed {
        manifest: String,
        receipt: TransactionReceiptV1,
    },
    TransactionPollingYieldedNothing {
        intent_hash: String,
    },
    TransactionWasNotSuccessful {
        intent_hash: String,
    },
}

impl From<DecompileError> for Error {
    fn from(value: DecompileError) -> Self {
        Self::ManifestDecompilation(value)
    }
}

impl From<TransactionPreviewError> for Error {
    fn from(value: TransactionPreviewError) -> Self {
        Self::TransactionPreviewError(value)
    }
}

impl From<TransactionSubmitError> for Error {
    fn from(value: TransactionSubmitError) -> Self {
        Self::TransactionSubmitError(value)
    }
}

impl From<TransactionCommittedDetailsError> for Error {
    fn from(value: TransactionCommittedDetailsError) -> Self {
        Self::TransactionCommittedDetailsError(value)
    }
}

impl From<GatewayStatusError> for Error {
    fn from(value: GatewayStatusError) -> Self {
        Self::GatewayStatusError(value)
    }
}
