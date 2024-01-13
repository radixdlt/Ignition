mod blueprint_interface;
pub use blueprint_interface::*;

use adapters_interface::prelude::*;
use scrypto::prelude::*;
use scrypto_interface::*;

macro_rules! define_error {
    (
        $(
            $name: ident => $item: expr;
        )*
    ) => {
        $(
            const $name: &'static str = concat!("[Ociswap Adapter]", " ", $item);
        )*
    };
}

define_error! {
    FAILED_TO_GET_RESOURCE_ADDRESSES_ERROR
        => "Failed to get resource addresses - unexpected error.";
    FAILED_TO_GET_VAULT_ERROR
        => "Failed to get vault - unexpected error.";
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
                    .unwrap_or_default(),
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
            }
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
                .expect(FAILED_TO_GET_VAULT_ERROR);
            let amount2 = *vault_amounts
                .get(&resource_address2)
                .expect(FAILED_TO_GET_VAULT_ERROR);

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

            let resource_address1 =
                keys.next().expect(FAILED_TO_GET_RESOURCE_ADDRESSES_ERROR);
            let resource_address2 =
                keys.next().expect(FAILED_TO_GET_RESOURCE_ADDRESSES_ERROR);

            (resource_address1, resource_address2)
        }
    }
}