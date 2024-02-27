use scrypto::prelude::*;
use scrypto_interface::*;

define_interface! {
    PlazaPair as DefiPlazaV2Pool impl [
        ScryptoStub,
        ScryptoTestStub,
        #[cfg(feature = "manifest-builder-stubs")]
        ManifestBuilderStub
    ] {
        fn instantiate_pair(
            owner_role: OwnerRole,
            base_address: ResourceAddress,
            quote_address: ResourceAddress,
            config: PairConfig,
            initial_price: Decimal,
        ) -> Self;
        fn add_liquidity(
            &mut self,
            #[manifest_type = "ManifestBucket"]
            input_bucket: Bucket,
            #[manifest_type = "Option<ManifestBucket>"]
            co_liquidity_bucket: Option<Bucket>,
        ) -> (Bucket, Option<Bucket>);
        fn remove_liquidity(
            &mut self,
            #[manifest_type = "ManifestBucket"]
            lp_bucket: Bucket,
            is_quote: bool,
        ) -> (Bucket, Bucket);
        fn swap(
            &mut self,
            #[manifest_type = "ManifestBucket"]
            input_bucket: Bucket,
        ) -> (Bucket, Option<Bucket>);
        fn quote(
            &self,
            input_amount: Decimal,
            input_is_quote: bool,
        ) -> (Decimal, Decimal, Decimal, TradeAllocation, PairState);
        fn get_state(&self) -> PairState;
        fn get_tokens(&self) -> (ResourceAddress, ResourceAddress);
        fn get_pools(
            &self,
        ) -> (ComponentAddress, ComponentAddress);
    }
}

#[derive(
    ScryptoSbor,
    ManifestSbor,
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
pub struct PairConfig {
    pub k_in: Decimal,
    pub k_out: Decimal,
    pub fee: Decimal,
    pub decay_factor: Decimal,
}

#[derive(
    ScryptoSbor,
    ManifestSbor,
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
pub struct TradeAllocation {
    pub base_base: Decimal,
    pub base_quote: Decimal,
    pub quote_base: Decimal,
    pub quote_quote: Decimal,
}

#[derive(
    ScryptoSbor,
    ManifestSbor,
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
pub struct PairState {
    pub p0: Decimal,
    pub shortage: Shortage,
    pub target_ratio: Decimal,
    pub last_outgoing: i64,
    pub last_out_spot: Decimal,
}

#[derive(
    ScryptoSbor,
    ManifestSbor,
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
pub enum Shortage {
    BaseShortage,
    Equilibrium,
    QuoteShortage,
}

#[derive(
    ScryptoSbor,
    ManifestSbor,
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
pub enum ShortageState {
    Equilibrium,
    Shortage(Asset),
}

#[derive(
    ScryptoSbor,
    ManifestSbor,
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
pub enum Asset {
    Base,
    Quote,
}

impl From<Shortage> for ShortageState {
    fn from(value: Shortage) -> Self {
        match value {
            Shortage::Equilibrium => ShortageState::Equilibrium,
            Shortage::BaseShortage => ShortageState::Shortage(Asset::Base),
            Shortage::QuoteShortage => ShortageState::Shortage(Asset::Quote),
        }
    }
}
