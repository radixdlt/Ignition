use scrypto::prelude::*;

extern_blueprint_internal! {
    BasicPool,
    "BasicPool",
    "OwnedBasicPool",
    "GlobalBasicPool",
    BasicPoolFunctions {
        fn instantiate(
            a_address: ResourceAddress,
            b_address: ResourceAddress,
            input_fee_rate: Decimal,
            dapp_definition: ComponentAddress,
        ) -> Global<BasicPool>;
        fn instantiate_with_liquidity(
            a_bucket: Bucket,
            b_bucket: Bucket,
            input_fee_rate: Decimal,
            dapp_definition: ComponentAddress,
        ) -> (Global<BasicPool>, Bucket, Option<Bucket>);
    },
    {
        fn add_liquidity(&mut self, a_bucket: Bucket, b_bucket: Bucket) -> (Bucket, Option<Bucket>);
        fn remove_liquidity(&mut self, lp_token: Bucket) -> (Bucket, Bucket);
        fn swap(&mut self, input_bucket: Bucket) -> Bucket;
        fn price_sqrt(&mut self) -> Option<PreciseDecimal>;
        fn liquidity_pool(&self) -> Global<TwoResourcePool>;
        fn set_liquidity_pool_meta(
            &self,
            pool_address: ComponentAddress,
            lp_address: ResourceAddress,
            dapp_definition: ComponentAddress,
        );
        fn observation(&self, timestamp: u64) -> AccumulatedObservation;
        fn observation_intervals(&self, intervals: Vec<(u64, u64)>) -> Vec<ObservationInterval>;
        fn increase_observation_capacity(&mut self, new_capacity: u16);
        fn observations_limit(&self) -> u16;
        fn observations_stored(&self) -> u16;
        fn last_observation_index(&self) -> u16;
    }
}

#[derive(:: scrypto :: prelude :: ScryptoSbor)]
pub struct AccumulatedObservation {
    timestamp: u64,
    price_sqrt_log_acc: Decimal,
    liquidity_log_acc: Decimal,
}
#[derive(:: scrypto :: prelude :: ScryptoSbor)]
pub struct ObservationInterval {
    start: u64,
    end: u64,
    price_sqrt: Decimal,
    liquidity: Decimal,
    seconds_per_liquidity: Decimal,
}
