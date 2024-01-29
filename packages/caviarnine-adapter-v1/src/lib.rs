#![allow(clippy::new_without_default)]

mod bin_selector;
mod blueprint_interface;
mod tick_math;

pub use crate::bin_selector::*;
pub use crate::blueprint_interface::*;
pub use crate::tick_math::*;

use adapters_interface::prelude::*;
use scrypto::prelude::*;
use scrypto_interface::*;

use std::cmp::*;
use std::ops::*;

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
    NO_ACTIVE_BIN_ERROR => "Pool has no active bin.";
    NO_ACTIVE_AMOUNTS_ERROR => "Pool has no active amounts.";
    NO_PRICE_ERROR => "Pool has no price.";
    TICK_ARITHMETIC_ERROR => "An error has happened in the tick arithmetic.";
    OVERFLOW_ERROR => "Overflow error.";
    UNEXPECTED_ERROR => "Unexpected error.";
}

/// The total number of bins that we will be using on the left and the right
/// excluding the one in the middle.
pub const PREFERRED_TOTAL_NUMBER_OF_HIGHER_AND_LOWER_BINS: u32 = 10 * 2;

#[blueprint_with_traits]
pub mod adapter {
    struct CaviarnineAdapter;

    impl CaviarnineAdapter {
        pub fn instantiate(
            metadata_init: MetadataInit,
            owner_role: OwnerRole,
            address_reservation: Option<GlobalAddressReservation>,
        ) -> Global<CaviarnineAdapter> {
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
        ) -> CaviarNinePoolInterfaceScryptoStub {
            CaviarNinePoolInterfaceScryptoStub::from(component_address)
        }
    }

    impl PoolAdapterInterfaceTrait for CaviarnineAdapter {
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

            let bucket_0_resource_address = buckets.0.resource_address();
            let bucket_1_resource_address = buckets.1.resource_address();

            let (bucket_x, bucket_y) = if bucket_0_resource_address
                == resource_address_x
                && bucket_1_resource_address == resource_address_y
            {
                (buckets.0, buckets.1)
            } else if bucket_1_resource_address == resource_address_x
                && bucket_0_resource_address == resource_address_y
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
                highest_bin,
                lowest_bin,
                ..
            } = SelectedBins::select(
                active_bin,
                bin_span,
                PREFERRED_TOTAL_NUMBER_OF_HIGHER_AND_LOWER_BINS,
            );

