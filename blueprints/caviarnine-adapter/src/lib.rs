//! This crate and module implements the CaviarSwap adapter that is used by the
//! Olympus to translate high-level commands such as `open_liquidity_position`
//! and `close_liquidity_position` to lower level methods and function calls
//! that need to be made to the CaviarSwap components to achieve those higher
//! level intents.
//!
//! This is an adapter for Caviar's QuantaSwap blueprint which is the main
//! blueprint used in their concentrated-liquidity liquidity pools and the
//! one of interest to Olympus.
//!
//! Since CaviarNine follows Uniswap v3 concentrated liquidity approach this
//! makes this adapter implementation more complex than a Uniswap v2 style
//! liquidity pool such as that of Ociswap. This doc-string aims to document
//! and explain the logic that this adapter implements and also explain the
//! relevant portions of CaviarNine's model.
//!
//! First, in a concentrated liquidity market makers allow users to contribute
//! liquidity in a particular price range. Typically, the user does not have
//! complete control over what price range to contribute their liquidity to,
//! just some amount of control. To elaborate, each pool has discrete price
//! ranges, so, a user must provide their liquidity within the discrete price
//! ranges defined on the pool.
//!
//! CaviarNine uses the terms "Tick" and "Bin". A Tick is a discrete point in
//! price logarithmic space. A Bin holds liquidity for a discrete part of price
//! logarithmic space. A Bin's position is set by the Tick of it's lower bound.
//! The number of Ticks a Bin covers, and therefore it's upper bound is set by
//! the `bin_span` parameter of the pool.
//!
//! All pools always have exactly one active bin at all times; all other bins
//! are inactive. An important point to note is that when a bin is inactive:
//! * No trades go through it (when a bin is inactive, it means that the assets
//! are not being traded at the Bin's price).
//! * Since no trades go through inactive bins they do not earn fees while
//! inactive.
//! * An inactive bin is composed entirely of a single asset. The _only_ bin
//! that has two assets is the currently active bin.
//!
//! Aside from the glaring differences that exist between the concentrated
//! liquidity model and the uniswap v2 model, there are some other subtle
//! differences that should be highlighted and accounted for in the adapter:
//!
//! * One main difference between CaviarNine and a Uniswap v2-style DEX is that
//! a liquidity provider does not own a portion of the pool, they own a portion
//! of the bin that they added liquidity to.
//! * A user who wishes to add liquidity to non-active bins does not have to
//! contribute both of the assets, all that they need is one of them. Which
//! asset is needed depends on whether the bin being contributed to is above
//! or below the currently active bin.
//! * With the above point in mind, this leads us to a bid difference between
//! the two models: in uniswap-v2, a pool's value is split 50/50 between the
//! two assets if the market is efficient and prices are being constantly
//! arbitraged. However, in an efficient market, a CaviarNine's pool value may
//! or may not be split 50/50.
//!
//! This adapter tries its best to treat CaviarSwap almost as if it does not
//! have concentrated liquidity. So, the hope is to implement something that
//! adds liquidity in the ranges [0, ∞]. However, this is not possible to do
//! as CaviarNine allows for a maximum of 200 positions in a receipt (an NFT),
//! in other words, we can add liquidity to a maximum of 200 price ranges. If
//! we wanted, we can work around that by providing liquidity multiple times,
//! but we do not do that for this adapter implementation.
//!
//! Since we can only provide liquidity to 200 bins, we provide liquidity to the
//! active bin, to 99 bins of ticks less than the active bin, and to 98 bins
//! with ticks more than the active bin, this totals up to 199 bins, which is
//! one less than the maximum we're allowed to contribute to. Liquidity will be
//! provided to these bins equally; meaning we will provide the `x` bins with
//! `amount_x / len(x)` bins, and the same for the other resource.
//!
//! As eluded to previously, inactive bins only hold one resource and not both.
//! Bins with ticks lower than that of the active tick only have the `y` asset,
//! and the opposite for the other side. So, the `x` resources will be divided
//! across 100 bins and the y resources will be divided across another 100 bins.
//!
//! The bin selection algorithm tries to select 199 bins with the currently
//! active bin in the center, 98 lower bins, and 98 higher bins. However, there
//! are certain cases when this is not possible such as cases when the active
//! bin is skewed in one direction or another. In such case, the algorithm tries
//! to compensate on the other non-skewed side and attempts to get us 98 lower
//! and 98 higher. In cases when the bin span is too large, it is possible that
//! we get less 199 bins.

mod bin_selector;
pub use bin_selector::*;

use adapters_interface::oracle::*;
use adapters_interface::pool::*;
use scrypto::prelude::*;
use scrypto_interface::*;

