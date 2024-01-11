use adapters_interface::prelude::*;
use scrypto::prelude::*;
use scrypto_interface::*;

define_interface! {
    BasicPool as OciswapPool impl [ScryptoStub, ScryptoTestStub] {
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

define_interface! {
    TwoResourcePool impl [ScryptoStub, ScryptoTestStub] {
        fn instantiate(
            owner_role: OwnerRole,
            pool_manager_rule: AccessRule,
            resource_addresses: (ResourceAddress, ResourceAddress),
            address_reservation: Option<GlobalAddressReservation>,
        ) -> Self;
        fn contribute(&mut self, buckets: (Bucket, Bucket)) -> (Bucket, Option<Bucket>);
        fn redeem(&mut self, bucket: Bucket) -> (Bucket, Bucket);
        fn protected_deposit(&mut self, bucket: Bucket);
        fn protected_withdraw(
            &mut self,
            resource_address: ResourceAddress,
            amount: Decimal,
            withdraw_strategy: WithdrawStrategy,
        ) -> Bucket;
        fn get_redemption_value(
            &self,
            amount_of_pool_units: Decimal,
        ) -> IndexMap<ResourceAddress, Decimal>;
        fn get_vault_amounts(&self) -> IndexMap<ResourceAddress, Decimal>;
    }
}

#[blueprint_with_traits]
pub mod adapter {
    struct OciswapAdapter;

    impl OciswapAdapter {
        fn pool(
            component_address: ComponentAddress,
        ) -> OciswapPoolInterfaceScryptoStub {
            OciswapPoolInterfaceScryptoStub::from(component_address)
        }
    }

    impl PoolAdapterInterfaceTrait for OciswapAdapter {
        fn open_liquidity_position(
            &mut self,
            pool_address: ComponentAddress,
            buckets: (Bucket, Bucket),
        ) -> OpenLiquidityPositionOutput {
            let mut pool = Self::pool(pool_address);

            // TODO: Is this actually pool units and change?
            let (pool_units, change) = pool.add_liquidity(buckets.0, buckets.1);

            OpenLiquidityPositionOutput {
                pool_units,
                change: change
                    .map(|bucket| {
                        indexmap! {
                            bucket.resource_address() => bucket
                        }
                    })
                    .unwrap_or(indexmap! {}),
                others: Default::default(),
            }
        }

        fn close_liquidity_position(
            &mut self,
            pool_address: ComponentAddress,
            pool_units: Bucket,
        ) -> CloseLiquidityPositionOutput {
            let mut pool = Self::pool(pool_address);

            let (bucket1, bucket2) = pool.remove_liquidity(pool_units);

            CloseLiquidityPositionOutput {
                resources: indexmap! {
                    bucket1.resource_address() => bucket1,
                    bucket2.resource_address() => bucket2,
                },
                others: Default::default(),
                // TODO: Determine how we wish to go about this calculation.
                fees: Default::default(),
            };

            todo!()
        }

        // TODO: Is this the same as getting the price from Ociswap directly
        // via a method call?
        fn price(&mut self, pool_address: ComponentAddress) -> Price {
            let pool = Self::pool(pool_address);
            let pool = Global::<TwoResourcePool>::from(pool.liquidity_pool());
            let vault_amounts = pool.get_vault_amounts();

            let (resource_address1, resource_address2) =
                self.resource_addresses(pool_address);
            let amount1 = *vault_amounts
                .get(&resource_address1)
                .expect("Must be defined!");
            let amount2 = *vault_amounts
                .get(&resource_address2)
                .expect("Must be defined!");

            Price {
                base: resource_address1,
                quote: resource_address2,
                price: amount2 / amount1,
            }
        }

        fn resource_addresses(
            &mut self,
            pool_address: ComponentAddress,
        ) -> (ResourceAddress, ResourceAddress) {
            let pool = Self::pool(pool_address);
            let pool = Global::<TwoResourcePool>::from(pool.liquidity_pool());
            let mut keys = pool.get_vault_amounts().into_keys();

            let resource_address1 = keys.next().expect("Must be defined!");
            let resource_address2 = keys.next().expect("Must be defined!");

            (resource_address1, resource_address2)
        }
    }
}
