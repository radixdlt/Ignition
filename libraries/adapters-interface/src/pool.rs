use radix_engine_common::prelude::*;
use radix_engine_interface::prelude::*;
use scrypto_interface::define_interface;

use crate::oracle::Price;

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
            pool_units: Bucket,
            current_oracle_price: Price,
            adapter_specific_data: AnyScryptoValue
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
    /// An adapter-specific field containing information on the opening of the
    /// liquidity position. This typically contains data that is to be used
    /// later by the adapter when the position is to be closed.
    pub adapter_specific_data: AnyScryptoValue,
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

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
#[sbor(transparent)]
pub struct AnyScryptoValue((ScryptoValue,));

impl AnyScryptoValue {
    pub fn new(value: ScryptoValue) -> Self {
        Self((value,))
    }

    pub fn from_typed<T>(item: &T) -> Self
    where
        T: ScryptoEncode,
    {
        let encoded = scrypto_encode(item).expect("Must succeed!");
        let decoded =
            scrypto_decode::<ScryptoValue>(&encoded).expect("Must succeed!");
        Self::new(decoded)
    }

    pub fn as_typed<T>(&self) -> Result<T, DecodeError>
    where
        T: ScryptoDecode,
    {
        let encoded = scrypto_encode(&self.0 .0).expect("Must succeed!");
        scrypto_decode(&encoded)
    }
}

impl std::ops::Deref for AnyScryptoValue {
    type Target = ScryptoValue;

    fn deref(&self) -> &Self::Target {
        &self.0 .0
    }
}

impl std::ops::DerefMut for AnyScryptoValue {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0 .0
    }
}
