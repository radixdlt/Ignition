use radix_engine_common::prelude::*;
use radix_engine_interface::prelude::*;
use scrypto_interface::define_interface;

define_interface! {
    PoolAdapter {
        fn open_liquidity_position(
            &mut self,
            pool_address: ComponentAddress,
            buckets: (Bucket, Bucket),
        ) -> OpenLiquidityPositionOutput;

        fn close_liquidity_position(
            &mut self,
            pool_address: ComponentAddress,
            pool_units: Bucket
        ) -> CloseLiquidityPositionOutput;
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
    /// The `k` value of the pool after the liquidity position was opened.
    pub pool_k: PreciseDecimal,
    /// The percentage ownership of the user in the pool as a result of this
    /// opened liquidity position. This is a [`Decimal`] in the range [0, 1].
    pub user_share: Decimal,
}

#[derive(Debug, ScryptoSbor)]
pub struct CloseLiquidityPositionOutput {
    /// Resources obtained from closing the liquidity position, indexed by the
    /// resource address.
    pub resources: IndexMap<ResourceAddress, Bucket>,
    /// Any additional tokens that the pool has returned back.
    pub others: Vec<Bucket>,
}
