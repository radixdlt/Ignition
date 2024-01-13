#![allow(clippy::enum_variant_names)]

#[derive(Debug)]
pub enum Error {
    ReqwestError(reqwest::Error),
    TransactionPollingTimeOut,
    TransactionDidNotSucceed,
    IoError(std::io::Error),
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::ReqwestError(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}
