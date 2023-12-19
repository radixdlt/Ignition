use std::time::Duration;

use humantime::format_duration;
use scrypto::prelude::*;

/// The data of the liquidity positions given to the users of Olympus.
#[derive(ScryptoSbor, NonFungibleData)]
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
    /// The address of the resource that the user contributed through the
    /// protocol.
    pub contributed_resource: ResourceAddress,

    /// The amount of the resource that the user contributed through the
    /// protocol.
    pub contributed_amount: Decimal,

    /// The amount of XRD that was contributed by the Olympus protocol to match
    /// the users contribution.
    pub matched_xrd_amount: Decimal,

    /// The date after which this liquidity position can be closed.
    //TODO: Wallet should display this as time and not unix timestamp.
    pub maturity_date: Instant,
}

impl LiquidityPosition {
    pub fn new(
        lockup_period: u32,
        contributed_resource: ResourceAddress,
        contributed_amount: Decimal,
        matched_xrd_amount: Decimal,
    ) -> Self {
        let string_lockup_period =
            format_duration(Duration::new(lockup_period as u64, 0)).to_string();
        let maturity_date = Clock::current_time_rounded_to_minutes()
            .add_seconds(lockup_period as i64)
            .unwrap();

        Self {
            name: "Olympus Liquidity Position".to_string(),
            description: "A non-fungible representing an open liquidity position in the Olympus protocol.".to_string(),
            key_image_url: Url::of("https://www.google.com"),
            lockup_period: string_lockup_period,
            redemption_url: Url::of("https://www.google.com"),
            contributed_resource,
            contributed_amount,
            maturity_date,
            matched_xrd_amount,
        }
    }
}
