use crate::*;

test_bindings! {
    BasicPool,
    OciswapPool,
    BasicPoolFunctions {
        fn instantiate(
            a_address: ResourceAddress,
            b_address: ResourceAddress,
            input_fee_rate: Decimal,
            dapp_definition: ComponentAddress,
        ) -> OciswapPool;
        fn instantiate_with_liquidity(
            a_bucket: Bucket,
            b_bucket: Bucket,
            input_fee_rate: Decimal,
            dapp_definition: ComponentAddress,
        ) -> (OciswapPool, Bucket, Option<Bucket>);
    },
    {
        fn add_liquidity(&mut self, a_bucket: Bucket, b_bucket: Bucket) -> (Bucket, Option<Bucket>);
        fn remove_liquidity(&mut self, lp_token: Bucket) -> (Bucket, Bucket);
        fn swap(&mut self, input_bucket: Bucket) -> Bucket;
        fn price_sqrt(&mut self) -> Option<PreciseDecimal>;
        fn increase_observation_capacity(&mut self, new_capacity: u16);
        fn observations_limit(&self) -> u16;
        fn observations_stored(&self) -> u16;
        fn last_observation_index(&self) -> u16;
    }
}
