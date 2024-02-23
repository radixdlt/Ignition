#![allow(clippy::new_without_default)]

mod blueprint_interface;
mod tick_math;
mod tick_selector;

pub use crate::blueprint_interface::*;
pub use crate::tick_math::*;
pub use crate::tick_selector::*;

use common::prelude::*;
use ports_interface::prelude::*;
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
            const $name: &'static str = concat!("[Caviarnine v1 Adapter v1]", " ", $item);
        )*
    };
}

define_error! {
    RESOURCE_DOES_NOT_BELONG_ERROR
        => "One or more of the resources do not belong to pool.";
    NO_ACTIVE_AMOUNTS_ERROR => "Pool has no active amounts.";
    NO_PRICE_ERROR => "Pool has no price.";
    OVERFLOW_ERROR => "Overflow error.";
}

macro_rules! pool {
    ($address: expr) => {
        $crate::blueprint_interface::CaviarnineV1PoolInterfaceScryptoStub::from(
            $address,
        )
    };
}

/// The total number of bins that we will be using on the left and the right
/// excluding the one in the middle. This number, in addition to the bin span
/// of the pool determines how much upside and downside we're covering. The
/// upside and downside we should cover is a business decision and its 20x up
/// and down. To calculate how much bins are needed (on each side) we can do
/// the following:
///
/// ```math
/// bins_required = floor(log(value = multiplier, base = 1.0005) / (2 * bin_span))
/// ```
///
/// In the case of a bin span of 50, the amount of bins we want to contribute to
/// on each side is 60 bins (60L and 60R). Therefore, the amount of bins to
/// contribute to is dependent on the bin span of the pool. However, in this
/// implementation we assume pools of a fixed bin span of 50 since we can't find
/// the number of bins required in Scrypto due to a missing implementation of a
/// function for computing the log.
pub const PREFERRED_TOTAL_NUMBER_OF_HIGHER_AND_LOWER_BINS: u32 = 30 * 2;

#[blueprint_with_traits]
#[types(ComponentAddress, PoolInformation, Decimal, PreciseDecimal)]
pub mod adapter {
    struct CaviarnineV1Adapter {
        /// A cache of the information of the pool, this is done so that we do
        /// not need to query the pool's information each time. Note: I would've
        /// preferred to keep the adapter completely stateless but it seems like
        /// we're pretty much forced to cache this data to get some fee gains.
        pool_information_cache:
            KeyValueStore<ComponentAddress, PoolInformation>,
    }

    impl CaviarnineV1Adapter {
        pub fn instantiate(
            metadata_init: MetadataInit,
            owner_role: OwnerRole,
            address_reservation: Option<GlobalAddressReservation>,
        ) -> Global<CaviarnineV1Adapter> {
            let address_reservation =
                address_reservation.unwrap_or_else(|| {
                    Runtime::allocate_component_address(BlueprintId {
                        package_address: Runtime::package_address(),
                        blueprint_name: Runtime::blueprint_name(),
                    })
                    .0
                });

            Self {
                pool_information_cache: KeyValueStore::new_with_registered_type(
                ),
            }
            .instantiate()
            .prepare_to_globalize(owner_role)
            .metadata(ModuleConfig {
                init: metadata_init,
                roles: Default::default(),
            })
            .with_address(address_reservation)
            .globalize()
        }

        pub fn preload_pool_information(
            &mut self,
            pool_address: ComponentAddress,
        ) -> PoolInformation {
            let pool = pool!(pool_address);
            let resource_address_x = pool.get_token_x_address();
            let resource_address_y = pool.get_token_y_address();
            let bin_span = pool.get_bin_span();

            let pool_information = PoolInformation {
                bin_span,
                resources: ResourceIndexedData {
                    resource_x: resource_address_x,
                    resource_y: resource_address_y,
                },
            };
            self.pool_information_cache
                .insert(pool_address, pool_information);
            pool_information
        }

