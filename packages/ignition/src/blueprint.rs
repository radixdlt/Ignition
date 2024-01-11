//! This module implements the main project ignition blueprint and protocol.
//!
//! In simple terms, project ignition allows for users to provide one side of
//! liquidity and for itself to provide the other side of the liquidity. The
//! protocol is not quite made to be profit-generating, its main purpose is to
//! incentivize people to provide liquidity by providing users with a number of
//! benefits:
//!
//! * User's contribution is doubled in value; Ignition will contribute the
//! other side of the liquidity.
//! * Users get some percentage of rewards upfront.
//! * Users have impermanent loss protection and in most cases are guaranteed
//! to withdraw out the same amount of tokens that they put in plus fees earned
//! on their position.
//!
//! This makes Ignition a perfect incentive for users who already own an amount
//! of some of the supported tokens and who wish to provide liquidity with very
//! low downside, upfront rewards, and impermanent loss protection.
//!
//! The user locks their tokens for some period of time allowed by the protocol
//! and based on that they get some amount of upfront rewards. The longer the
//! lockup period is, the higher the rewards are. When the period is over, the
//! protocol will try to provide the user with the same amount of tokens that
//! they put in plus any trading fees earned in the process (on their asset).
//! If that can't be given, then the protocol will try to provide the user of
//! as much of the protocol's asset as possible to make them whole in terms of
//! value.
//!
//! In Ignition, the term "protocol's asset" refers to the asset that Ignition
//! has and that the protocol is willing to lend out to users when they wish to
//! provide liquidity. The term "user asset" refers to the asset or resource
//! that was provided by the user. So, the protocol and user assets are the two
//! sides of the liquidity that go into a liquidity pool, which name is used
//! depends on their source: the protocol for the ledger's resource and the user
//! for the user's resource.
//!
//! An important thing to note is that the protocol's protocol's asset can't be
//! changed at runtime after the component has been instantiated, it will be
//! forever stuck with that protocol's asset. The user assets can be added and
//! removed by adding and removing pools to the allowed pools list. In the case
//! of the protocol officially run by RDX Works, the protocol's asset will be
//! XRD and the user's asset will be BTC, ETH, USDC, and USDT. However, Ignition
//! is actually general enough that it can be used by projects who would like to
//! improve their liquidity and who're willing to lose some tokens in the
//! process.
//!
//! The protocol's blueprint is made to be quite modular and to allow for easy
//! upgrading if needed. This means that the protocol's assets can be withdrawn
//! by the protocol owner and that many of the external components that the
//! protocol relies on can be swapped at runtime with little trouble. As an
//! example, the protocol communicates with Dexes through adapters meaning that
//! additional Dexes can be supported by writing and registering new adapters to
//! the existing component on ledger and that support for dexes can be removed
//! by removing their adapter. Additionally, the oracle can be swapped and
//! changed at any point of time to a new oracle. Changing the oracle or the
//! adapters relies on the interface being the same, if the interface is
//! different then, unfortunately, there is no way for the protocol to check at
//! runtime but calls using the oracle or adapter would fail. Thus, changes must
//! be preceded by an interface check.
//!
//! Similarly, the reward rates are quite modular too and are added at runtime
//! and not baked into the blueprint itself allowing additional reward rates to
//! be added and for some reward rates to be removed.

use crate::types::*;
use adapters_interface::prelude::*;
use scrypto::prelude::*;

type PoolAdapter = PoolAdapterInterfaceScryptoStub;
type OracleAdapter = OracleAdapterInterfaceScryptoStub;

#[blueprint]
mod ignition {
    struct Ignition {
        /// A reference to the resource manager of the protocol's resource. This
        /// is the resource that the protocol will be lending out to users who
        /// wish to provide liquidity. In other words, this is the one side of
        /// the liquidity that will be provided by the protocol and the other
        /// side must be provided by the user. This can't be changed after the
        /// component has been instantiated. Thus, it would be chosen with some
        /// caution.
        protocol_resource: ResourceManager,

        /// The adapter of the oracle to use for the protocol. The oracle is
        /// expected to have a specific interface that is required by this
        /// blueprint. This adapter can be updated and changed at runtime to
        /// a new one or even to a completely new oracle.
        oracle_adapter: OracleAdapter,

        /// Information about the pool blueprints, indexed by the id of the
        /// blueprint. This contains information about the adapter to use, the
        /// list of pools that contributions are allowed to, and a reference
        /// to the resource manager of the liquidity receipt. Everything about
        /// this is updatable. Entries can be added and removed, adapters can
        /// be changed, pools can be added or removed from the list of allowed
        /// pools, and liquidity receipt addresses can be changed.
        pool_information: KeyValueStore<BlueprintId, PoolBlueprintInformation>,

        /* Vaults */
        /// A key value store of all of the vaults of ignition, including the
        /// vault of the protocol resources that the protocol uses to provide
        /// liquidity to pools. Only the owner of the protocol is allowed to
        /// deposit and withdraw from these vaults.
        vaults: KeyValueStore<ResourceAddress, FungibleVault>,

        /// The vaults storing the pool units and liquidity receipts obtained
        /// from providing the liquidity. It is indexed by the non-fungible
        /// global id of the liquidity receipt non-fungible token minted by
        /// the protocol when liquidity is provided. Only the owner of the
        /// protocol is allowed to deposit or withdraw into these vaults.
        pool_units: KeyValueStore<NonFungibleGlobalId, Vault>,

        /* Configuration */
        /// The maximum allowed staleness of prices in seconds. If a price is
        /// found to be older than this then it is deemed to be invalid.
        maximum_allowed_price_staleness: i64,

        /// The upfront reward rates supported by the protocol. This is a map of
        /// the lockup period to the reward rate ratio. In this case, the value
        /// is a decimal in the range [0, ∞] where 0 means 0%, 0.5 means 50%,
        /// and 1 means 100%.
        reward_rates: KeyValueStore<LockupPeriod, Decimal>,

        /// Controls whether the protocol currently allows users to open
        /// liquidity positions or not.
        is_open_position_enabled: bool,

        /// Controls whether the protocol currently allows users to close
        /// liquidity positions or not.
        is_close_position_enabled: bool,

        /// The maximum percentage of price difference the protocol is willing
        /// to accept before deeming the price difference to be too much. This
        /// is a decimal in the range [0, ∞] where 0 means 0%, 0.5 means 50%,
        /// and 1 means 100%.
        maximum_allowed_price_difference_percentage: Decimal,
    }

    impl Ignition {}
}

/// Represents the information of pools belonging to a particular blueprint.
#[derive(Clone, Debug, PartialEq, Eq, ScryptoSbor)]
pub struct PoolBlueprintInformation {
    /// The adapter to utilize when making calls to pools belonging to this
    /// blueprint.
    pub adapter: PoolAdapter,

    /// A vector of the pools that the protocol allows contributions to. A pool
    /// that is not found in this list for their corresponding blueprint will
    /// not be allowed to be contributed to.
    pub allowed_pools: IndexSet<ComponentAddress>,

    /// A reference to the resource manager of the resource used as a receipt
    /// for providing liquidity to pools of this blueprint
    pub liquidity_receipt: ResourceManager,
}
