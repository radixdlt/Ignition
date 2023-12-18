#![allow(clippy::too_many_arguments)]

mod percent;

use adapters_interface::oracle::*;
use adapters_interface::pool::*;
use scrypto::prelude::*;

use percent::*;

/// The data of the liquidity positions given to the users of Olympus.
#[derive(ScryptoSbor, NonFungibleData)]
pub struct LiquidityPosition {
    /* Metadata/NonFungibleData standard */
    name: String,
    description: String,
    key_image_url: Url,

    /* Display Data - Just for wallet display, no logic depends on this. */
    /// A string of the lockup period of the liquidity provided through the
    /// protocol (e.g., "6 Months").
    lockup_period: String,

    /// A url linking to where we redeem
    redemption_url: Url,

    /* Application data */
    /// The address of the resource that the user contributed through the
    /// protocol.
    contributed_resource: ResourceAddress,

    /// The amount of the resource that the user contributed through the
    /// protocol.
    contributed_amount: Decimal,

    /// This is the USDC value of the contribution that the user has made. By
    /// extension of that, it is also the value of XRD that the protocol has
    /// provided the user.
    contribution_value: Decimal,

    /// The date after which this liquidity position can be closed.
    //TODO: Wallet should display this as time and not unix timestamp.
    maturity_date: Instant,
}

#[blueprint]
#[types(LiquidityPosition)]
mod olympus {
    enable_method_auth! {
        roles {
            protocol_owner => updatable_by: [protocol_owner];
            protocol_manager => updatable_by: [protocol_manager];
        },
        methods {
            update_oracle => restrict_to: [
                protocol_owner,
                protocol_manager
            ];
            add_allowed_pool => restrict_to: [
                protocol_owner,
                protocol_manager
            ];
            remove_allowed_pool => restrict_to: [
                protocol_owner,
                protocol_manager
            ];
            config_open_liquidity_position => restrict_to: [
                protocol_owner,
                protocol_manager
            ];
            config_close_liquidity_position => restrict_to: [
                protocol_owner,
                protocol_manager
            ];
            add_pool_adapter => restrict_to: [
                protocol_owner,
                protocol_manager
            ];
            remove_pool_adapter => restrict_to: [
                protocol_owner,
                protocol_manager
            ];
            update_maximum_allowed_price_staleness => restrict_to: [
                protocol_owner,
                protocol_manager
            ];
            deposit => restrict_to: [protocol_owner];
            withdraw => restrict_to: [protocol_owner];
            withdraw_pool_units => restrict_to: [protocol_owner];
            add_rewards_rate => restrict_to: [protocol_owner];
            remove_rewards_rate => restrict_to: [protocol_owner];
            update_usd_resource_address => restrict_to: [protocol_owner];
        }
    }

    pub struct Olympus {
        /// The oracle that Olympus uses to get the price of assets at any point
        /// of time. This blueprint requires that the oracle implements a
        /// specific interface which is defined in the adapters_interface
        /// interface crate.
        ///
        /// This field is updatable allowing us to switch to different oracles
        /// at any point of time. The only role that can update this field is
        /// the "protocol_manager" role through the `update_oracle` method. This
        /// method does _not_ check the interface. So, the caller must ensure
        /// the compatibility of the interface.
        ///
        /// [OLYPS-12]: We need a mechanism for verifying that the various
        /// adapters_interface do indeed implement the interface that expect
        /// them to implement. How do we verify that? Would off-chain
        /// verification be acceptable since we're the only admins?
        oracle: OracleAdapter,

        /// The set of all of the pools that are supported by the incentive
        /// program. No contributions are allowed to any pool that is outside
        /// of this set of pools.
        ///
        /// The "protocol_manager" role can add and remove items from this set
        /// through the `add_allowed_pool` and `remove_allowed_pool` methods.
        ///
        /// The pool users want to contribute to must be in this set when they
        /// want to open their liquidity positions but does not have to be in
        /// this set by the time that they close it.
        allowed_pools: IndexSet<NodeId>,