        pub fn liquidity_receipt_data(
            // Does not depend on state, this is kept in case this is required
            // in the future for whatever reason.
            &mut self,
            global_id: NonFungibleGlobalId,
        ) -> LiquidityReceipt<CaviarnineV1AdapterSpecificInformation> {
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
                .as_typed::<CaviarnineV1AdapterSpecificInformation>()
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

        // This function is here to optimize the adapter for fees. Previously,
        // getting the price and the active tick were two separate invocations
        // which proved to be rather costly. Therefore, since we typically need
        // both pieces of data, this function makes an invocation for the price
        // and then calculates the active tick from it. The relationship between
        // the price and tick is: `p(t) = 1.0005 ^ (2*(t - 27000))`.
        pub fn price_and_active_tick(
            &mut self,
            pool_address: ComponentAddress,
            pool_information: Option<PoolInformation>,
        ) -> Option<(Decimal, u32)> {
            let pool = pool!(pool_address);
            let PoolInformation { bin_span, .. } = pool_information
                .unwrap_or_else(|| self.get_pool_information(pool_address));
            let price = pool.get_price()?;
            // The following division and multiplication by the bin span rounds
            // the calculated tick down to the nearest multiple of the bin span.
            // This is because in Caviarnine valid ticks depend on the pool's
            // bin span and there only exist valid ticks at multiples of the bin
            // span. Alternatively, you can think of the following bit of code
            // as active_tick = active_tick - active_tick % bin_span.
            let active_tick = spot_to_tick(price)
                .and_then(|value| value.checked_div(bin_span))
                .and_then(|value| value.checked_mul(bin_span))?;
            Some((price, active_tick))
        }

        fn get_pool_information(
            &mut self,
            pool_address: ComponentAddress,
        ) -> PoolInformation {
            let entry = self.pool_information_cache.get(&pool_address);
            if let Some(entry) = entry {
                *entry
            } else {
                drop(entry);
                self.preload_pool_information(pool_address)
            }
        }
    }

