use crate::network_connection_provider::*;
use crate::publishing::*;
use state_manager::traits::*;

#[derive(Debug)]
pub enum Error {
    PrivateKeyError,
    GatewayExecutorError(PublishingError<GatewayExecutorError>),
    SimulatorExecutorError(PublishingError<MainnetSimulatorError>),
    IoError(std::io::Error),
    RocksDbOpenError(DatabaseConfigValidationError),
}

impl From<PublishingError<GatewayExecutorError>> for Error {
    fn from(value: PublishingError<GatewayExecutorError>) -> Self {
        Self::GatewayExecutorError(value)
    }
}

impl From<PublishingError<MainnetSimulatorError>> for Error {
    fn from(value: PublishingError<MainnetSimulatorError>) -> Self {
        Self::SimulatorExecutorError(value)
    }
}
