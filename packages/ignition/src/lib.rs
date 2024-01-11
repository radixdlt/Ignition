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

use scrypto::prelude::*;

#[blueprint]
mod ignition {
    struct Ignition {}

    impl Ignition {}
}
