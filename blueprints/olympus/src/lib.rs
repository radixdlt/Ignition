use adapters::oracle::*;
use adapters::pool::*;
use scrypto::prelude::*;

#[blueprint]
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
            deposit => restrict_to: [protocol_owner];
            withdraw => restrict_to: [protocol_owner];
        }
    }

    pub struct Olympus {
        /// The oracle that Olympus uses to get the price of assets at any point
        /// of time. This blueprint requires that the oracle implements a
        /// specific interface which is defined in the adapters interface crate.
        ///
        /// This field is updatable allowing us to switch to different oracles
        /// at any point of time. The only role that can update this field is
        /// the "protocol_manager" role through the `update_oracle` method. This
        /// method does _not_ check the interface. So, the caller must ensure
        /// the compatibility of the interface.
        ///
        /// [OLYPS-12]: We need a mechanism for verifying that the various
        /// adapters do indeed implement the interface that expect them to
        /// implement. How do we verify that? Would off-chain verification
        /// be acceptable since we're the only admins?
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

        /// A mapping of the adapters to use for each of the pool blueprints
        /// supported by the protocol.
        ///
        /// The "protocol_manager" role can upsert new entries through the
        /// `add_pool_adapter` method and can remove entries through the
        /// `remove_pool_adapter` method.
        ///
        /// It is possible to remove an adapter while it is still in use and
        /// the protocol makes no guarantees on the existence of adapters. This
        /// should be managed off-ledger.
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
    }
}
