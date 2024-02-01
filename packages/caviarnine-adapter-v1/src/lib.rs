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
            const $name: &'static str = concat!("[Caviarnine Adapter]", " ", $item);
        )*
    };
}

define_error! {
    RESOURCE_DOES_NOT_BELONG_ERROR
        => "One or more of the resources do not belong to pool.";
    NO_ACTIVE_BIN_ERROR => "Pool has no active bin.";
    NO_ACTIVE_AMOUNTS_ERROR => "Pool has no active amounts.";
    NO_PRICE_ERROR => "Pool has no price.";
    OVERFLOW_ERROR => "Overflow error.";
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
        ) -> CaviarninePoolInterfaceScryptoStub {
            CaviarninePoolInterfaceScryptoStub::from(component_address)
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

            let receipt_global_id = {
                let resource_address = receipt.resource_address();
                let local_id =
                    receipt.as_non_fungible().non_fungible_local_id();
                NonFungibleGlobalId::new(resource_address, local_id)
            };

            let adapter_specific_information =
                CaviarnineAdapterSpecificInformation {
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
            let mut pool = Self::pool(pool_address);
            let bin_span = pool.get_bin_span();
            let resource_x = pool.get_token_x_address();
            let resource_y = pool.get_token_y_address();
            let active_bin = pool.get_active_tick().expect(NO_ACTIVE_BIN_ERROR);

            // Decoding the adapter specific information as the type we
            // expect it to be.
            let CaviarnineAdapterSpecificInformation {
                bin_contributions,
                price_when_position_was_opened,
                ..
            } = adapter_specific_information.as_typed().unwrap();

            let (bucket1, bucket2) = pool.remove_liquidity(pool_units);
            let resources = indexmap! {
                bucket1.resource_address() => bucket1,
                bucket2.resource_address() => bucket2,
            };

            let fees = {
                // Calculate how much we expect to find in the bins at this
                // price.
                let expected_bin_amounts =
                    calculate_bin_amounts_due_to_price_action(
                        &bin_contributions
                            .into_iter()
                            .map(|value| (value.0, value.1))
                            .collect::<Vec<_>>(),
                        pool.get_price().expect(NO_PRICE_ERROR),
                        price_when_position_was_opened,
                        active_bin,
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
                        acc + item
                    });

                // The difference between the amount we got back and the amount
                // calculated up above is the fees.
                indexmap! {
                    resource_x => max(
                        resources
                            .get(&resource_x)
                            .map(|bucket| bucket.amount())
                            .unwrap_or_default()
                            - expected_amount_back.resource_x,
                        Decimal::ZERO
                    ),
                    resource_y => max(
                        resources
                            .get(&resource_y)
                            .map(|bucket| bucket.amount())
                            .unwrap_or_default()
                            - expected_amount_back.resource_y,
                        Decimal::ZERO
                    )
                }
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
    }
}

#[derive(ScryptoSbor, Debug, Clone)]
pub struct CaviarnineAdapterSpecificInformation {
    /// Stores how much was contributed to the bin.
    pub bin_contributions: IndexMap<u32, ResourceIndexedData<Decimal>>,

    /// The price in the pool when the position was opened.
    pub price_when_position_was_opened: Decimal,

    /// Stores the non-fungible global id of the liquidity receipt.
    pub liquidity_receipt_non_fungible_global_id: NonFungibleGlobalId,
}

impl CaviarnineAdapterSpecificInformation {
    pub fn new(
        liquidity_receipt_non_fungible_global_id: NonFungibleGlobalId,
        price_when_position_was_opened: Decimal,
    ) -> Self {
        CaviarnineAdapterSpecificInformation {
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
// TODO: Write unit tests for this function in isolation of everything else.
fn calculate_bin_amounts_due_to_price_action(
    bin_amounts: &[(u32, ResourceIndexedData<Decimal>)],
    current_price: Decimal,
    price_when_position_was_opened: Decimal,
    active_bin: u32,
    bin_span: u32,
) -> Option<Vec<(u32, ResourceIndexedData<Decimal>)>> {
    bin_amounts.iter().copied().map(
        |(tick, bin_amount)| -> Option<(u32, ResourceIndexedData<Decimal>)> {
            // Calculating the lower and upper prices of the bin based on the
            // the starting tick and the bin span.
            let lower_tick = tick;
            let upper_tick = tick.checked_add(bin_span)?;

            let bin_lower_price = tick_to_spot(lower_tick)?;
            let bin_upper_price = tick_to_spot(upper_tick)?;

            // Determine the starting and ending prices to use in the math.

            let (starting_price, ending_price) = {
                let bin_composition_when_position_opened = match (
                    bin_amount.resource_x == Decimal::ZERO,
                    bin_amount.resource_y == Decimal::ZERO,
                ) {
                    // TODO: I think that this is impossible? I believe that
                    // it is impossible since we only capture the bins that
                    // we have contributed to and THEN store the reserves in
                    // the bin. If we contributed to it, how come its empty
                    // of both resources?
                    //
                    // If this is somehow possible, what do we want to do
                    // in this case?
                    (true, true) => return None,
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

                match (
                    bin_composition_when_position_opened,
                    expected_bin_composition_now,
                ) {
                    // The bin was entirely made of X and is still the same. We
                    // have not touched it. The starting and ending price of the
                    // "swap" is the same, we didn't go through this bin.
                    (Composition::EntirelyX, Composition::EntirelyX)
                        => (bin_lower_price, bin_lower_price),
                    (Composition::EntirelyY, Composition::EntirelyY)
                        => (bin_upper_price, bin_upper_price),
                    // The bin was entirely made up of one asset and is now made
                    // up of another. 
                    (Composition::EntirelyX, Composition::EntirelyY)
                        => (bin_lower_price, bin_upper_price),
                    (Composition::EntirelyY, Composition::EntirelyX)
                        => (bin_upper_price, bin_lower_price),
                    // The bin was entirely made up of one of the assets and 
                    // is now made up of both of them. 
                    (Composition::EntirelyX, Composition::Composite)
                        => (bin_lower_price, current_price),
                    (Composition::EntirelyY, Composition::Composite)
                        => (bin_upper_price, current_price),
                    // The bin was made up of both assets and is now just made 
                    // up of one of them.
                    (Composition::Composite, Composition::EntirelyX)
                        => (price_when_position_was_opened, bin_lower_price),
                    (Composition::Composite, Composition::EntirelyY)
                        => (price_when_position_was_opened, bin_upper_price),
                    // The bin was made up of both assets and is still made up
                    // of both assets.
                    (Composition::Composite, Composition::Composite)
                        => (price_when_position_was_opened, current_price),
                }
            };

            let liquidity = calculate_liquidity(
                bin_amount,
                bin_lower_price,
                bin_upper_price,
            )?;

            let change_x = liquidity.checked_mul(
                Decimal::ONE
                    .checked_div(ending_price.checked_sqrt()?)?
                    .checked_sub(
                        Decimal::ONE
                            .checked_div(starting_price.checked_sqrt()?)?,
                    )?,
            )?;
            let change_y = liquidity.checked_mul(
                ending_price
                    .checked_sqrt()?
                    .checked_sub(starting_price.checked_sqrt()?)?,
            )?;

            let new_x = max(bin_amount.resource_x.checked_add(change_x)?, Decimal::ZERO);
            let new_y = max(bin_amount.resource_y.checked_add(change_y)?, Decimal::ZERO);

            Some((
                tick,
                ResourceIndexedData {
                    resource_x: new_x,
                    resource_y: new_y,
                },
            ))
        },
    )
    .collect()
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
