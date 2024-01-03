use adapters_interface::pool::*;
use scrypto::prelude::*;
use scrypto_interface::*;

define_interface! {
    BasicPool as OciswapPool {
        fn instantiate(
            a_address: ResourceAddress,
            b_address: ResourceAddress,
            input_fee_rate: Decimal,
            dapp_definition: ComponentAddress,
        ) -> Self;
        fn instantiate_with_liquidity(
            a_bucket: Bucket,
            b_bucket: Bucket,
            input_fee_rate: Decimal,
            dapp_definition: ComponentAddress,
        ) -> (Self, Bucket, Option<Bucket>);
        fn add_liquidity(
            &mut self,
            a_bucket: Bucket,
            b_bucket: Bucket
        ) -> (Bucket, Option<Bucket>);
        fn remove_liquidity(&mut self, lp_token: Bucket) -> (Bucket, Bucket);
        fn swap(&mut self, input_bucket: Bucket) -> Bucket;
        fn price_sqrt(&mut self) -> Option<PreciseDecimal>;
        fn liquidity_pool(&self) -> ComponentAddress;
        fn set_liquidity_pool_meta(
            &self,
            pool_address: ComponentAddress,
            lp_address: ResourceAddress,
            dapp_definition: ComponentAddress,
        );
        fn increase_observation_capacity(&mut self, new_capacity: u16);
    }
}

#[blueprint_with_traits]
mod adapter {
    struct OciswapAdapter;

    impl OciswapAdapter {
        pub fn instantiate(
            owner_role: OwnerRole,
            address_reservation: Option<GlobalAddressReservation>,
        ) -> Global<OciswapAdapter> {
            let address_reservation = address_reservation.unwrap_or(
                Runtime::allocate_component_address(BlueprintId {
                    package_address: Runtime::package_address(),
                    blueprint_name: Runtime::blueprint_name(),
                })
                .0,
            );

            Self {}
                .instantiate()
                .prepare_to_globalize(owner_role)
                .with_address(address_reservation)
                .globalize()
        }
    }

    impl PoolAdapterInterfaceTrait for OciswapAdapter {
        fn open_liquidity_position(
            &mut self,
            pool_address: ComponentAddress,
            buckets: (Bucket, Bucket),
        ) -> OpenLiquidityPositionOutput {
            // Convert the component address into an Ociswap "BasicPool".
            let mut basic_pool =
                OciswapPoolInterfaceScryptoStub::from(pool_address);

            // Add liquidity to the pool.
            // TODO: Is this actually pool units and change?
            let (pool_units, change) =
                basic_pool.add_liquidity(buckets.0, buckets.1);

            // Calculate the `k` after the contribution was made to the pool.
            // We do this by getting the TwoResourcePool of this pool, getting
            // the reserves from there, and multiplying them.
            let pool_k =
                Global::<TwoResourcePool>::from(basic_pool.liquidity_pool())
                    .get_vault_amounts()
                    .values()
                    .fold(PreciseDecimal::ONE, |acc, other| acc * *other);

            // Getting the share of the user in the pool. We do this by dividing
            // the amount of pool units we got back by the total supply of all
            // pool units.
            let user_share = pool_units.amount()
                / pool_units
                    .resource_manager()
                    .total_supply()
                    .expect("Pool units have total supply enabled");

            // Construct the output
            OpenLiquidityPositionOutput {
                pool_units,
                change: change
                    .map(|bucket| {
                        indexmap! {
                            bucket.resource_address() => bucket
                        }
                    })
                    .unwrap_or_default(),
                others: vec![],
                pool_k,
                user_share,
            }
        }

        fn close_liquidity_position(
            &mut self,
            pool_address: ComponentAddress,
            pool_units: Bucket,
        ) -> CloseLiquidityPositionOutput {
            // Convert the component address into an Ociswap "BasicPool".
            let mut basic_pool =
                OciswapPoolInterfaceScryptoStub::from(pool_address);

            // Remove the liquidity
            let buckets = basic_pool.remove_liquidity(pool_units);

            // Construct the output
            CloseLiquidityPositionOutput {
                resources: indexmap! {
                    buckets.0.resource_address() => buckets.0,
                    buckets.1.resource_address() => buckets.1,
                },
                others: vec![],
            }
        }
    }
}
