mod bin_selector;
mod blueprint_interface;

pub use crate::bin_selector::*;
pub use crate::blueprint_interface::*;

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
            const $name: &'static str = concat!("[CaviarNine Adapter]", " ", $item);
        )*
    };
}

define_error! {
    RESOURCE_DOES_NOT_BELONG_ERROR
        => "One or more of the resources do not belong to pool.";
    NO_ACTIVE_BIN_ERROR
        => "Pool has no active bin.";
    NO_ACTIVE_AMOUNTS_ERROR
        => "Pool has no active amounts.";
    NO_PRICE_ERROR
        => "Pool has no price.";
}

#[blueprint_with_traits]
pub mod adapter {
    struct CaviarNineAdapter;

    impl CaviarNineAdapter {
        pub fn instantiate(
            owner_role: OwnerRole,
            address_reservation: Option<GlobalAddressReservation>,
        ) -> Global<CaviarNineAdapter> {
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

        fn pool(
            component_address: ComponentAddress,
        ) -> CaviarNinePoolInterfaceScryptoStub {
            CaviarNinePoolInterfaceScryptoStub::from(component_address)
        }
    }

    impl PoolAdapterInterfaceTrait for CaviarNineAdapter {
        fn open_liquidity_position(
            &mut self,
            pool_address: ComponentAddress,
            buckets: (Bucket, Bucket),
        ) -> OpenLiquidityPositionOutput {
            let mut pool = Self::pool(pool_address);

            // Split the two buckets into bucket_x and bucket_y in the same way
            // that they're defined in the pool itself.
            let resource_address_x = pool.get_token_x_address();
            let resource_address_y = pool.get_token_y_address();
            let (bucket_x, bucket_y) = if buckets.0.resource_address()
                == resource_address_x
                && buckets.1.resource_address() == resource_address_y
            {
                (buckets.0, buckets.1)
            } else if buckets.1.resource_address() == resource_address_x
                && buckets.0.resource_address() == resource_address_y
            {
                (buckets.1, buckets.0)
            } else {
                panic!("{}", RESOURCE_DOES_NOT_BELONG_ERROR)
            };
            let amount_x = bucket_x.amount();
            let amount_y = bucket_y.amount();

            // Select the bins that we will contribute to.
            let bin_span = pool.get_bin_span();
            let active_bin = pool.get_active_tick().expect(NO_ACTIVE_BIN_ERROR);
            let SelectedBins {
                higher_bins,
                lower_bins,
                ..
            } = SelectedBins::select(active_bin, bin_span, 198);

            // Determine the amount of resources that we will add to each of the
            // bins. We have 99 on the left and 99 on the right. But, we also
            // have the active bin that is composed of both x and y. So, this
            // be like contributing to 99.x and 99.y bins where x = 1-y. X here
            // is the ratio of resources x in the active bin.
            let (amount_in_active_bin_x, amount_in_active_bin_y) =
                pool.get_active_amounts().expect(NO_ACTIVE_AMOUNTS_ERROR);
            let price = pool.get_price().expect(NO_PRICE_ERROR);

            let ratio_in_active_bin_x = amount_in_active_bin_x * price
                / (amount_in_active_bin_x * price + amount_in_active_bin_y);
            let ratio_in_active_bin_y = Decimal::one() - ratio_in_active_bin_x;

            // In here, we decide the amount x by the number of higher bins plus
            // the ratio of the x in the currently active bin since the pool
            // starting from the current price and upward is entirely composed
            // of X. Similarly, we divide amount_y by the number of lower
            // positions plus the ratio of y in the active bin since the pool
            // starting from the current price and downward is composed just of
            // y.
            let position_amount_x = amount_x
                / (Decimal::from(higher_bins.len() as u32)
                    + ratio_in_active_bin_x);
            let position_amount_y = amount_y
                / (Decimal::from(lower_bins.len() as u32)
                    + ratio_in_active_bin_y);

            // TODO: What?
            let amount_bin_x_in_y = position_amount_x * price;
            let (position_amount_x, position_amount_y) =
                if amount_bin_x_in_y > position_amount_y {
                    let position_amount_y_in_x = position_amount_y / price;
                    (position_amount_y_in_x, position_amount_y)
                } else {
                    (position_amount_x, amount_bin_x_in_y)
                };

            let mut positions = vec![(
                active_bin,
                position_amount_x * ratio_in_active_bin_x,
                position_amount_y * ratio_in_active_bin_y,
            )];
            positions.extend(
                lower_bins
                    .iter()
                    .map(|bin_id| (*bin_id, dec!(0), position_amount_y)),
            );
            positions.extend(
                lower_bins
                    .iter()
                    .map(|bin_id| (*bin_id, position_amount_x, dec!(0))),
            );

            let (receipt, change_x, change_y) =
                pool.add_liquidity(bucket_x, bucket_y, positions);

            OpenLiquidityPositionOutput {
                pool_units: receipt,
                change: indexmap! {
                    change_x.resource_address() => change_x,
                    change_y.resource_address() => change_y,
                },
                others: vec![],
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

        fn price(&mut self, pool_address: ComponentAddress) -> Price {
            let pool = Self::pool(pool_address);

            let (resource_address_x, resource_address_y) =
                self.resource_addresses(pool_address);
            let price = pool.get_price().expect(NO_PRICE_ERROR);

            Price {
                base: resource_address_x,
                quote: resource_address_y,
                price,
            }
        }

        fn resource_addresses(
            &mut self,
            pool_address: ComponentAddress,
        ) -> (ResourceAddress, ResourceAddress) {
            let pool = Self::pool(pool_address);

            (pool.get_token_x_address(), pool.get_token_y_address())
        }
    }
}