            // Determine the amount of resources that we will add to each of the
            // bins. We have 62 on the left and 62 on the right. But, we also
            // have the active bin that is composed of both x and y. So, this
            // be like contributing to 62.x and 62.y bins where x = 1-y. X here
            // is the ratio of resources x in the active bin.
            let (amount_in_active_bin_x, amount_in_active_bin_y) =
                pool.get_active_amounts().expect(NO_ACTIVE_AMOUNTS_ERROR);
            let pool_price = self.price(pool_address);
            let price = pool_price.price;

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
                higher_bins
                    .iter()
                    .map(|bin_id| (*bin_id, position_amount_x, dec!(0))),
            );

            let (receipt, change_x, change_y) =
                pool.add_liquidity(bucket_x, bucket_y, positions.clone());

            // Creating the adapter-specific information of this position.
            let adapter_specific_information = {
                // The adapter-specific information of the Caviarnine adapter
                // contains two main data points that are needed to estimate the
                // fees when closing the liquidity position: a) the reserves of
                // the assets in the various bins _we've contributed to_, and
                // b) the amount that was contributed to each bin. Therefore,
                // it also implicitly contains the share of the user in these
                // particular bins as it's the ratio if how much they added to
                // the bin to the total amount of resources in the bin.
                //
                // For the first data point, the reserves of the assets in the
                // pool, we have no way other than querying the Caviarnine pool
                // for this information as there is pretty much no other way we
                // can go about finding this. The methods that we can use to do
                // that are: `get_bins_above` and `get_bins_below`. These two
                // methods return the amount of x and y assets in the pool.
                // Recall that the active bin is the only bin that contains both
                // x and y assets and that the bins _above_ the current active
                // bin contain only x and those below contain only y. Therefore,
                // what `get_bins_above` is actually the amount of x resources
                // in the active bin and all higher bins and `get_bins_below` is
                // the amount of y resources in the active bin and all the lower
                // bins. We aggregate those per bin to determine the reserves of
                // the bins we've contributed to. We have no interest in other
                // bins.
                //
                // For the amount of resources that were contributed to each bin
                // there are two ways we can go about finding that. Either by
                // using the `positions` defined above or by contributing and
                // then getting the worth of the liquidity position by invoking
                // caviarnine. The latter approach has proved too expensive and
                // pushes us over the fee limit therefore we're pretty much
                // locked into approach 1. With the first approach the change
                // must be handled as its an amount that was not contributed and
                // therefore should not be added to our contribution. We can use
                // a property of the Caviarnine blueprints where they _never_
                // return any change for any bins that are not the active bin.
                // Therefore, it is guaranteed that all of the change received
                // back from the contribution originated from the active bin.

                let mut adapter_specific_information =
                    CaviarnineAdapterSpecificInformation::new(pool_price);

                // Reserves calculations
                let reserves_x =
                    pool.get_bins_above(None, Some(highest_bin), None);
                let reserves_y =
                    pool.get_bins_below(None, Some(lowest_bin), None);

                for (bin, reserves) in reserves_x
                    .into_iter()
                    .map(|(bin, x)| {
                        (
                            bin,
                            ResourceIndexedData {
                                resource_x: x,
                                resource_y: Decimal::ZERO,
                            },
                        )
                    })
                    .chain(reserves_y.into_iter().map(|(bin, y)| {
                        (
                            bin,
                            ResourceIndexedData {
                                resource_y: y,
                                resource_x: Decimal::ZERO,
                            },
                        )
                    }))
                {
                    adapter_specific_information
                        .bin_information_after_position_opened
                        .entry(bin)
                        .or_default()
                        .reserves += reserves
                }

                // Contributions calculations
                for (bin, position) in
                    positions.into_iter().map(|(bin, x, y)| {
                        (
                            bin,
                            ResourceIndexedData {
                                resource_x: x,
                                resource_y: y,
                            },
                        )
                    })
                {
                    adapter_specific_information
                        .bin_information_after_position_opened
                        .entry(bin)
                        .or_default()
                        .contribution += position
                }

                // Final step, account for the change. As mentioned above, there
                // is only one bin that the change could've come from and it's
                // the active bin.
                let active_amounts =
                    pool.get_active_amounts().expect(NO_ACTIVE_AMOUNTS_ERROR);
                adapter_specific_information
                    .bin_information_after_position_opened
                    .get_mut(&active_bin)
                    .unwrap()
                    .contribution = ResourceIndexedData {
                    resource_x: active_amounts.0 - amount_in_active_bin_x,
                    resource_y: active_amounts.1 - amount_in_active_bin_y,
                };

                adapter_specific_information
            };

            OpenLiquidityPositionOutput {
                pool_units: receipt,
                change: indexmap! {
                    change_x.resource_address() => change_x,
                    change_y.resource_address() => change_y,
                },
                others: vec![],
                adapter_specific_information: adapter_specific_information
                    .into(),
            }
        }

        fn close_liquidity_position(
            &mut self,
            pool_address: ComponentAddress,
            pool_units: Bucket,
            adapter_specific_information: AnyValue,
        ) -> CloseLiquidityPositionOutput {
            let mut pool = Self::pool(pool_address);
            let bin_span = pool.get_bin_span();
            let active_bin = pool.get_active_tick().expect(NO_ACTIVE_BIN_ERROR);
            let resource_x = pool.get_token_x_address();
            let resource_y = pool.get_token_y_address();

            let redemption_value = pool.get_redemption_bin_values(
                pool_units.as_non_fungible().non_fungible_local_id(),
            );

            let (bucket1, bucket2) = pool.remove_liquidity(pool_units);

            // Calculating the fees on the position. The way we go about this is
            // by calculating how much we expect to close our position for based
            // on information we know about it such as the contribution we made
            // and the reserves in that particular bin. Once we have calculated
            // this theoretical amount we say that the fees is the difference
            // between this theoretical amount and the actual amount.
            let amount_expected_in_bins = {
                let mut amount_expected_in_bins =
                    IndexMap::<u32, ResourceIndexedData<Decimal>>::new();

                // Decoding the adapter specific information as the type we
                // expect it to be.
                let CaviarnineAdapterSpecificInformation {
                    bin_information_after_position_opened,
                    price_after_position_opened,
                } = adapter_specific_information.as_typed().unwrap();

                // Iterating over each bin we contributed to and doing the math
                // needed to populate `amount_expected_in_bins`
                for (tick, bin_information) in
                    bin_information_after_position_opened.into_iter()
                {
                    // Computing some information based on the bin's tick such
                    // that is is available throughout this block.
                    let bin_start = tick;
                    let bin_end = tick + bin_span;

                    let lower_price =
                        tick_to_spot(bin_start).expect(TICK_ARITHMETIC_ERROR);
                    let upper_price =
                        tick_to_spot(bin_end).expect(TICK_ARITHMETIC_ERROR);

                    let average_bin_price = PreciseDecimal::from(lower_price)
                        .checked_add(upper_price)
                        .and_then(|value| value.checked_div(2))
                        .and_then(|value| Decimal::try_from(value).ok())
                        .map(|price| Price {
                            price,
                            base: resource_x,
                            quote: resource_y,
                        })
                        .expect(OVERFLOW_ERROR);

                    // Determine the composition of the bin when the pool was
                    // first opened.
                    let ResourceIndexedData {
                        resource_x: reserves_x,
                        resource_y: reserves_y,
                    } = bin_information.reserves;
                    let contribution @ ResourceIndexedData {
                        resource_x: contribution_x,
                        resource_y: contribution_y,
                    } = bin_information.contribution;

                    let bin_composition_when_position_opened = match (
                        reserves_x == Decimal::ZERO,
                        reserves_y == Decimal::ZERO,
                    ) {
                        // TODO: I think that this is impossible? I believe that
                        // it is impossible since we only capture the bins that
                        // we have contributed to and THEN store the reserves in
                        // the bin. If we contributed to it, how come its empty
                        // of both resources?
                        //
                        // If this is somehow possible, what do we want to do
                        // in this case?
                        (true, true) => panic!("What?"),
                        (true, false) => Composition::EntirelyY,
                        (false, true) => Composition::EntirelyX,
                        (false, false) => Composition::Composite,
                    };

                    // Determine what we expect the composition of this bin to
                    // be based on the current price.
                    let expected_bin_composition_now =
                        match tick.cmp(&active_bin) {
                            // Case A: The current price is inside this bin.
                            // Since we are the current active bin then it's
                            // expected that this bin has both X and Y assets.
                            Ordering::Equal => Composition::Composite,
                            //  // Case B: The current price of the pool is
                            // greater than the upper bound of the bin. We're
                            // outside of that range and there should only be Y
                            // assets in the bin.
                            Ordering::Less => Composition::EntirelyY,
                            // Case C: The current price of the pool is smaller
                            // than the lower bound of the bin. We're outside of
                            // that range and there should only be X assets in
                            // the bin.
                            Ordering::Greater => Composition::EntirelyX,
                        };

                    // Matching over what the bin composition was and what it is
                    // expected to be now to determine what calculation to do.
                    match (
                        bin_composition_when_position_opened,
                        expected_bin_composition_now,
                    ) {
                        // Case A: The bin was composed of some asset and is
                        // still composed of the same asset. In this case, we
                        // do not need to do any calculations. We expect to get
                        // back the same amount we put in.
                        (Composition::EntirelyX, Composition::EntirelyX)
                        | (Composition::EntirelyY, Composition::EntirelyY) => {
                            *amount_expected_in_bins
                                .entry(tick)
                                .or_default() += contribution;
                        }
                        // Case B: The bin was composed of one asset and is now
                        // composed entirely of the other asset. In this case,
                        // we need to "swap" the entire amount that was in the
                        // bin for the other asset.
                        //
                        // TODO: The way that we currently do this conversion
                        // is that we assume that the resources were swapped to
                        // the other asset at the average price of the bin which
                        // is the (lower_bound + upper_bound) / 2. This is due
                        // to us not having any way to find this out accurately.
                        // If a more accurate thing exists then we must get some
                        // support from the C9 team on it.
                        (Composition::EntirelyX, Composition::EntirelyY) => {
                            let (_, output_amount) = average_bin_price
                                .exchange(resource_x, contribution_x)
                                .expect(UNEXPECTED_ERROR);

                            *amount_expected_in_bins
                                .entry(tick)
                                .or_default() += ResourceIndexedData {
                                resource_x: Decimal::ZERO,
                                resource_y: output_amount,
                            };
                        }
                        (Composition::EntirelyY, Composition::EntirelyX) => {
                            let (_, output_amount) = average_bin_price
                                .exchange(resource_y, contribution_y)
                                .expect(UNEXPECTED_ERROR);

                            *amount_expected_in_bins
                                .entry(tick)
                                .or_default() += ResourceIndexedData {
                                resource_x: output_amount,
                                resource_y: Decimal::ZERO,
                            };
                        }
                        // Case C: The bin was entirely composed of one asset
                        // and is now composed of both assets meaning that it
                        // is now the active bin.
                        //
                        // TODO: Inaccurate, we just get the amount from the
                        // pool itself which means that we ignore any fees
                        // earned on such bins. Must change this.
                        (Composition::EntirelyX, Composition::Composite) => {
                            *amount_expected_in_bins.entry(tick).or_default() =
                                redemption_value
                                    .iter()
                                    .find(|(t, ..)| *t == tick)
                                    .map(|value| ResourceIndexedData {
                                        resource_x: value.1,
                                        resource_y: value.2,
                                    })
                                    .unwrap_or_default();
                        }
                        (Composition::EntirelyY, Composition::Composite) => {
                            let (_, _output_amount) = average_bin_price
                                .exchange(resource_y, contribution_y / 2)
                                .expect(UNEXPECTED_ERROR);

                            *amount_expected_in_bins.entry(tick).or_default() =
                                redemption_value
                                    .iter()
                                    .find(|(t, ..)| *t == tick)
                                    .map(|value| ResourceIndexedData {
                                        resource_x: value.1,
                                        resource_y: value.2,
                                    })
                                    .unwrap_or_default();
                        }
                        // Case D: There was a composite amount of resources in
                        // the bin when the position was opened and this amount
                        // has been converted to be entirely of the other asset.
                        //
                        // TODO: The logic we currently use to compute what
                        // should be in the bin at this point is not very
                        // accurate but it is as accurate as we can get without
                        // a mathematical model developed with aid from the C9
                        // team.
                        //
                        // If the bin is currently composed of entirely X then
                        // we say that the Y amount was swapped into X at a
                        // price of avg(p, p_a). If the bin is currently
                        // composed of entirely Y then we say that the X amount
                        // was swapped into Y at a price of avg(p, p_b). Where
                        // p is the price we observed when the position was
                        // opened, p_a is the bin's price lower bound and p_b is
                        // the bin's price upper bound.
                        (Composition::Composite, Composition::EntirelyX) => {
                            let average_bin_price = PreciseDecimal::from(
                                price_after_position_opened.price,
                            )
                            .checked_add(lower_price)
                            .and_then(|value| value.checked_div(2))
                            .and_then(|value| Decimal::try_from(value).ok())
                            .map(|price| Price {
                                price,
                                base: resource_x,
                                quote: resource_y,
                            })
                            .expect(OVERFLOW_ERROR);

                            let (_, output_amount) = average_bin_price
                                .exchange(resource_y, contribution_y)
                                .expect(UNEXPECTED_ERROR);

                            *amount_expected_in_bins
                                .entry(tick)
                                .or_default() += ResourceIndexedData {
                                resource_x: contribution_x + output_amount,
                                resource_y: Decimal::ZERO,
                            };
                        }
                        (Composition::Composite, Composition::EntirelyY) => {
                            let average_bin_price = PreciseDecimal::from(
                                price_after_position_opened.price,
                            )
                            .checked_add(upper_price)
                            .and_then(|value| value.checked_div(2))
                            .and_then(|value| Decimal::try_from(value).ok())
                            .map(|price| Price {
                                price,
                                base: resource_x,
                                quote: resource_y,
                            })
                            .expect(OVERFLOW_ERROR);

                            let (_, output_amount) = average_bin_price
                                .exchange(resource_x, contribution_x)
                                .expect(UNEXPECTED_ERROR);

                            *amount_expected_in_bins
                                .entry(tick)
                                .or_default() += ResourceIndexedData {
                                resource_x: Decimal::ZERO,
                                resource_y: contribution_y + output_amount,
                            };
                        }
                        // Case E: The bin was composite and is still composite.
                        // This means that the position was opened and closed
                        // in the same bin; As in, we never left the price range
                        // of the bin.
                        //
                        // TODO: We do not currently have any way of doing this
                        // in an accurate manner. We assume that the composition
                        // is the same between the time the position was opened
                        // and closed which is quite inaccurate but is as
                        // accurate as we can get. Must get input from the C9
                        // team on this.
                        (Composition::Composite, Composition::Composite) => {
                            *amount_expected_in_bins
                                .entry(tick)
                                .or_default() += contribution;
                        }
                    };
                }

                amount_expected_in_bins
            };

            // Calculating the fees earned on each one of the bins and then
            // summing it up to get the total x and y fees.
            let total_amount_expected_back = amount_expected_in_bins
                .into_values()
                .fold(ResourceIndexedData::default(), |acc, item| acc + item);

            let resources = indexmap! {
                bucket1.resource_address() => bucket1,
                bucket2.resource_address() => bucket2,
            };

            let fees = indexmap! {
                resource_x => max(
                    resources
                        .get(&resource_x)
                        .map(|bucket| bucket.amount())
                        .unwrap_or_default()
                        - total_amount_expected_back.resource_x,
                    Decimal::ZERO
                ),
                resource_y => max(
                    resources
                        .get(&resource_y)
                        .map(|bucket| bucket.amount())
                        .unwrap_or_default()
                        - total_amount_expected_back.resource_y,
                    Decimal::ZERO
                )
            };

            CloseLiquidityPositionOutput {
                resources,
                others: Default::default(),
                fees,
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

        // TODO: Final check prior to launch.
        fn exchange_specific_liquidity_receipt_data(
            &mut self,
        ) -> LiquidityReceiptExchangeSpecificData {
            LiquidityReceiptExchangeSpecificData {
                name: "Caviarnine Liquidity Receipt".to_owned(),
                description: "A receipt of liquidity provided to a Caviarnine pool through the Ignition protocol".to_owned(),
                key_image_url: Url::of("https://assets.caviarnine.com/tokens/resource_rdx1t5pyvlaas0ljxy0wytm5gvyamyv896m69njqdmm2stukr3xexc2up9.png"),
                redemption_url: Url::of("https://www.caviarnine.com/"),
            }
        }
    }
}