    impl PoolAdapterInterfaceTrait for CaviarnineV1Adapter {
        fn open_liquidity_position(
            &mut self,
            pool_address: ComponentAddress,
            buckets: (Bucket, Bucket),
        ) -> OpenLiquidityPositionOutput {
            let mut pool = pool!(pool_address);

            // Split the two buckets into bucket_x and bucket_y in the same way
            // that they're defined in the pool itself.
            let pool_information @ PoolInformation {
                bin_span,
                resources:
                    ResourceIndexedData {
                        resource_x: resource_address_x,
                        resource_y: resource_address_y,
                    },
            } = self.get_pool_information(pool_address);

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
            let (price, active_tick) = self
                .price_and_active_tick(pool_address, Some(pool_information))
                .expect(NO_PRICE_ERROR);

            let SelectedTicks {
                higher_ticks,
                lower_ticks,
                ..
            } = SelectedTicks::select(
                active_tick,
                bin_span,
                PREFERRED_TOTAL_NUMBER_OF_HIGHER_AND_LOWER_BINS,
            );

            // This comment was quickly going out of sync with the constant that
            // is defined above of the number of bins to contribute to. Thus, to
            // make this simple, let's say that the number of bins to contribute
            // to is defined as `m` such that we're contributing to `m` bins to
            // the left and `m` to the right.
            //
            // Determine the amount of resources that we will add to each of the
            // bins. We have m on the left and m on the right. But, we also
            // have the active bin that is composed of both x and y. So, this
            // be like contributing to m.x and m.y bins where x = 1-y. X here
            // is the percentage of resources x in the active bin.
            let (amount_in_active_bin_x, amount_in_active_bin_y) =
                pool.get_active_amounts().expect(NO_ACTIVE_AMOUNTS_ERROR);

            let percentage_in_active_bin_x = amount_in_active_bin_x
                .checked_mul(price)
                .and_then(|value| {
                    value.checked_div(
                        amount_in_active_bin_x
                            .checked_mul(price)?
                            .checked_add(amount_in_active_bin_y)?,
                    )
                })
                .expect(OVERFLOW_ERROR);
            let percentage_in_active_bin_y = Decimal::one()
                .checked_sub(percentage_in_active_bin_x)
                .expect(OVERFLOW_ERROR);

            // In here, we decide the amount x by the number of higher bins plus
            // the percentage of the x in the currently active bin since the
            // pool starting from the current price and upward is
            // entirely composed of X. Similarly, we divide amount_y
            // by the number of lower positions plus the percentage
            // of y in the active bin since the pool starting from
            // the current price and downward is composed just of y.
            let position_amount_x = Decimal::from(higher_ticks.len() as u32)
                .checked_add(percentage_in_active_bin_x)
                .and_then(|value| amount_x.checked_div(value))
                .expect(OVERFLOW_ERROR);
            let position_amount_y = Decimal::from(lower_ticks.len() as u32)
                .checked_add(percentage_in_active_bin_y)
                .and_then(|value| amount_y.checked_div(value))
                .expect(OVERFLOW_ERROR);

            let position_amount_x_in_y =
                position_amount_x.checked_mul(price).expect(OVERFLOW_ERROR);
            let (position_amount_x, position_amount_y) =
                if position_amount_x_in_y > position_amount_y {
                    let position_amount_y_in_x = position_amount_y
                        .checked_div(price)
                        .expect(OVERFLOW_ERROR);
                    (position_amount_y_in_x, position_amount_y)
                } else {
                    (position_amount_x, position_amount_x_in_y)
                };

            let mut positions = vec![(
                active_tick,
                position_amount_x
                    .checked_mul(percentage_in_active_bin_x)
                    .expect(OVERFLOW_ERROR),
                position_amount_y
                    .checked_mul(percentage_in_active_bin_y)
                    .expect(OVERFLOW_ERROR),
            )];
            positions.extend(
                lower_ticks
                    .into_iter()
                    .map(|bin_id| (bin_id, dec!(0), position_amount_y)),
            );
            positions.extend(
                higher_ticks
                    .into_iter()
                    .map(|bin_id| (bin_id, position_amount_x, dec!(0))),
            );

            let (receipt, change_x, change_y) =
                pool.add_liquidity(bucket_x, bucket_y, positions);

            let receipt_global_id = {
                let resource_address = receipt.resource_address();
                let local_id =
                    receipt.as_non_fungible().non_fungible_local_id();
                NonFungibleGlobalId::new(resource_address, local_id)
            };

            let adapter_specific_information =
                CaviarnineV1AdapterSpecificInformation {
                    bin_contributions: pool
                        .get_redemption_bin_values(
                            receipt_global_id.local_id().clone(),
                        )
                        .into_iter()
                        .map(|(tick, amount_x, amount_y)| {
                            (
                                tick,
                                ResourceIndexedData {
                                    resource_x: amount_x,
                                    resource_y: amount_y,
                                },
                            )
                        })
                        .collect(),
                    liquidity_receipt_non_fungible_global_id: receipt_global_id,
                    price_when_position_was_opened: price,
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
            let mut pool = pool!(pool_address);
            let pool_information @ PoolInformation {
                bin_span,
                resources:
                    ResourceIndexedData {
                        resource_x,
                        resource_y,
                    },
            } = self.get_pool_information(pool_address);
            let (current_price, active_tick) = self
                .price_and_active_tick(pool_address, Some(pool_information))
                .expect(NO_PRICE_ERROR);

            // Decoding the adapter specific information as the type we expect
            // it to be.
            let CaviarnineV1AdapterSpecificInformation {
                bin_contributions,
                price_when_position_was_opened,
                ..
            } = adapter_specific_information.as_typed().unwrap();

            let (bucket_x, bucket_y) = pool.remove_liquidity(pool_units);

            let fees = {
                // Calculate how much we expect to find in the bins at this
                // price.
                let expected_bin_amounts =
                    calculate_bin_amounts_due_to_price_action(
                        bin_contributions,
                        current_price,
                        price_when_position_was_opened,
                        active_tick,
                        bin_span,
                    )
                    .expect(OVERFLOW_ERROR);

                // Based on the calculated bin amounts calculate how much we
                // should expect to get back if we close the liquidity position
                // by just summing them all up.
                let expected_amount_back = expected_bin_amounts
                    .into_iter()
                    .map(|(_, amount_in_bin)| amount_in_bin)
                    .fold(ResourceIndexedData::default(), |acc, item| {
                        acc.checked_add(item).expect(OVERFLOW_ERROR)
                    });

                // The difference between the amount we got back and the amount
                // calculated up above is the fees.
                indexmap! {
                    resource_x => max(
                        bucket_x.amount()
                            .checked_sub(expected_amount_back.resource_x)
                            .expect(OVERFLOW_ERROR),
                        Decimal::ZERO
                    ),
                    resource_y => max(
                        bucket_y.amount()
                            .checked_sub(expected_amount_back.resource_y)
                            .expect(OVERFLOW_ERROR),
                        Decimal::ZERO
                    )
                }
            };

            CloseLiquidityPositionOutput {
                resources: indexmap! {
                    resource_x => bucket_x,
                    resource_y => bucket_y,
                },
                others: Default::default(),
                fees,
            }
        }

        fn price(&mut self, pool_address: ComponentAddress) -> Price {
            let pool = pool!(pool_address);

            let PoolInformation {
                resources:
                    ResourceIndexedData {
                        resource_x: resource_address_x,
                        resource_y: resource_address_y,
                    },
                ..
            } = self.get_pool_information(pool_address);
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
            let pool = pool!(pool_address);

            (pool.get_token_x_address(), pool.get_token_y_address())
        }
    }
}

#[derive(ScryptoSbor, Debug, Clone, Copy)]
pub struct PoolInformation {
    pub bin_span: u32,
    pub resources: ResourceIndexedData<ResourceAddress>,
}

#[derive(ScryptoSbor, Debug, Clone)]
pub struct CaviarnineV1AdapterSpecificInformation {
    /// Stores how much was contributed to the bin.
    pub bin_contributions: IndexMap<u32, ResourceIndexedData<Decimal>>,

