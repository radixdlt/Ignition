mod blueprint_interface;
pub use blueprint_interface::*;

use common::prelude::*;
use ports_interface::prelude::*;
use scrypto::prelude::*;
use scrypto_interface::*;
use scrypto_math::*;

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
    INVALID_NUMBER_OF_BUCKETS => "Invalid number of buckets.";
}

macro_rules! pool {
    ($address: expr) => {
        $crate::blueprint_interface::OciswapV2PoolInterfaceScryptoStub::from(
            $address,
        )
    };
}

#[blueprint_with_traits]
pub mod adapter {
    struct OciswapV2Adapter;

    impl OciswapV2Adapter {
        pub fn instantiate(
            _: AccessRule,
            _: AccessRule,
            metadata_init: MetadataInit,
            owner_role: OwnerRole,
            address_reservation: Option<GlobalAddressReservation>,
        ) -> Global<OciswapV2Adapter> {
            let address_reservation =
                address_reservation.unwrap_or_else(|| {
                    Runtime::allocate_component_address(BlueprintId {
                        package_address: Runtime::package_address(),
                        blueprint_name: Runtime::blueprint_name(),
                    })
                    .0
                });

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

        pub fn liquidity_receipt_data(
            // Does not depend on state, this is kept in case this is required
            // in the future for whatever reason.
            &self,
            global_id: NonFungibleGlobalId,
        ) -> LiquidityReceipt<OciswapV2AdapterSpecificInformation> {
            // Read the non-fungible data.
            let LiquidityReceipt {
                name,
                lockup_period,
                pool_address,
                user_resource_address,
                user_contribution_amount,
                user_resource_volatility_classification,
                protocol_contribution_amount,
                maturity_date,
                adapter_specific_information,
            } = ResourceManager::from_address(global_id.resource_address())
                .get_non_fungible_data::<LiquidityReceipt<AnyValue>>(
                global_id.local_id(),
            );
            let adapter_specific_information = adapter_specific_information
                .as_typed::<OciswapV2AdapterSpecificInformation>()
                .unwrap();

            LiquidityReceipt {
                name,
                lockup_period,
                pool_address,
                user_resource_address,
                user_contribution_amount,
                user_resource_volatility_classification,
                protocol_contribution_amount,
                maturity_date,
                adapter_specific_information,
            }
        }
    }

    impl PoolAdapterInterfaceTrait for OciswapV2Adapter {
        fn open_liquidity_position(
            &mut self,
            pool_address: ComponentAddress,
            buckets: (Bucket, Bucket),
        ) -> OpenLiquidityPositionOutput {
            let mut pool = pool!(pool_address);

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
            //
            // In Ociswap v2, prices can be calculated from ticks by using the
            // equation p(t) = 1.0001^t. The currently active tick can be found
            // from the current price by ln(price) / ln(1.0001).
            //
            // The following calculation finds the currently active tick based
            // on the equation above which all happens using the PreciseDecimal
            // type. To use the active tick we must convert it to an i32 which
            // is expected by the Ociswap interface so the I256 of the computed
            // active tick is divided by PreciseDecimal::ONE.0 to remove all of
            // the decimal places and just have the integral part which we then
            // call i32::try_from on.
            let active_tick = pool
                .price_sqrt()
                .checked_powi(2)
                .and_then(|value| value.ln())
                .and_then(|ln_price| {
                    dec!(1.0001)
                        .ln()
                        .and_then(|ln_base| ln_price.checked_div(ln_base))
                })
                .and_then(|value| value.0.checked_div(PreciseDecimal::ONE.0))
                .and_then(|value| i32::try_from(value).ok())
                .expect(OVERFLOW_ERROR);
            let offset = 29959;

            let lower_tick =
                active_tick.checked_sub(offset).expect(OVERFLOW_ERROR);
            let upper_tick =
                active_tick.checked_add(offset).expect(OVERFLOW_ERROR);

            let (receipt, change_x, change_y) =
                pool.add_liquidity(lower_tick, upper_tick, bucket_x, bucket_y);

            OpenLiquidityPositionOutput {
                pool_units: IndexedBuckets::from_bucket(receipt),
                change: IndexedBuckets::from_buckets([change_x, change_y]),
                others: Default::default(),
                adapter_specific_information: AnyValue::from_typed(&())
                    .expect(UNEXPECTED_ERROR),
            }
        }

        fn close_liquidity_position(
            &mut self,
            pool_address: ComponentAddress,
            mut pool_units: Vec<Bucket>,
            _: AnyValue,
        ) -> CloseLiquidityPositionOutput {
            let mut pool = pool!(pool_address);
            let pool_units = {
                let pool_units_bucket =
                    pool_units.pop().expect(INVALID_NUMBER_OF_BUCKETS);
                if !pool_units.is_empty() {
                    panic!("{}", INVALID_NUMBER_OF_BUCKETS)
                }
                pool_units_bucket
            };

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
                resources: IndexedBuckets::from_buckets([
                    resource_x, resource_y,
                ]),
                others: vec![],
                fees: indexmap! {
                    resource_address_x => fees_x,
                    resource_address_y => fees_y,
                },
            }
        }

        fn price(&mut self, pool_address: ComponentAddress) -> Price {
            let pool = pool!(pool_address);
            let price_sqrt = pool.price_sqrt();
            let price = price_sqrt
                .checked_powi(2)
                .and_then(|value| Decimal::try_from(value).ok())
                .expect(OVERFLOW_ERROR);
            let (resource_x, resource_y) = (pool.x_address(), pool.y_address());
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
            let pool = pool!(pool_address);
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