#[derive(ScryptoSbor, Debug, Clone)]
pub struct CaviarnineAdapterSpecificInformation {
    /// Stores the state of the bins that liquidity was contributed to when
    /// the position was first opened. Both the reserves and the amount that
    /// was contributed is stored. This information is later used to estimate
    /// how much fees was earned on the position.
    pub bin_information_after_position_opened: IndexMap<u32, BinInformation>,

    /// The price of the pool after the position was opened.
    pub price_after_position_opened: Price,
}

impl CaviarnineAdapterSpecificInformation {
    pub fn new(price: Price) -> Self {
        CaviarnineAdapterSpecificInformation {
            bin_information_after_position_opened: Default::default(),
            price_after_position_opened: price,
        }
    }

    pub fn contributions(&self) -> Vec<(u32, Decimal, Decimal)> {
        let mut contributions = self
            .bin_information_after_position_opened
            .iter()
            .map(|(bin, bin_information)| {
                (
                    *bin,
                    bin_information.contribution.resource_x,
                    bin_information.contribution.resource_y,
                )
            })
            .collect::<Vec<_>>();
        contributions.sort_by(|a, b| a.0.cmp(&b.0));
        contributions
    }
}

impl From<CaviarnineAdapterSpecificInformation> for AnyValue {
    fn from(value: CaviarnineAdapterSpecificInformation) -> Self {
        AnyValue::from_typed(&value).unwrap()
    }
}