        /// A mapping of the adapters_interface to use for each of the pool
        /// blueprints supported by the protocol.
        ///
        /// The "protocol_manager" role can upsert new entries through the
        /// `add_pool_adapter` method and can remove entries through the
        /// `remove_pool_adapter` method.
        ///
        /// It is possible to remove an adapter while it is still in use and
        /// the protocol makes no guarantees on the existence of
        /// adapters_interface. This should be managed off-ledger.
        pool_adapters: KeyValueStore<BlueprintId, PoolAdapter>,

        /// The vaults where XRD and the various other assets are stored to be
        /// used by the protocol.
        ///
        /// The "protocol_owner" role is allowed to directly withdraw or deposit
        /// from these vaults. This is mainly for upgradeability and to allow it
        /// to provide the XRD used for the incentives.
        vaults: KeyValueStore<ResourceAddress, FungibleVault>,

        /// The resource manager and resource address of the liquidity position
        /// non-fungible resource. Users are given this resource when they open
        /// their liquidity positions and use this resource to close their
        /// positions and get their assets back.
        liquidity_position_resource: ResourceManager,

        /// Stores the pool units associated with the various liquidity
        /// positions in multiple different vaults that are indexed by the
        /// non-fungible global id of the position non-fungible. This separates
        /// the pool units of different positions.
        pool_units: KeyValueStore<NonFungibleGlobalId, FungibleVault>,

        /// The reward rates offered by the incentive program. This maps the
        /// lockup time in seconds to the percentage. This means that there can
        /// only be one percentage associated with any lockup period.
        ///
        /// Note the following:
        /// * The key is a [`u32`] of the seconds of the lockup time. A u32
        /// value of 1 equals 1 second.
        /// * The value is a [`Percent`] which is a decimal between 0 and 1.
        reward_rates: KeyValueStore<u32, Percent>,

        /// The resource address of the USDC, USDT, or any stablecoin. This
        /// resource is needed when trying to find the value of the tokens
        /// contributed by the user so we get their price with USD as the quote
        /// currency.
        usd_resource_address: ResourceAddress,

        /// The maximum allowed staleness of prices in seconds. If a price is
        /// found to be older than this, then it will be deemed invalid and will
        /// cause a panic.
        maximum_allowed_price_staleness: i64,

        /// Controls whether the opening of new liquidity positions is enabled.
        ///
        /// The "protocol_manager" role can set this to [`true`] or [`false`]
        /// through the `config_open_liquidity_positions` method.
        is_open_liquidity_position_enabled: bool,

        /// Controls whether the closing of liquidity positions is enabled.
        ///
        /// The "protocol_manager" role can set this to [`true`] or [`false`]
        /// through the `config_close_liquidity_positions` method.
        is_close_liquidity_position_enabled: bool,
    }