define_interface! {
    QuantaSwap as CaviarNinePool {
        fn new(
            owner_rule: AccessRule,
            user_rule: AccessRule,
            token_x_address: ResourceAddress,
            token_y_address: ResourceAddress,
            bin_span: u32,
            reservation: Option<GlobalAddressReservation>,
        ) -> Self;
        fn get_fee_controller_address(&self) -> ComponentAddress;
        fn get_fee_vaults_address(&self) -> ComponentAddress;
        fn get_token_x_address(&self) -> ResourceAddress;
        fn get_token_y_address(&self) -> ResourceAddress;
        fn get_liquidity_receipt_address(&self) -> ResourceAddress;
        fn get_bin_span(&self) -> u32;
        fn get_amount_x(&self) -> Decimal;
        fn get_amount_y(&self) -> Decimal;
        fn get_active_tick(&self) -> Option<u32>;
        fn get_price(&self) -> Option<Decimal>;
        fn get_active_bin_price_range(&self) -> Option<(Decimal, Decimal)>;
        fn get_active_amounts(&self) -> Option<(Decimal, Decimal)>;
        fn get_bins_above(
            &self,
            start_tick: Option<u32>,
            stop_tick: Option<u32>,
            number: Option<u32>,
        ) -> Vec<(u32, Decimal)>;
        fn get_bins_below(
            &self,
            start_tick: Option<u32>,
            stop_tick: Option<u32>,
            number: Option<u32>,
        ) -> Vec<(u32, Decimal)>;
        fn get_liquidity_claims(
            &self,
            liquidity_receipt_id: NonFungibleLocalId,
        ) -> IndexMap<u32, Decimal>;
        fn get_redemption_value(&self, liquidity_receipt_id: NonFungibleLocalId) -> (Decimal, Decimal);
        fn get_redemption_bin_values(
            &self,
            liquidity_receipt_id: NonFungibleLocalId,
        ) -> Vec<(u32, Decimal, Decimal)>;
        fn mint_liquidity_receipt(&mut self) -> Bucket;
        fn burn_liquidity_receipt(&mut self, liquidity_receipt: Bucket);
        fn add_liquidity_to_receipt(
            &mut self,
            liquidity_receipt: Bucket,
            tokens_x: Bucket,
            tokens_y: Bucket,
            positions: Vec<(u32, Decimal, Decimal)>,
        ) -> (Bucket, Bucket, Bucket);
        fn add_liquidity(
            &mut self,
            tokens_x: Bucket,
            tokens_y: Bucket,
            positions: Vec<(u32, Decimal, Decimal)>,
        ) -> (Bucket, Bucket, Bucket);
        fn remove_specific_liquidity(
            &mut self,
            liquidity_receipt: Bucket,
            claims: Vec<(u32, Decimal)>,
        ) -> (Bucket, Bucket, Bucket);
        fn remove_liquidity(&mut self, liquidity_receipt: Bucket) -> (Bucket, Bucket);
        fn swap(&mut self, tokens: Bucket) -> (Bucket, Bucket);
    }
}

#[blueprint_with_traits]
mod adapter {
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
    }

    impl PoolAdapterInterfaceTrait for CaviarNineAdapter {
        // Opens 199 positions in flat formation. One at the current price, 99
        // at the next 99 lower bins and 99 and the next 99 higher bins.
        fn open_liquidity_position(
            &mut self,
            pool_address: ComponentAddress,
            buckets: (Bucket, Bucket),
        ) -> OpenLiquidityPositionOutput {
            let mut pool =
                CaviarNinePoolInterfaceScryptoStub::from(pool_address);

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
                panic!("One or more of the buckets do not belong to the pool")
            };
            let amount_x = bucket_x.amount();
            let amount_y = bucket_y.amount();

            // Determine all of the bins that we will be adding positions for.
            let bin_span = pool.get_bin_span();
            let active_bin =
                pool.get_active_tick().expect("Pool has no active bin!");
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
                pool.get_active_amounts().expect("No active amounts");
            let price = pool.get_price().expect("No price");

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
                adapter_specific_data: AnyScryptoValue::from_typed(
                    &CaviarnineAdapterData {},
                ),
            }
        }

        fn close_liquidity_position(
            &mut self,
            _pool_address: ComponentAddress,
            _pool_units: Bucket,
            _current_oracle_price: Price,
            _adapter_specific_data: AnyScryptoValue,
        ) -> CloseLiquidityPositionOutput {
            todo!()
        }
    }
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, ScryptoSbor,
)]
pub struct CaviarnineAdapterData {
    // TODO: Yet to determine what is required for the adapter to be able to
    // estimate the fees earned on the position. Fill this in once we have a
    // model for this calculation.
}