#[derive(ScryptoSbor, Debug, Clone, Default)]
pub struct BinInformation {
    /// The reserves of resources x and y in the bin.
    pub reserves: ResourceIndexedData<Decimal>,
    /// The amount of resources contributed to the bin.
    pub contribution: ResourceIndexedData<Decimal>,
}

/// A type-safe way of representing two-resources without using a map that is
/// indexed by a resource address.
///
/// This guarantees that there is only two [`T`] fields, one for each resource
/// and that they're both of the same type. This also allows for addition and
/// subtraction over two [`ResourceIndexedData<T>`] where [`T`] is the same in
/// both.
#[derive(ScryptoSbor, Debug, Clone, Copy, Default)]
pub struct ResourceIndexedData<T> {
    pub resource_x: T,
    pub resource_y: T,
}

impl<T> Add<Self> for ResourceIndexedData<T>
where
    T: Add<T, Output = T>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            resource_x: self.resource_x + rhs.resource_x,
            resource_y: self.resource_y + rhs.resource_y,
        }
    }
}

impl<T> Sub<Self> for ResourceIndexedData<T>
where
    T: Sub<T, Output = T>,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            resource_x: self.resource_x - rhs.resource_x,
            resource_y: self.resource_y - rhs.resource_y,
        }
    }
}

