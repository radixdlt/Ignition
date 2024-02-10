#![warn(clippy::arithmetic_side_effects)]

mod blueprint_interface;
pub use blueprint_interface::*;

use ports_interface::prelude::*;
use scrypto::prelude::*;
use scrypto_interface::*;

macro_rules! define_error {
    (
        $(
            $name: ident => $item: expr;
        )*
    ) => {
        $(
            pub const $name: &'static str = concat!("[Ociswap v2 Adapter v1]", " ", $item);
        )*
    };
}

define_error! {
    RESOURCE_DOES_NOT_BELONG_ERROR
        => "One or more of the resources do not belong to pool.";
    OVERFLOW_ERROR => "Calculation overflowed.";
    UNEXPECTED_ERROR => "Unexpected error.";
}

#[blueprint_with_traits]
pub mod adapter {
    struct OciswapV2Adapter;

    impl OciswapV2Adapter {
        pub fn instantiate(
            metadata_init: MetadataInit,
            owner_role: OwnerRole,
            address_reservation: Option<GlobalAddressReservation>,
        ) -> Global<OciswapV2Adapter> {
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
                .metadata(ModuleConfig {
                    init: metadata_init,
                    roles: Default::default(),
                })
                .with_address(address_reservation)
                .globalize()
        }

        fn pool(
            component_address: ComponentAddress,
        ) -> OciswapV2PoolInterfaceScryptoStub {
            OciswapV2PoolInterfaceScryptoStub::from(component_address)
        }
    }

    impl PoolAdapterInterfaceTrait for OciswapV2Adapter {
        fn open_liquidity_position(
            &mut self,
            pool_address: ComponentAddress,
            buckets: (Bucket, Bucket),
        ) -> OpenLiquidityPositionOutput {
            let mut pool = Self::pool(pool_address);

            // Sorting the buckets according to the ordering of the pool itself.
            let (bucket_x, bucket_y) = {
                let resource_x = pool.x_address();
                let resource_y = pool.y_address();

                if buckets.0.resource_address() == resource_x
                    && buckets.1.resource_address() == resource_y
                {
                    (buckets.0, buckets.1)
                } else if buckets.1.resource_address() == resource_x
                    && buckets.0.resource_address() == resource_y
                {
                    (buckets.1, buckets.0)
                } else {
                    panic!("{}", RESOURCE_DOES_NOT_BELONG_ERROR)
                }
            };

            // Contributing liquidity to the pool - the offset that is defined
            // here is the amount of ticks that we need to contribute to get to
            // a 20x upside and downside. We calculate this through a function
            // provided by Ociswap: offset = ln(multiplier) / ln(1.0001) and
            // then round up.
            let active_tick = pool.active_tick();
            let offset = 29959;

            let lower_tick =
                active_tick.checked_sub(offset).expect(OVERFLOW_ERROR);
            let upper_tick =
                active_tick.checked_add(offset).expect(OVERFLOW_ERROR);

            let (receipt, change_x, change_y) =
                pool.add_liquidity(lower_tick, upper_tick, bucket_x, bucket_y);

            OpenLiquidityPositionOutput {
                pool_units: receipt,
                change: indexmap! {
                    change_x.resource_address() => change_x,
                    change_y.resource_address() => change_y,
                },
                others: Default::default(),
                adapter_specific_information: AnyValue::from_typed(&())
                    .expect(UNEXPECTED_ERROR),
            }
        }

        fn close_liquidity_position(
            &mut self,
            pool_address: ComponentAddress,
            pool_units: Bucket,
            _: AnyValue,
        ) -> CloseLiquidityPositionOutput {
            let mut pool = Self::pool(pool_address);

            // Calculate how much fees were earned on the position while it was
            // opened.
            let resource_address_x = pool.x_address();
            let resource_address_y = pool.y_address();
            let (fees_x, fees_y) = pool.total_fees(
                pool_units.as_non_fungible().non_fungible_local_id(),
            );

            // Close the liquidity position
            let (resource_x, resource_y) =
                pool.remove_liquidity(pool_units.as_non_fungible());

            CloseLiquidityPositionOutput {
                resources: indexmap! {
                    resource_x.resource_address() => resource_x,
                    resource_y.resource_address() => resource_y,
                },
                others: vec![],
                fees: indexmap! {
                    resource_address_x => fees_x,
                    resource_address_y => fees_y,
                },
            }
        }

        fn price(&mut self, pool_address: ComponentAddress) -> Price {
            let pool = Self::pool(pool_address);
            let price_sqrt = pool.price_sqrt();
            let price = price_sqrt
                .checked_powi(2)
                .and_then(|value| Decimal::try_from(value).ok())
                .expect(OVERFLOW_ERROR);
            let (resource_x, resource_y) =
                self.resource_addresses(pool_address);
            Price {
                base: resource_x,
                quote: resource_y,
                price,
            }
        }

        fn resource_addresses(
            &mut self,
            pool_address: ComponentAddress,
        ) -> (ResourceAddress, ResourceAddress) {
            let pool = Self::pool(pool_address);
            (pool.x_address(), pool.y_address())
        }
    }
}

#[derive(ScryptoSbor, Debug, Clone)]
pub struct OciswapV2AdapterSpecificInformation {}

impl From<OciswapV2AdapterSpecificInformation> for AnyValue {
    fn from(value: OciswapV2AdapterSpecificInformation) -> Self {
        AnyValue::from_typed(&value).unwrap()
    }
}