    /// The price in the pool when the position was opened.
    pub price_when_position_was_opened: Decimal,

    /// Stores the non-fungible global id of the liquidity receipt.
    pub liquidity_receipt_non_fungible_global_id: NonFungibleGlobalId,
}

impl CaviarnineV1AdapterSpecificInformation {
    pub fn new(
        liquidity_receipt_non_fungible_global_id: NonFungibleGlobalId,
        price_when_position_was_opened: Decimal,
    ) -> Self {
        CaviarnineV1AdapterSpecificInformation {
            bin_contributions: Default::default(),
            liquidity_receipt_non_fungible_global_id,
            price_when_position_was_opened,
        }
    }

    pub fn contributions(&self) -> Vec<(u32, Decimal, Decimal)> {
        let mut contributions = self
            .bin_contributions
            .iter()
            .map(|(bin, contribution)| {
                (*bin, contribution.resource_x, contribution.resource_y)
            })
            .collect::<Vec<_>>();
        contributions.sort_by(|a, b| a.0.cmp(&b.0));
        contributions
    }
}

impl From<CaviarnineV1AdapterSpecificInformation> for AnyValue {
    fn from(value: CaviarnineV1AdapterSpecificInformation) -> Self {
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
    Self: CheckedAdd<Self, Output = Self>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.checked_add(rhs).unwrap()
    }
}

impl<T> Sub<Self> for ResourceIndexedData<T>
where
    Self: CheckedSub<Self, Output = Self>,
{
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.checked_sub(rhs).unwrap()
    }
}

impl<T> CheckedAdd<Self> for ResourceIndexedData<T>
where
    T: CheckedAdd<T, Output = T>,
{
    type Output = Self;

    fn checked_add(self, rhs: Self) -> Option<Self::Output>
    where
        Self: Sized,
    {
        Some(Self {
            resource_x: self.resource_x.checked_add(rhs.resource_x)?,
            resource_y: self.resource_y.checked_add(rhs.resource_y)?,
        })
    }
}

impl<T> CheckedSub<Self> for ResourceIndexedData<T>
where
    T: CheckedSub<T, Output = T>,
{
    type Output = Self;

    fn checked_sub(self, rhs: Self) -> Option<Self::Output>
    where
        Self: Sized,
    {
        Some(Self {
            resource_x: self.resource_x.checked_sub(rhs.resource_x)?,
            resource_y: self.resource_y.checked_sub(rhs.resource_y)?,
        })
    }
}

#[derive(Clone, Debug, Copy)]
pub enum Composition {
    EntirelyX,
    EntirelyY,
    Composite,
}

/// This method calculates the liquidity or the `l` of each bin based on the
/// reserves in the bin and the lower and upper ticks of the bin.
pub fn calculate_liquidity(
    bin_reserves: ResourceIndexedData<Decimal>,
    lower_price: Decimal,
    upper_price: Decimal,
) -> Option<Decimal> {
    let ResourceIndexedData {
        resource_x: reserves_x,
        resource_y: reserves_y,
    } = bin_reserves;

    let reserves_x = PreciseDecimal::from(reserves_x);
    let reserves_y = PreciseDecimal::from(reserves_y);
    // TODO: Consider caching the the sqrt.
    let lower_price_sqrt = PreciseDecimal::from(lower_price).checked_sqrt()?;
    let upper_price_sqrt = PreciseDecimal::from(upper_price).checked_sqrt()?;

    // Solve quadratic for liquidity
    let a = lower_price_sqrt
        .checked_div(upper_price_sqrt)?
        .checked_sub(PreciseDecimal::ONE)?;
    let b = reserves_x
        .checked_mul(lower_price_sqrt)?
        .checked_add(reserves_y.checked_div(upper_price_sqrt)?)?;
    let c = reserves_x.checked_mul(reserves_y)?;

    let nominator = b.checked_neg()?.checked_sub(
        b.checked_powi(2)?
            .checked_sub(pdec!(4).checked_mul(a)?.checked_mul(c)?)?
            .checked_sqrt()?,
    )?;
    let denominator = pdec!(2).checked_mul(a)?;

    nominator
        .checked_div(denominator)
        .and_then(|value| Decimal::try_from(value).ok())
}

/// Given the amount of assets that used to be in the bin and a certain change
/// in price, this function calculates the new composition of the bins based on
/// price action alone.
fn calculate_bin_amounts_due_to_price_action(
    bin_amounts: IndexMap<u32, ResourceIndexedData<Decimal>>,
    current_price: Decimal,
    price_when_position_was_opened: Decimal,
    active_tick: u32,
    bin_span: u32,
) -> Option<Vec<(u32, ResourceIndexedData<Decimal>)>> {
    bin_amounts
        .into_iter()
        .map(|(tick, bin_amount_at_opening_time)| {
            // Calculating the lower and upper prices of the bin based on the
            // the starting tick and the bin span.
            let lower_tick = tick;
            let upper_tick = tick.checked_add(bin_span)?;

            let bin_lower_price = tick_to_spot(lower_tick)?;
            let bin_upper_price = tick_to_spot(upper_tick)?;

            let bin_composition_when_position_opened = match (
                bin_amount_at_opening_time.resource_x.is_zero(),
                bin_amount_at_opening_time.resource_y.is_zero(),
            ) {
                (true, true) => return None,
                (true, false) => Composition::EntirelyY,
                (false, true) => Composition::EntirelyX,
                (false, false) => Composition::Composite,
            };

            // Determine what we expect the composition of this bin to be based
            // on the current active tick.
            let expected_bin_composition_now = match tick.cmp(&active_tick) {
                // Case A: The current price is inside this bin. Since we are
                // the current active bin then it's expected that this bin has
                // both X and Y assets.
                Ordering::Equal => Composition::Composite,
                // Case B: The current price of the pool is greater than the
                // upper bound of the bin. We're outside of that range and there
                // should only be Y assets in the bin.
                Ordering::Less => Composition::EntirelyY,
                // Case C: The current price of the pool is smaller than the
                // lower bound of the bin. We're outside of that range and there
                // should only be X assets in the bin.
                Ordering::Greater => Composition::EntirelyX,
            };

            let new_contents = match (
                bin_composition_when_position_opened,
                expected_bin_composition_now,
            ) {
                // The bin was entirely made of X and is still the same. Thus,
                // this bin "has not been touched" and should in theory contain
                // the same amount as before. Difference found can therefore be
                // attributed to fees. The other case is when the bin was made
                // of up just Y and still is just Y.
                (Composition::EntirelyX, Composition::EntirelyX) => Some((
                    bin_amount_at_opening_time.resource_x,
                    bin_amount_at_opening_time.resource_y,
                )),
                (Composition::EntirelyY, Composition::EntirelyY) => Some((
                    bin_amount_at_opening_time.resource_x,
                    bin_amount_at_opening_time.resource_y,
                )),
                // The bin was entirely made up of one asset and is now made up
                // of another. We therefore want to do a full "swap" of that
                // amount. For this calculation we use y = sqrt(pa * pb) * x.
                // We can also use the equation used in the later cases but it
                // is very expensive to run.
                (Composition::EntirelyX, Composition::EntirelyY) => Some((
                    dec!(0),
                    bin_lower_price
                        .checked_mul(bin_upper_price)
                        .and_then(|value| value.checked_sqrt())
                        .and_then(|value| {
                            value.checked_mul(
                                bin_amount_at_opening_time.resource_x,
                            )
                        })
                        .expect(OVERFLOW_ERROR),
                )),
                (Composition::EntirelyY, Composition::EntirelyX) => Some((
                    bin_lower_price
                        .checked_mul(bin_upper_price)
                        .and_then(|value| value.checked_sqrt())
                        .and_then(|value| {
                            bin_amount_at_opening_time
                                .resource_y
                                .checked_div(value)
                        })
                        .expect(OVERFLOW_ERROR),
                    dec!(0),
                )),
                // The bin was entirely made up of one of the assets and
                // is now made up of both of them.
                (Composition::EntirelyX, Composition::Composite) => {
                    let (starting_price, ending_price) =
                        (bin_lower_price, current_price);
                    calculate_bin_amount_using_liquidity(
                        bin_amount_at_opening_time,
                        bin_lower_price,
                        bin_upper_price,
                        starting_price,
                        ending_price,
                    )
                }
                (Composition::EntirelyY, Composition::Composite) => {
                    let (starting_price, ending_price) =
                        (bin_upper_price, current_price);
                    calculate_bin_amount_using_liquidity(
                        bin_amount_at_opening_time,
                        bin_lower_price,
                        bin_upper_price,
                        starting_price,
                        ending_price,
                    )
                }
                // The bin was made up of both assets and is now just made
                // up of one of them.
                (Composition::Composite, Composition::EntirelyX) => {
                    let (starting_price, ending_price) =
                        (price_when_position_was_opened, bin_lower_price);
                    calculate_bin_amount_using_liquidity(
                        bin_amount_at_opening_time,
                        bin_lower_price,
                        bin_upper_price,
                        starting_price,
                        ending_price,
                    )
                }
                (Composition::Composite, Composition::EntirelyY) => {
                    let (starting_price, ending_price) =
                        (price_when_position_was_opened, bin_upper_price);
                    calculate_bin_amount_using_liquidity(
                        bin_amount_at_opening_time,
                        bin_lower_price,
                        bin_upper_price,
                        starting_price,
                        ending_price,
                    )
                }
                // The bin was made up of both assets and is still made up
                // of both assets.
                (Composition::Composite, Composition::Composite) => {
                    let (starting_price, ending_price) =
                        (price_when_position_was_opened, current_price);
                    calculate_bin_amount_using_liquidity(
                        bin_amount_at_opening_time,
                        bin_lower_price,
                        bin_upper_price,
                        starting_price,
                        ending_price,
                    )
                }
            };

            new_contents.map(|contents| {
                (
                    tick,
                    ResourceIndexedData {
                        resource_x: contents.0,
                        resource_y: contents.1,
                    },
                )
            })
        })
        .collect()
}

fn calculate_bin_amount_using_liquidity(
    bin_amount: ResourceIndexedData<Decimal>,
    bin_lower_price: Decimal,
    bin_upper_price: Decimal,
    starting_price: Decimal,
    ending_price: Decimal,
) -> Option<(Decimal, Decimal)> {
    let liquidity =
        calculate_liquidity(bin_amount, bin_lower_price, bin_upper_price)?;

    let change_x = liquidity.checked_mul(
        Decimal::ONE
            .checked_div(ending_price.checked_sqrt()?)?
            .checked_sub(
                Decimal::ONE.checked_div(starting_price.checked_sqrt()?)?,
            )?,
    )?;
    let change_y = liquidity.checked_mul(
        ending_price
            .checked_sqrt()?
            .checked_sub(starting_price.checked_sqrt()?)?,
    )?;

    let new_x =
        max(bin_amount.resource_x.checked_add(change_x)?, Decimal::ZERO);
    let new_y =
        max(bin_amount.resource_y.checked_add(change_y)?, Decimal::ZERO);

    Some((new_x, new_y))
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
}
