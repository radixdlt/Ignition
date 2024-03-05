use super::macros::*;
use common::prelude::*;
use macro_rules_attribute::apply;
use radix_engine::prelude::*;
use transaction::prelude::*;

pub struct PublishingConfiguration {
    /// The configuration of the Ignition protocol.
    pub protocol_configuration: ProtocolConfiguration,

    /// The metadata to use for the dapp definition that is created.
    pub dapp_definition_metadata: IndexMap<String, MetadataValue>,

    /// Contains configurations for the transactions that will be submitted
    /// such as the notary and the account to get the fees from. Information
    /// that mostly pertains to signing.
    pub transaction_configuration: TransactionConfiguration,

    /// Contains information on the various badges to use for publishing and
    /// whether these badges already exist or should be created.
    pub badges: BadgeIndexedData<BadgeHandling>,

    /// Contains information on the user resources that this deployment will use
    /// such as their addresses or information about their properties if they're
    /// to be created during the publishing process.
    pub user_resources: UserResourceIndexedData<UserResourceHandling>,

    /// Contains information on how each of the packages should be handled and
    /// whether they should be compiled and published or if pre-existing ones
    /// should be used.
    pub packages: Entities<PackageHandling>,

    /// Information about the exchange will be supported in Ignition. This
    /// contains information necessary for the publishing and bootstrapping
    /// process of Ignition. As an example, the address of the exchange's
    /// package, the name of the blueprint, and the pools that we wish to
    /// support. This uses an [`Option`] to allow for cases when there are
    /// some networks where these exchanges are not live and therefore their
    /// information can't be provided as part of publishing.
    pub exchange_information: ExchangeIndexedData<
        Option<ExchangeInformation<PoolHandling, LiquidityReceiptHandling>>,
    >,

    /// Additional information that doesn't quite fit into any of the above
    /// categories nicely.
    pub additional_information: AdditionalInformation,

    /// Bit flags for additional operations that can be done by the publishing
    /// logic during the publishing process.
    pub additional_operation_flags: AdditionalOperationFlags,
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct PublishingReceipt {
    pub packages: Entities<PackageAddress>,
    pub components: Entities<ComponentAddress>,
    pub exchange_information: ExchangeIndexedData<
        Option<ExchangeInformation<ComponentAddress, ResourceAddress>>,
    >,
    pub protocol_configuration: ProtocolConfigurationReceipt,
    pub badges: BadgeIndexedData<ResourceAddress>,
}

bitflags::bitflags! {
    /// Additional operations that the publishing process can be instructed to
    /// perform.
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct AdditionalOperationFlags: u8 {
        /// Submits prices to the oracle that are just one for all of the assets
        /// supported in the deployment.
        const SUBMIT_ORACLE_PRICES_OF_ONE = 0b00000001;

        /// Provide initial liquidity to Ignition. How this is done depends on
        /// the selected protocol resource. If it is XRD then the publisher will
        /// attempt to get the XRD from the faucet. If otherwise then it will
        /// attempt to mint it.
        const PROVIDE_INITIAL_IGNITION_LIQUIDITY = 0b00000010;
    }
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct ProtocolConfigurationReceipt {
    pub protocol_resource: ResourceAddress,
    pub user_resource_volatility: UserResourceIndexedData<Volatility>,
    pub reward_rates: IndexMap<LockupPeriod, Decimal>,
    pub allow_opening_liquidity_positions: bool,
    pub allow_closing_liquidity_positions: bool,
    pub maximum_allowed_price_staleness: i64,
    pub maximum_allowed_price_difference_percentage: Decimal,
    pub user_resources: UserResourceIndexedData<ResourceAddress>,
    pub registered_pools:
        ExchangeIndexedData<Option<UserResourceIndexedData<ComponentAddress>>>,
}

pub struct AdditionalInformation {
    pub ociswap_v2_registry_component: Option<ComponentAddress>,
}

pub struct ProtocolConfiguration {
    pub protocol_resource: ResourceAddress,
    pub user_resource_volatility: UserResourceIndexedData<Volatility>,
    pub reward_rates: IndexMap<LockupPeriod, Decimal>,
    pub allow_opening_liquidity_positions: bool,
    pub allow_closing_liquidity_positions: bool,
    pub maximum_allowed_price_staleness: i64,
    pub maximum_allowed_price_difference_percentage: Decimal,
    pub entities_metadata: Entities<MetadataInit>,
}

pub struct TransactionConfiguration {
    pub notary: PrivateKey,
    pub fee_payer_information: AccountAndControllingKey,
}

pub struct AccountAndControllingKey {
    pub account_address: ComponentAddress,
    pub controlling_key: PrivateKey,
}

impl AccountAndControllingKey {
    pub fn new_virtual_account(controlling_key: PrivateKey) -> Self {
        let account_address = ComponentAddress::virtual_account_from_public_key(
            &controlling_key.public_key(),
        );
        Self {
            account_address,
            controlling_key,
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, ScryptoSbor,
)]
pub struct Entities<T> {
    pub protocol_entities: ProtocolIndexedData<T>,
    pub exchange_adapter_entities: ExchangeIndexedData<T>,
}

#[apply(name_indexed_struct)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, ScryptoSbor,
)]
pub struct ProtocolIndexedData<T> {
    pub ignition: T,
    pub simple_oracle: T,
}

#[apply(name_indexed_struct)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, ScryptoSbor,
)]
pub struct ExchangeIndexedData<T> {
    pub ociswap_v2: T,
    pub defiplaza_v2: T,
    pub caviarnine_v1: T,
}

