use scrypto::prelude::*;

/// An enum that describes the volatility of an asset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub enum Volatility {
    Volatile,
    NonVolatile,
}
