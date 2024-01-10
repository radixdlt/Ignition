use crate::*;
use scrypto::prelude::*;

/// The data of the liquidity positions given to the users of Ignition.
#[derive(ScryptoSbor, Clone, Debug, PartialEq, Eq, NonFungibleData)]
pub struct LiquidityPosition {
    /* Metadata/NonFungibleData standard */
    pub name: String,
    pub description: String,
    pub key_image_url: Url,

    /* Display Data - Just for wallet display, no logic depends on this. */
    /// A string of the lockup period of the liquidity provided through the
    /// protocol (e.g., "6 Months").
    pub lockup_period: String,

    /// A url linking to where we redeem
    pub redemption_url: Url,

    /* Application data */
    /// The address of the pool that the resources were contributed to and that
    /// they must be redeemed from.
    pub pool: ComponentAddress,

    /// The value of the contribution made in USD.
    pub contribution_usd_value: Decimal,

    /// The address of the resource that the user contributed through the
    /// protocol.
    pub contributed_resource: ResourceAddress,

    /// The amount of the resource that the user contributed through the
    /// protocol.
    pub contributed_amount: Decimal,

    /// The amount of XRD that was contributed by the Ignition protocol to match
    /// the users contribution.
    pub matched_xrd_amount: Decimal,

    /// The date after which this liquidity position can be closed.
    pub maturity_date: Instant,

    /// An adapter-specific field containing information on the opening of the
    /// liquidity position. This typically contains data that is to be used
    /// later by the adapter when the position is to be closed.
    pub adapter_specific_data: AnyScryptoValue,
}

impl LiquidityPosition {
    pub fn new(
        lockup_period: LockupPeriod,
        pool: ComponentAddress,
        contribution_usd_value: Decimal,
        contributed_resource: ResourceAddress,
        contributed_amount: Decimal,
        matched_xrd_amount: Decimal,
        adapter_specific_data: AnyScryptoValue,
    ) -> Self {
        let maturity_date = Clock::current_time_rounded_to_minutes()
            .add_seconds(*lockup_period.seconds() as i64)
            .unwrap();

        Self {
            name: "Ignition Liquidity Position".to_string(),
            description: "A non-fungible representing an open liquidity position in the Ignition protocol.".to_string(),
            key_image_url: Url::of("https://www.google.com"),
            lockup_period: lockup_period.to_string(),
            redemption_url: Url::of("https://www.google.com"),
            contribution_usd_value,
            pool,
            contributed_resource,
            contributed_amount,
            maturity_date,
            matched_xrd_amount,
            adapter_specific_data
        }
    }
}