#[apply(name_indexed_struct)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, ScryptoSbor,
)]
pub struct UserResourceIndexedData<T> {
    pub bitcoin: T,
    pub ethereum: T,
    pub usdc: T,
    pub usdt: T,
}

#[apply(name_indexed_struct)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, ScryptoSbor,
)]
pub struct BadgeIndexedData<T> {
    pub oracle_manager_badge: T,
    pub protocol_owner_badge: T,
    pub protocol_manager_badge: T,
}

pub enum BadgeHandling {
    /// Creates a new badge and deposits it into the specified account.
    CreateAndSend {
        /// The account that the badges should be sent to.
        account_address: ComponentAddress,
        /// The metadata of the created badges.
        metadata_init: MetadataInit,
    },
    /// Use an existing badge that exists in some account. If the badge is
    /// required in one of the operations then a proof of it will be created.
    /// A signature of this account must be provided.
    UseExisting {
        /// The private key of the account that controlling the badge. This is
        /// required for any proofs that need to be created.
        controlling_private_key: PrivateKey,
        /// The address of the holder
        holder_account_address: ComponentAddress,
        /// The address of the badge
        badge_resource_address: ResourceAddress,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ExchangeInformation<P, R> {
    /// The id of the pool blueprint of the exchange.
    pub blueprint_id: BlueprintId,
    /// The pools that we wish to support for the exchange.
    pub pools: UserResourceIndexedData<P>,
    /// The liquidity receipt to use for the exchange.
    pub liquidity_receipt: R,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum PackageHandling {
    /// The package should be compiled and published in the process.
    LoadAndPublish {
        /// The name of the crate that contains the package. This is the name
        /// that will be used when instructing the package loader to get the
        /// package.
        crate_package_name: String,
        /// The initial metadata to set on the package when it's being published
        metadata: MetadataInit,
        /// The name of the blueprint to use from this package. This is under
        /// the assumption that each package is just a single blueprint.
        blueprint_name: String,
    },
    /// The package already exists on the desired network.
    UseExisting {
        /// The address of the package on the network and
        package_address: BlueprintId,
    },
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, ScryptoSbor,
)]
pub enum PoolHandling {
    /// A pool does not exist and should be created.
    Create,
    /// A pool already exists and should be used.
    UseExisting {
        /// The address of the pool to use
        pool_address: ComponentAddress,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum UserResourceHandling {
    /// Resources do not exist on the network and should be created
    CreateFreelyMintableAndBurnable {
        /// The divisibility to create the resource with
        divisibility: u8,
        /// The initial metadata to use for the resource
        metadata: MetadataInit,
    },
    /// Resources exist on the network and should be used.
    UseExisting {
        /// The address of the resource
        resource_address: ResourceAddress,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum LiquidityReceiptHandling {
    /// Create a new resource to use as the liquidity receipt
    CreateNew {
        /// The non-fungible data schema of the resource.
        non_fungible_schema: NonFungibleDataSchema,
        /// The initial metadata to use for the resource.
        metadata: MetadataInit,
    },
    /// Use an existing resource as the liquidity receipt of the exchange
    UseExisting {
        /// The address of the liquidity receipt resource
        resource_address: ResourceAddress,
    },
}