    impl Olympus {
        pub fn instantiate(
            /* Access Rules */
            owner_role: OwnerRole,
            protocol_owner_role: AccessRule,
            protocol_manager_role: AccessRule,
            /* Protocol Parameters */
            oracle: OracleAdapter,
            usd_resource_address: ResourceAddress,
            /* Misc */
            address_reservation: Option<GlobalAddressReservation>,
        ) -> Global<Olympus> {
            // If no address reservation is provided then reserve an address to
            // globalize the component to - this is to provide us with a non
            // branching way of globalizing the component.
            let (address_reservation, component_address) =
                match address_reservation {
                    Some(address_reservation) => {
                        let address = ComponentAddress::try_from(
                            Runtime::get_reservation_address(
                                &address_reservation,
                            ),
                        )
                        .expect(
                            "Allocated address is not a component address!",
                        );

                        (address_reservation, address)
                    }
                    None => Runtime::allocate_component_address(
                        Olympus::blueprint_id(),
                    ),
                };

            // Creating the liquidity position non-fungible resource. This
            // resource can be minted and burned by this component and the
            // "protocol_owner" role has the ability to update who can mint
            // and burn the resource. This is to allow for upgradeability such
            // that the mint and burn abilities can be given to newer versions
            // of the protocol and taken away from older ones.
            let this_component = global_caller(component_address);
            let liquidity_position_resource =
                ResourceBuilder::new_ruid_non_fungible_with_registered_type::<
                    LiquidityPosition,
                >(owner_role.clone())
                .metadata(metadata! {
                    init {
                        // TODO: What should we put here - is this ok?
                        // TODO: Should the fields be locked?
                        "name" => "Olympus Liquidity Position", locked;
                        "description" => "A non-fungible that represents a liquidity position in the Olympus incentive program.", locked;
                        "tags" => Vec::<String>::new(), locked;
                        "icon_url" => "https://www.example.com", locked;
                        "info_url" => "https://www.example.com", locked;
                    }
                })
                .mint_roles(mint_roles! {
                    minter => rule!(require(this_component.clone()));
                    minter_updater => protocol_owner_role.clone();
                })
                .burn_roles(burn_roles! {
                    burner => rule!(require(this_component.clone()));
                    burner_updater => protocol_owner_role.clone();
                })
                .create_with_no_initial_supply();

            Self {
                oracle,
                usd_resource_address,
                allowed_pools: Default::default(),
                pool_adapters: KeyValueStore::new(),
                vaults: KeyValueStore::new(),
                liquidity_position_resource,
                pool_units: KeyValueStore::new(),
                is_open_liquidity_position_enabled: false,
                is_close_liquidity_position_enabled: false,
                reward_rates: KeyValueStore::new(),
                maximum_allowed_price_staleness: 5 * 60, /* 5 Minutes */
            }
            .instantiate()
            .prepare_to_globalize(owner_role)
            .roles(roles! {
                protocol_owner => protocol_owner_role;
                protocol_manager => protocol_manager_role;
            })
            .with_address(address_reservation)
            .globalize()
        }

        /// Updates the oracle used by the protocol to a different oracle.
        ///
        /// This method does _not_ check that the interface of the new oracle
        /// matches that we expect. Thus, such a check must be performed
        /// off-ledger.
        ///
        /// To be more specific, this method takes in the component address of
        /// the oracle's _adapter_ and not the oracle itself. The adapter must
        /// have the interface defined in [`OracleAdapter`].
        ///
        /// # Example Scenario
        ///
        /// We may wish to change the oracle provider for any number of reasons.
        /// As an example, imagine if the initial oracle provider goes under and
        /// stops operations. This allows for the oracle to be replaced with one
        /// that has the same interface without the need to jump to a new
        /// component.
        ///
        /// # Access
        ///
        /// Requires the `protocol_manager` or `protocol_owner` roles.
        ///
        /// # Arguments
        ///
        /// * `oracle`: [`ComponentAddress`] - The address of the new oracle
        /// component to use.
        ///
        /// # Note
        ///
        /// This performs no interface checks and can theoretically accept the
        /// address of a component that does not implement the oracle interface.
        ///
        /// # Issues To Resolve
        ///
        /// * OLYPS-12
        pub fn update_oracle(&mut self, oracle: ComponentAddress) {
            self.oracle = OracleAdapter::from(oracle);
        }

        /// Adds a pool adapter to the protocol.
        ///
        /// Adds a new pool adapter component to the protocol thus forwarding
        /// all contributions and redeems to pools of the provided blueprint id
        /// to the provided adapter.
        ///
        /// This method does not check that the provided adapter conforms to the
        /// [`PoolAdapter`] interface. It is the job of the caller to perform
        /// this check off-ledger.
        ///
        /// If the [`BlueprintId`] already maps to an adapter then it will be
        /// overwritten. Thus, this is an upsert operation.
        ///
        /// # Example Scenario
        ///
        /// We may wish to add support for additional decentralized exchanges
        /// after the protocol goes live. To do this, we would just need to
        /// develop and deploy an adapter and then register the adapter to the
        /// protocol through this method.
        ///
        /// # Access
        ///
        /// Requires the `protocol_manager` or `protocol_owner` roles.
        ///
        /// # Arguments
        ///
        /// `blueprint_id`: [`BlueprintId`] - The package address and blueprint
        /// name of the pool blueprint.
        /// `adapter`: [`ComponentAddress`] - The address of the adapter
        /// component.
        ///
        /// # Note
        ///
        /// This performs no interface checks and can theoretically accept the
        /// address of a component that does not implement the oracle interface.
        ///
        /// # Issues To Resolve
        ///
        /// * OLYPS-12
        pub fn add_pool_adapter(
            &mut self,
            blueprint_id: BlueprintId,
            adapter: ComponentAddress,
        ) {
            self.pool_adapters
                .insert(blueprint_id, PoolAdapter::from(adapter));
        }

