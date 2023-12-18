use crate::define_adapter_stubs;
use ::scrypto::prelude::*;

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
}

define_adapter_stubs! {
    name: PoolAdapter,
    functions: [
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
    ]
}
