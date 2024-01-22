use crate::*;
use adapters_interface::prelude::*;
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
    /// The pool that the resources were contributed to.
    pub pool_address: ComponentAddress,

    /// The address of the resource that the user contributed through the
    /// protocol.
    pub user_resource_address: ResourceAddress,

    /// The amount of the resource that the user contributed through the
    /// protocol.
    pub user_contribution_amount: Decimal,

    /// The volatility classification of the user resource at the time when the
    /// liquidity position was opened. This will be used to later deposit any
    /// protocol assets back into the same vault.
    pub user_resource_volatility_classification: Volatility,

    /// The amount of XRD that was contributed by the Ignition protocol to match
    /// the users contribution.
    pub protocol_contribution_amount: Decimal,

    /// The date after which this liquidity position can be closed.
    pub maturity_date: Instant,
}

impl LiquidityReceipt {
    pub fn new(
        exchange_specific: LiquidityReceiptExchangeSpecificData,
        lockup_period: LockupPeriod,
        pool_address: ComponentAddress,
        user_resource_address: ResourceAddress,
        user_contribution_amount: Decimal,
        user_volatility_classification: Volatility,
        protocol_contribution_amount: Decimal,
    ) -> Self {
        let maturity_date = Clock::current_time_rounded_to_minutes()
            .add_seconds(*lockup_period.seconds() as i64)
            .unwrap();

        let LiquidityReceiptExchangeSpecificData {
            name,
            description,
            key_image_url,
            redemption_url,
        } = exchange_specific;

        Self {
            name,
            description,
            key_image_url,
            redemption_url,
            lockup_period: lockup_period.to_string(),
            pool_address,
            user_resource_address,
            user_contribution_amount,
            maturity_date,
            protocol_contribution_amount,
            user_resource_volatility_classification:
                user_volatility_classification,
        }
    }
}