        /// Removes a pool adapter from the protocol.
        ///
        /// Un-registers the pool adapter associated with the given blueprint
        /// id from the protocol. If it does not exist then nothing happens and
        /// no errors are thrown.
        ///
        /// # Access
        ///
        /// Requires the `protocol_manager` or `protocol_owner` roles.
        ///
        /// # Example Scenario
        ///
        /// Say one of the exchanges migrate from one package to another, we may
        /// wish to honor this upgrade too and to deregister the adapter for the
        /// particular package from the protocol.
        ///
        /// # Arguments
        ///
        /// * `blueprint_id`: [`BlueprintId`] - The package address and
        /// blueprint name of the pool blueprint to remove the adapter for.
        pub fn remove_pool_adapter(&mut self, blueprint_id: BlueprintId) {
            self.pool_adapters.remove(&blueprint_id);
        }

        /// Adds an allowed pool to the protocol.
        ///
        /// This protocol does not provide an incentive to any liquidity pool.
        /// Only a small set of pools that are chosen by the pool manager. This
        /// method adds a pool to the set of pools that the protocol provides an
        /// incentive for and that users can provide liquidity to.
        ///
        /// This method checks that an adapter exists for the passed component.
        /// If no adapter exists then this method panics and the transaction
        /// fails.
        ///
        /// # Access
        ///
        /// Requires the `protocol_manager` or `protocol_owner` roles.
        ///
        /// # Example Scenario
        ///
        /// We may wish to incentivize liquidity for a new bridged asset and a
        /// new set of pools. An even more compelling scenario, we may wish to
        /// provide incentives for a newly released DEX.
        ///
        /// # Arguments
        ///
        /// * `component`: [`ComponentAddress`] - The address of the pool
        /// component to add to the set of allowed pools.
        ///
        /// # Note:
        ///
        /// * The component address provided as an argument is of the wrapper
        /// or encapsulating pool and not the native pool.
        pub fn add_allowed_pool(&mut self, component: ComponentAddress) {
            let blueprint_id =
                ScryptoVmV1Api::object_get_blueprint_id(component.as_node_id());
            if self.pool_adapters.get(&blueprint_id).is_some() {
                self.allowed_pools.insert(component.into_node_id());
            } else {
                let address_string = Runtime::bech32_encode_address(component);
                panic!("No adapter found for component: {}", address_string)
            }
        }

        /// Removes one of the existing allowed liquidity pools.
        ///
        /// Given the component address of the liquidity pool, this method
        /// removes that liquidity pool from the list of allowed liquidity
        /// pools.
        ///
        /// # Access
        ///
        /// Requires the `protocol_manager` or `protocol_owner` roles.
        ///
        /// # Example Scenario
        ///
        /// We may wish to to remove or stop a certain liquidity pool from the
        /// incentive program essentially disallowing new liquidity positions
        /// but permitting closure of liquidity positions.
        ///
        /// # Arguments
        ///
        /// * `component`: [`ComponentAddress`] - The address of the pool
        /// component to remove from the set of allowed pools.
        ///
        /// # Note:
        ///
        /// * The component address provided as an argument is of the wrapper
        /// or encapsulating pool and not the native pool.
        pub fn remove_allowed_pool(&mut self, component: ComponentAddress) {
            self.allowed_pools.remove(component.as_node_id());
        }