impl<T> AddAssign for ResourceIndexedData<T>
where
    T: Add<T, Output = T> + Copy,
{
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl<T> SubAssign for ResourceIndexedData<T>
where
    T: Sub<T, Output = T> + Copy,
{
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

#[derive(Clone, Debug, Copy)]
pub enum Composition {
    EntirelyX,
    EntirelyY,
    Composite,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple_resource_indexed_data_addition_produces_expected_output() {
        // Arrange
        let a = ResourceIndexedData {
            resource_x: Decimal::ZERO,
            resource_y: dec!(200),
        };
        let b = ResourceIndexedData {
            resource_x: dec!(500),
            resource_y: dec!(12),
        };

        // Act
        let c = a + b;

        // Assert
        assert_eq!(c.resource_x, dec!(500));
        assert_eq!(c.resource_y, dec!(212));
    }

    #[test]
    fn simple_resource_indexed_data_add_assign_produces_expected_output() {
        // Arrange
        let mut a = ResourceIndexedData {
            resource_x: Decimal::ZERO,
            resource_y: dec!(200),
        };
        let b = ResourceIndexedData {
            resource_x: dec!(500),
            resource_y: dec!(12),
        };

        // Act
        a += b;

        // Assert
        assert_eq!(a.resource_x, dec!(500));
        assert_eq!(a.resource_y, dec!(212));
    }

    #[test]
    fn simple_resource_indexed_data_subtraction_produces_expected_output() {
        // Arrange
        let a = ResourceIndexedData {
            resource_x: Decimal::ZERO,
            resource_y: dec!(200),
        };
        let b = ResourceIndexedData {
            resource_x: dec!(500),
            resource_y: dec!(12),
        };

        // Act
        let c = a - b;

        // Assert
        assert_eq!(c.resource_x, dec!(-500));
        assert_eq!(c.resource_y, dec!(188));
    }

    #[test]
    fn simple_resource_indexed_data_sub_assign_produces_expected_output() {
        // Arrange
        let mut a = ResourceIndexedData {
            resource_x: Decimal::ZERO,
            resource_y: dec!(200),
        };
        let b = ResourceIndexedData {
            resource_x: dec!(500),
            resource_y: dec!(12),
        };

        // Act
        a -= b;

        // Assert
        assert_eq!(a.resource_x, dec!(-500));
        assert_eq!(a.resource_y, dec!(188));
    }
}
