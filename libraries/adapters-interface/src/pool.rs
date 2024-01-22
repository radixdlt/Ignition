//! Defines the interface of the adapters used to communicate with pools.

use crate::prelude::*;
use scrypto::prelude::*;
use scrypto_interface::*;

define_interface! {
    PoolAdapter impl [
        #[cfg(feature = "trait")]
        Trait,
        #[cfg(feature = "scrypto")]
        ScryptoStub,
        #[cfg(feature = "scrypto-test")]
        ScryptoTestStub,
        #[cfg(feature = "manifest-builder")]
        ManifestBuilderStub
    ] {
        /// Opens a liquidity position in the pool.
        ///
        /// This method opens a liquidity position, or adds liquidity, to a
        /// two-resource liquidity pool and returns the pool units, change, and
        /// other resources returned to it that are neither change nor pool
        /// units.
        ///
        /// There is no assumption on what kind of pool units are returned. They
        /// can be the pool units from the native pools, custom pool units, or
        /// even NFTs.
        fn open_liquidity_position(
            &mut self,
            pool_address: ComponentAddress,
            #[manifest_type = "(ManifestBucket, ManifestBucket)"]
            buckets: (Bucket, Bucket),
        ) -> OpenLiquidityPositionOutput;

        /// Closes a liquidity position on the passed pool.
        ///
        /// This method closes a liquidity position, or removes liquidity, from
        /// the pool returning the share of the user in the pool as well as the
        /// estimated fees.
        fn close_liquidity_position(
            &mut self,
            pool_address: ComponentAddress,
            #[manifest_type = "ManifestBucket"]
            pool_units: Bucket,
        ) -> CloseLiquidityPositionOutput;

        /// Returns the price of the pair of assets in the pool.
        fn price(&mut self, pool_address: ComponentAddress) -> Price;

        /// The addresses of the pool's resources.
        fn resource_addresses(
            &mut self,
            pool_address: ComponentAddress
        ) -> (ResourceAddress, ResourceAddress);

        /// Returns information specific to each exchange to be included in the
        /// non-fungible data of the liquidity receipt.
        ///
        /// Each exchange has control over a specific but small set of fields
        /// in the liquidity receipt that they are allowed to set. This method
        /// is called by the protocol to get the values that the exchanges want
        /// to set to these fields when opening a liquidity position.
        fn exchange_specific_liquidity_receipt_data(
            &mut self
        ) -> LiquidityReceiptExchangeSpecificData;
    }
}

#[derive(Debug, ScryptoSbor)]
pub struct OpenLiquidityPositionOutput {
    /// The pool units obtained as part of the contribution to the pool.
    pub pool_units: Bucket,
    /// Any change the pool has returned back indexed by the resource address.
    pub change: IndexMap<ResourceAddress, Bucket>,
    /// Any additional tokens that the pool has returned back.
    pub others: Vec<Bucket>,
}

#[derive(Debug, ScryptoSbor)]
pub struct CloseLiquidityPositionOutput {
    /// Resources obtained from closing the liquidity position, indexed by the
    /// resource address.
    pub resources: IndexMap<ResourceAddress, Bucket>,
    /// Any additional tokens that the pool has returned back.
    pub others: Vec<Bucket>,
    /// The amount of trading fees earned on the position.
    pub fees: IndexMap<ResourceAddress, Decimal>,
}