        /// Enables or disables the ability to open new liquidity positions
        ///
        /// # Access
        ///
        /// Requires the `protocol_manager` or `protocol_owner` roles.
        ///
        /// # Example Scenario
        ///
        /// We might want to pause the incentive program for some period due to
        /// any number of reasons.
        ///
        /// # Arguments
        ///
        /// * `is_enabled`: [`bool`] - Controls whether opening of liquidity
        /// positions is enabled or disabled.
        pub fn config_open_liquidity_position(&mut self, is_enabled: bool) {
            self.is_open_liquidity_position_enabled = is_enabled
        }

        /// Enables or disables the ability to close new liquidity positions
        ///
        /// # Access
        ///
        /// Requires the `protocol_manager` or `protocol_owner` roles.
        ///
        /// # Example Scenario
        ///
        /// We might want to pause the incentive program for some period due to
        /// any number of reasons.
        ///
        /// # Arguments
        ///
        /// * `is_enabled`: [`bool`] - Controls whether closing of liquidity
        /// positions is enabled or disabled.
        pub fn config_close_liquidity_position(&mut self, is_enabled: bool) {
            self.is_close_liquidity_position_enabled = is_enabled
        }

        /// Deposits resources into the protocol.
        ///
        /// # Access
        ///
        /// Requires the `protocol_owner` role.
        ///
        /// # Example Scenario
        ///
        /// This method can be used to fund the incentive program with XRD and
        /// deposit other assets as well.
        ///
        /// # Arguments
        ///
        /// * `bucket`: [`FungibleBucket`] - A bucket of resources to deposit
        /// into the protocol.
        pub fn deposit(&mut self, bucket: FungibleBucket) {
            let resource_address = bucket.resource_address();

            let mut entry = self.vaults.get_mut(&resource_address);
            match entry {
                Some(ref mut vault) => vault.put(bucket),
                None => {
                    drop(entry);

                    let mut vault = FungibleVault::new(resource_address);
                    vault.put(bucket);

                    self.vaults.insert(resource_address, vault);
                }
            }
        }

        /// Withdraws resources from the protocol.
        ///
        /// # Access
        ///
        /// Requires the `protocol_owner` role.
        ///
        /// # Example Scenario
        ///
        /// This method can be used to end the incentive program by withdrawing
        /// the XRD in the protocol. Additionally, it can be used for upgrading
        /// the protocol by withdrawing the resources in the protocol.
        ///
        /// # Arguments
        ///
        /// * `resource_address`: [`ResourceAddress`] - The address of the
        /// resource to withdraw.
        /// * `amount`: [`Decimal`] - The amount to withdraw.
        ///
        /// # Returns
        ///
        /// * [`FungibleBucket`] - A bucket of the withdrawn tokens.
        pub fn withdraw(
            &mut self,
            resource_address: ResourceAddress,
            amount: Decimal,
        ) -> FungibleBucket {
            self.vaults
                .get_mut(&resource_address)
                .expect("Vault does not exist")
                .take(amount)
        }

        /// Withdraws pool units from the protocol. This is primarily for any
        /// upgradeability needs that the protocol has.
        ///
        /// # Access
        ///
        /// Requires the `protocol_owner` role.
        ///
        /// # Example Scenario
        ///
        /// This method can be used to withdraw the pool units from the protocol
        /// for the purposes of upgradeability to move them to another component
        ///
        /// # Arguments
        ///
        /// * `id`: [`NonFungibleGlobalId`] - The global id of the non-fungible
        /// liquidity position NFTs to withdraw the pool units associated with.
        ///
        /// # Returns
        ///
        /// * [`FungibleBucket`] - A bucket of the withdrawn tokens.
        pub fn withdraw_pool_units(
            &mut self,
            id: NonFungibleGlobalId,
        ) -> FungibleBucket {
            self.pool_units
                .get_mut(&id)
                .expect("No pool units exist for id")
                .take_all()
        }

