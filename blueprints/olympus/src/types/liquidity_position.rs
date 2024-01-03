use crate::*;
use scrypto::prelude::*;

/// The data of the liquidity positions given to the users of Olympus.
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
    pub maturity_date: Instant,

    /// A struct of information on the state of the pool at the time when the
    /// position was opened. The majority of the information here will be used for
    /// calculations later on.
    pub state_after_position_was_opened: StateAfterPositionWasOpened,
}

#[derive(
    ScryptoSbor, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
/// A struct of information on the state of the pool at the time when the
/// position was opened. The majority of the information here will be used for
/// calculations later on.
pub struct StateAfterPositionWasOpened {
    /// The value of the pool's `k` at the time when the liquidity position was
    /// opened. This is used for the calculation of the trading fees when we
    /// close the liquidity position.
    pub k: PreciseDecimal,

    /// The share of the user in the pool at the time of opening the liquidity
    /// position. This is used for the calculation of the trading fees when we
    /// close the liquidity position. This is a [`Percent`] that is in the range
    /// [0, 1].
    pub user_share: Percent,
}

impl LiquidityPosition {
    pub fn new(
        lockup_period: LockupPeriod,
        contributed_resource: ResourceAddress,
        contributed_amount: Decimal,
        matched_xrd_amount: Decimal,
        state_after_position_was_opened: StateAfterPositionWasOpened,
    ) -> Self {
        let maturity_date = Clock::current_time_rounded_to_minutes()
            .add_seconds(*lockup_period.seconds() as i64)
            .unwrap();

        Self {
            name: "Olympus Liquidity Position".to_string(),
            description: "A non-fungible representing an open liquidity position in the Olympus protocol.".to_string(),
            key_image_url: Url::of("https://www.google.com"),
            lockup_period: lockup_period.to_string(),
            redemption_url: Url::of("https://www.google.com"),
            contributed_resource,
            contributed_amount,
            maturity_date,
            matched_xrd_amount,
            state_after_position_was_opened
        }
    }
}
