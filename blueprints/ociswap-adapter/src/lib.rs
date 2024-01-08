use adapters_interface::common::*;
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
                adapter_specific_data: AnyScryptoValue::from_typed(
                    &OciswapAdapterData {
                        k_value_when_opening_the_position: pool_k,
                        share_in_pool_when_opening_position: user_share,
                    },
                ),
            }
        }

        fn close_liquidity_position(
            &mut self,
            pool_address: ComponentAddress,
            pool_units: Bucket,
            current_oracle_price: Price,
            adapter_specific_data: AnyScryptoValue,
        ) -> CloseLiquidityPositionOutput {
            // Convert the component address into an Ociswap "BasicPool".
            let mut basic_pool =
                OciswapPoolInterfaceScryptoStub::from(pool_address);

            // Attempt to decode the adapter specific data as the data expected
            // by this adapter.
            let adapter_data = adapter_specific_data
                .as_typed::<OciswapAdapterData>()
                .expect("Failed to decode data as Ociswap Adapter data");

            // Remove the liquidity
            let buckets = basic_pool.remove_liquidity(pool_units);

            // Construct the output
            CloseLiquidityPositionOutput {
                resources: indexmap! {
                    buckets.0.resource_address() => buckets.0,
                    buckets.1.resource_address() => buckets.1,
                },
                others: vec![],
                fees: todo!(),
            }
        }

        // TODO: Does calculating the price this way differ in any way from
        // calling the method on ociswap?
        fn price(&mut self, pool_address: ComponentAddress) -> Price {
            let basic_pool =
                OciswapPoolInterfaceScryptoStub::from(pool_address);

            let pool =
                Global::<TwoResourcePool>::from(basic_pool.liquidity_pool());
            let vault_amounts = pool.get_vault_amounts();
            let mut keys = vault_amounts.keys();

            let resource_address1 = *keys.next().unwrap();
            let resource_address2 = *keys.next().unwrap();

            let value1 = vault_amounts[&resource_address1];
            let value2 = vault_amounts[&resource_address2];

            Price {
                base: resource_address1,
                quote: resource_address2,
                price: value2 / value1,
            }
        }
    }
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, ScryptoSbor,
)]
pub struct OciswapAdapterData {
    /// The value of the pool's `k` at the time when the liquidity position was
    /// opened. This is used for the calculation of the trading fees when we
    /// close the liquidity position.
    pub k_value_when_opening_the_position: PreciseDecimal,

    /// The share of the user in the pool at the time of opening the liquidity
    /// position. This is used for the calculation of the trading fees when we
    /// close the liquidity position. This is a value in the range [0, 1]
    pub share_in_pool_when_opening_position: Decimal,
}