        /// Adds a rewards rate to the protocol.
        ///
        /// Given a certain lockup period in seconds and a percentage rewards
        /// rate, this method adds this rate to the protocol allowing users to
        /// choose this option when contributing liquidity.
        ///
        /// # Access
        ///
        /// Requires the `protocol_owner` role.
        ///
        /// # Example Scenario
        ///
        /// We might wish to add a new higher rate with a longer lockup period
        /// to incentivize people to lock up their liquidity for even shorter.
        /// Or, we might want to introduce a new 3 months category, or anything
        /// in between.
        ///
        /// # Arguments
        ///
        /// * `lockup_period`: [`u32`] - The lockup period in seconds.
        /// * `rate`: [`Percent`] - The rewards rate as a percent.
        pub fn add_rewards_rate(&mut self, lockup_period: u32, rate: Percent) {
            self.reward_rates.insert(lockup_period, rate);
        }

        /// Removes a rewards rate from the protocol.
        ///
        /// # Access
        ///
        /// Requires the `protocol_owner` role.
        ///
        /// # Example Scenario
        ///
        /// A certain rate might get used too much and we might want to switch
        /// off this rate (even if temporarily). This allows us to remove this
        /// rate and add it back later when we want to.
        ///
        /// # Arguments
        ///
        /// * `lockup_period`: [`u32`] - The lockup period in seconds associated
        /// with the rewards rate that we would like to remove.
        pub fn remove_rewards_rate(&mut self, lockup_period: u32) {
            self.reward_rates.remove(&lockup_period);
        }

        /// Updates the value of the maximum allowed price staleness by the
        /// protocol.
        ///
        /// # Access
        ///
        /// Requires the `protocol_owner` or `protocol_manager` role.
        ///
        /// # Example Scenario
        ///
        /// We may wish to change the allowed staleness of prices to a very
        /// short period if we get an oracle that operates at realtime speeds
        /// or if we change oracle vendors.
        ///
        /// # Arguments
        ///
        /// * `value`: [`i64`] - The maximum allowed staleness period in
        /// seconds.
        pub fn update_maximum_allowed_price_staleness(&mut self, value: i64) {
            self.maximum_allowed_price_staleness = value;
        }

        /// Updates the resource address of the USD resource to another address.
        ///
        /// # Access
        ///
        /// Requires the `protocol_owner` role.
        ///
        /// # Example Scenario
        ///
        /// USDT goes under and we need to replace everything with USDC, of
        /// course this a disaster to Crypto, but still, we need to be able
        /// to do such things!
        ///
        /// # Arguments
        ///
        /// * `usd_resource_address`: [`ResourceAddress`] - The address of the
        /// USD resource.
        ///
        /// # Note
        ///
        /// This method does not checks whatsoever on whether this resource is
        /// supported by the current oracle or not. It is the protocol owner's
        /// role to make such a check before updating the resource address. If
        /// an address is provided that is not supported by the oracle then this
        /// would result in the component stopping to work: contributions and
        /// redemptions would not work.
        pub fn update_usd_resource_address(
            &mut self,
            usd_resource_address: ResourceAddress,
        ) {
            self.usd_resource_address = usd_resource_address;
        }

        /// Gets the price of the `base` asset in terms of the `quote` asset
        /// from the currently configured oracle, checks for staleness, and
        /// returns the price.
        ///
        /// # Arguments
        ///
        /// * `base`: [`ResourceAddress`] - The base resource address.
        /// * `quote`: [`ResourceAddress`] - The quote resource address.
        ///
        /// # Returns
        ///
        /// [`Decimal`] - The price of the base asset in terms of the quote
        /// asset.
        fn get_price(
            &self,
            base: ResourceAddress,
            quote: ResourceAddress,
        ) -> Decimal {
            // Get the price
            let (price, instant) = self.oracle.get_price(base, quote);

            // Check for staleness
            if Clock::current_time(TimePrecision::Minute)
                .seconds_since_unix_epoch
                - instant.seconds_since_unix_epoch
                > self.maximum_allowed_price_staleness
                && Clock::current_time_is_at_or_after(
                    instant,
                    TimePrecision::Minute,
                )
            {
                panic!("Maximum allowed price staleness exceeded for {base:?}-{quote:?}")
            }

            // Return price
            price
        }
    }
}
