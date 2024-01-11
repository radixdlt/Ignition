use crate::*;
use scrypto::prelude::*;

/// The data of the liquidity positions given to the users of Ignition.
#[derive(ScryptoSbor, Clone, Debug, PartialEq, Eq, NonFungibleData)]
pub struct LiquidityReceipt {
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
    pub user_resource_address: ResourceAddress,

    /// The amount of the resource that the user contributed through the
    /// protocol.
    pub user_contribution_amount: Decimal,

    /// The amount of XRD that was contributed by the Ignition protocol to match
    /// the users contribution.
    pub protocol_contribution_amount: Decimal,

    /// The date after which this liquidity position can be closed.
    pub maturity_date: Instant,
}

impl LiquidityReceipt {
    pub fn new(
        lockup_period: LockupPeriod,
        user_resource_address: ResourceAddress,
        user_contribution_amount: Decimal,
        protocol_contribution_amount: Decimal,
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
            user_resource_address,
            user_contribution_amount,
            maturity_date,
            protocol_contribution_amount,
        }
    }
}
