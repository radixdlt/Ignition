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

use std::cmp::min;

use crate::*;
use adapters_interface::prelude::*;
use scrypto::prelude::*;

type PoolAdapter = PoolAdapterInterfaceScryptoStub;
type OracleAdapter = OracleAdapterInterfaceScryptoStub;

#[blueprint]
mod ignition {
    enable_method_auth! {
        roles {
            protocol_owner => updatable_by: [protocol_owner];
            protocol_manager => updatable_by: [protocol_manager];
        },
        methods {
            set_oracle_adapter => restrict_to: [
                protocol_owner,
                protocol_manager
            ];
            set_pool_adapter => restrict_to: [
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
            set_liquidity_receipt => restrict_to: [
                protocol_owner,
                protocol_manager
            ];
            insert_pool_information => restrict_to: [
                protocol_owner,
                protocol_manager
            ];
            remove_pool_information => restrict_to: [
                protocol_owner,
                protocol_manager
            ];
            set_maximum_allowed_price_staleness => restrict_to: [
                protocol_owner,
                protocol_manager
            ];
            remove_reward_rate => restrict_to: [
                protocol_owner,
                protocol_manager
            ];
            add_reward_rate => restrict_to: [
                protocol_owner,
                protocol_manager
            ];
            set_is_open_position_enabled => restrict_to: [
                protocol_owner,
                protocol_manager
            ];
            set_is_close_position_enabled => restrict_to: [
                protocol_owner,
                protocol_manager
            ];
            set_maximum_allowed_price_difference_percentage => restrict_to: [
                protocol_owner,
                protocol_manager
            ];
            deposit_resources => restrict_to: [protocol_owner];
            withdraw_resources => restrict_to: [protocol_owner];
            deposit_pool_units => restrict_to: [protocol_owner];
            withdraw_pool_units => restrict_to: [protocol_owner];
            open_liquidity_position => PUBLIC;
            close_liquidity_position => PUBLIC;
        }
    }

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
        ///
        /// The mapping of the [`BlueprintId`] to the pool information means
        /// that each Dex, or at least Dex blueprint, has a single entry in the
        /// protocol.
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

        /// The maximum allowed staleness of prices in seconds. If a price is
        /// found to be older than this then it is deemed to be invalid.
        maximum_allowed_price_staleness: i64,

        /// The maximum percentage of price difference the protocol is willing
        /// to accept before deeming the price difference to be too much. This
        /// is a decimal in the range [0, ∞] where 0 means 0%, 0.5 means 50%,
        /// and 1 means 100%.
        maximum_allowed_price_difference_percentage: Decimal,
    }

    impl Ignition {
        /// Instantiates a new Ignition protocol component based on the provided
        /// protocol parameters.
        pub fn instantiate(
            /* Rules */
            owner_role: OwnerRole,
            protocol_owner_role: AccessRule,
            protocol_manager_role: AccessRule,
            /* Initial Configuration */
            protocol_resource: ResourceManager,
            oracle_adapter: ComponentAddress,
            maximum_allowed_price_staleness: i64,
            maximum_allowed_price_difference_percentage: Decimal,
            /* Misc */
            address_reservation: Option<GlobalAddressReservation>,
        ) -> Global<Ignition> {
            // If no address reservation is provided then reserve an address to
            // globalize the component to - this is to provide us with a non
            // branching way of globalizing the component.
            let address_reservation = address_reservation.unwrap_or(
                Runtime::allocate_component_address(Ignition::blueprint_id()).0,
            );

            Self {
                protocol_resource,
                oracle_adapter: oracle_adapter.into(),
                pool_information: KeyValueStore::new(),
                vaults: KeyValueStore::new(),
                pool_units: KeyValueStore::new(),
                reward_rates: KeyValueStore::new(),
                is_open_position_enabled: false,
                is_close_position_enabled: false,
                maximum_allowed_price_staleness,
                maximum_allowed_price_difference_percentage,
            }
            .instantiate()
            .prepare_to_globalize(owner_role)
            // TODO: update metadata
            .roles(roles! {
                protocol_owner => protocol_owner_role;
                protocol_manager => protocol_manager_role;
            })
            .with_address(address_reservation)
            .globalize()
        }

        /// Opens a liquidity position for the user.
        ///
        /// Given some bucket of tokens, this method matches this bucket with
        /// XRD of the same value and contributes that XRD to the pool specified
        /// as an argument. The liquidity is locked in that pool for the lockup
        /// period specified as an argument and the user is given back a non
        /// fungible token that represents their portion in the pool.
        ///
        /// If opening a liquidity pool returns more than the pool units and the
        /// change, then these additional tokens are returned back to the caller
        /// and not kept by the protocol.
        ///
        /// # Panics
        ///
        /// There are a number of situations when this method panics and leads
        /// the transaction to fail. Some of them are:
        ///
        /// * If the specified pool is not a registered pool in Ignition and
        /// thus, no liquidity is allowed to be provided to this pool.
        /// * If the lockup period specified by the caller has no corresponding
        /// upfront rewards percentage, and thus it is not a recognized lockup
        /// period by the pool.
        /// * If no adapter is registered for the liquidity pool.
        /// * If the price difference between the pool and the oracle is higher
        /// than what is allowed by the protocol.
        ///
        /// # Arguments
        ///
        /// * `bucket`: [`FungibleBucket`] - A fungible bucket of tokens to
        /// contribute to the pool. Ignition will match the value of this bucket
        /// in XRD and contribute it alongside it to the specified pool.
        /// * `pool_address`: [`ComponentAddress`] - The address of the pool to
        /// contribute to, this must be a valid pool that is registered in the
        /// protocol and that has an adapter.
        /// * `lockup_period`: [`LockupPeriod`] - The amount of time (in
        /// seconds) to lockup the liquidity. This must be a registered lockup
        /// period with a defined upfront rewards rate.
        ///
        /// # Returns
        ///
        /// * [`NonFungibleBucket`] - A non-fungible bucket of the liquidity
        /// position resource that gives the holder the right to close their
        /// liquidity position when the lockup period is up.
        /// * [`FungibleBucket`] - A bucket of the change.
        /// * [`Vec<Bucket>`] - A vector of other buckets that the pools can
        /// return upon contribution, this can be their rewards tokens or
        /// anything else.
        pub fn open_liquidity_position(
            &mut self,
            bucket: FungibleBucket,
            pool_address: ComponentAddress,
            lockup_period: LockupPeriod,
        ) -> (NonFungibleBucket, FungibleBucket, Vec<Bucket>) {
            // Ensure that we currently allow opening liquidity positions.
            assert!(
                self.is_open_position_enabled,
                "{}",
                OPENING_LIQUIDITY_POSITIONS_IS_CLOSED_ERROR
            );

            // Getting a few information so that it is not constantly read from
            // the system.
            let user_resource_address = bucket.resource_address();
            let user_resource_amount = bucket.amount();

            // Ensure that the pool has an adapter and that it is a registered
            // pool. If it is, this means that we can move ahead with the pool.
            // Also, it means that the pool is guaranteed to have the protocol
            // resource on one of its sides.
            let (mut adapter, liquidity_receipt_resource, _) = self
                .checked_get_pool_adapter_and_liquidity_receipt(pool_address)
                .expect(NO_ADAPTER_FOUND_FOR_POOL_ERROR);

            // Ensure that the passed bucket belongs to the pool and that it is
            // not some random resource.
            {
                let (resource1, resource2) =
                    adapter.resource_addresses(pool_address);

                assert!(
                    resource1 == user_resource_address
                        || resource2 == user_resource_address,
                    "{}",
                    USER_ASSET_DOES_NOT_BELONG_TO_POOL_ERROR
                )
            }

            // Compare the price difference between the oracle reported price
            // and the pool reported price - ensure that it is within the
            // allowed price difference range.
            let oracle_reported_price = {
                let oracle_reported_price = self.checked_get_price(
                    user_resource_address,
                    self.protocol_resource.address(),
                );
                let pool_reported_price = adapter.price(pool_address);
                let relative_difference = oracle_reported_price
                    .relative_difference(&pool_reported_price)
                    .expect(USER_ASSET_DOES_NOT_BELONG_TO_POOL_ERROR);

                assert!(
                    relative_difference
                        <= self.maximum_allowed_price_difference_percentage,
                    "{}",
                    RELATIVE_PRICE_DIFFERENCE_LARGER_THAN_ALLOWED_ERROR
                );

                oracle_reported_price
            };

            let oracle_reported_value_of_user_resource_in_protocol_resource =
                oracle_reported_price
                    .exchange(user_resource_address, user_resource_amount)
                    .expect(UNEXPECTED_ERROR)
                    .1;

            // Contribute the resources to the pool.
            let user_side_of_liquidity = bucket;
            let protocol_side_of_liquidity = self.withdraw_resources(
                self.protocol_resource.address(),
                oracle_reported_value_of_user_resource_in_protocol_resource,
            );
            let OpenLiquidityPositionOutput {
                pool_units,
                change,
                others,
            } = adapter.open_liquidity_position(
                pool_address,
                (user_side_of_liquidity.0, protocol_side_of_liquidity.0),
            );

            // Calculate the amount of resources that was actually contributed
            // based on the amount of change that we got back.
            let amount_of_user_tokens_contributed = user_resource_amount
                - change
                    .get(&user_resource_address)
                    .map(Bucket::amount)
                    .unwrap_or(Decimal::ZERO);
            let amount_of_protocol_tokens_contributed =
                oracle_reported_value_of_user_resource_in_protocol_resource
                    - change
                        .get(&self.protocol_resource.address())
                        .map(Bucket::amount)
                        .unwrap_or(Decimal::ZERO);

            // Determine the amount of upfront tokens to provide to the user
            // based on the lockup period specified.
            let upfront_rewards_amount_in_protocol_resource = {
                let oracle_reported_value_of_user_resource_actually_contributed_in_protocol_resource =
                    oracle_reported_price
                        .exchange(
                            user_resource_address,
                            amount_of_user_tokens_contributed,
                        )
                        .expect(UNEXPECTED_ERROR)
                        .1;

                let associated_rewards_rate = self
                    .reward_rates
                    .get(&lockup_period)
                    .expect(LOCKUP_PERIOD_HAS_NO_ASSOCIATED_REWARDS_RATE_ERROR);

                oracle_reported_value_of_user_resource_actually_contributed_in_protocol_resource
                    * *associated_rewards_rate
            };

            let upfront_reward = self.withdraw_resources(
                self.protocol_resource.address(),
                upfront_rewards_amount_in_protocol_resource,
            );

            // Deposit the pool units into the protocol itself and mint an NFT
            // used to represent these locked pool units.
            let liquidity_receipt = {
                let data = LiquidityReceipt::new(
                    lockup_period,
                    pool_address,
                    user_resource_address,
                    amount_of_user_tokens_contributed,
                    amount_of_protocol_tokens_contributed,
                );
                let liquidity_receipt = liquidity_receipt_resource
                    .mint_ruid_non_fungible(data)
                    .as_non_fungible();

                let global_id = NonFungibleGlobalId::new(
                    liquidity_receipt_resource.address(),
                    liquidity_receipt.non_fungible_local_id(),
                );
                self.pool_units
                    .insert(global_id, Vault::with_bucket(pool_units));

                liquidity_receipt
            };

            // Create the buckets to return back to the user.
            let mut buckets_to_return = vec![];
            for bucket in change.into_values() {
                let bucket_resource_address = bucket.resource_address();
                if bucket_resource_address == self.protocol_resource.address() {
                    self.deposit_resources(FungibleBucket(bucket))
                } else {
                    buckets_to_return.push(bucket);
                }
            }
            buckets_to_return.extend(others);

            // Return all
            (liquidity_receipt, upfront_reward, buckets_to_return)
        }

        /// Closes a liquidity position after its maturity period has elapsed.
        ///
        /// Given the non-fungible representing the liquidity receipt, this
        /// method closes the liquidity position after the maturity period
        /// elapses. The liquidity receipt is burned and the user is given
        /// back some amount of assets.
        ///
        /// The assets given back to the user depends on what the protocol gets
        /// back from closing the liquidity position. The following is the
        /// algorithm employed to determine what and how much should be returned
        ///
        /// * Is the amount of the user asset the protocol got back greater than
        /// or equal to the amount that they initially put in?
        ///     * Yes: Return the same amount to them plus any fees from the
        ///     _user_ asset.
        ///     * No: Return to them all of the user asset the protocol got back
        ///     plus the amount required to buy back their missing amount or the
        ///     protocol assets returned when closing the liquidity position,
        ///     whichever one is smaller.
        ///
        /// Whatever the amount obtained from the algorithm defined at the top
        /// is the amount returned to the user. Some of the calculations take
        /// place in the adapters: specifically the estimation of fees.
        ///
        /// # Arguments
        ///
        /// `liquidity_receipt`: [`NonFungibleBucket`] - A bucket of the non
        /// fungible liquidity receipt.
        ///
        /// # Returns
        ///
        /// [`Vec<Bucket>`] - A vector of buckets of the amount to give back to
        /// the user.
        pub fn close_liquidity_position(
            &mut self,
            liquidity_receipt: NonFungibleBucket,
        ) -> Vec<Bucket> {
            // Ensure that we currently allow closing liquidity positions.
            assert!(
                self.is_close_position_enabled,
                "{}",
                CLOSING_LIQUIDITY_POSITIONS_IS_CLOSED_ERROR
            );
            // Ensure that there is only a single NFT in the bucket, we do not
            // service more than a single one at a time.
            assert!(
                liquidity_receipt.amount() == Decimal::ONE,
                "{}",
                MORE_THAN_ONE_LIQUIDITY_RECEIPT_NFTS_ERROR
            );

            let (
                mut adapter,
                liquidity_receipt_data,
                liquidity_receipt_global_id,
            ) = {
                // Reading the data of the non-fungible resource passed and then
                // validating that the resource address is what we expect. We do
                // this as we need to check it against the data of the blueprint
                // of the pool. So, that must be read first.
                let non_fungible =
                    liquidity_receipt.non_fungible::<LiquidityReceipt>();
                let liquidity_receipt_data = non_fungible.data();
                let (pool_adapter, liquidity_receipt_resource, _) = self
                    .checked_get_pool_adapter_and_liquidity_receipt(
                        liquidity_receipt_data.pool_address,
                    )
                    .expect(NO_ADAPTER_FOUND_FOR_POOL_ERROR);

                assert_eq!(
                    non_fungible.resource_address(),
                    liquidity_receipt_resource.address(),
                    "{}",
                    NOT_A_VALID_LIQUIDITY_RECEIPT_ERROR
                );

                // Burn the liquidity receipt
                liquidity_receipt.burn();

                // At this point, the non-fungible can be trusted to belong to
                // the liquidity receipt resource of the blueprint.
                (
                    pool_adapter,
                    liquidity_receipt_data,
                    non_fungible.global_id().clone(),
                )
            };

            // Assert that we're after the maturity date.
            assert!(
                Clock::current_time_is_at_or_after(
                    liquidity_receipt_data.maturity_date,
                    TimePrecision::Minute
                ),
                "{}",
                LIQUIDITY_POSITION_HAS_NOT_MATURED_ERROR
            );

            // Compare the price difference between the oracle reported price
            // and the pool reported price - ensure that it is within the
            // allowed price difference range.
            let oracle_reported_price = {
                let oracle_reported_price = self.checked_get_price(
                    liquidity_receipt_data.user_resource_address,
                    self.protocol_resource.address(),
                );
                let pool_reported_price =
                    adapter.price(liquidity_receipt_data.pool_address);
                let relative_difference = oracle_reported_price
                    .relative_difference(&pool_reported_price)
                    .expect(USER_ASSET_DOES_NOT_BELONG_TO_POOL_ERROR);

                assert!(
                    relative_difference
                        <= self.maximum_allowed_price_difference_percentage,
                    "{}",
                    RELATIVE_PRICE_DIFFERENCE_LARGER_THAN_ALLOWED_ERROR
                );

                oracle_reported_price
            };

            /* The liquidity position can be closed! */

            // Withdraw all of the pool units associated with the position and
            // close it through the adapter.
            let CloseLiquidityPositionOutput {
                resources,
                others,
                fees,
            } = {
                let pool_units = self
                    .pool_units
                    .get_mut(&liquidity_receipt_global_id)
                    .expect(UNEXPECTED_ERROR)
                    .take_all();
                adapter.close_liquidity_position(
                    liquidity_receipt_data.pool_address,
                    pool_units,
                )
            };

            let (mut user_resource_bucket, mut protocol_resource_bucket) = {
                let user_resource = resources
                    .get(&liquidity_receipt_data.user_resource_address)
                    .map(|item| Bucket(item.0))
                    .expect(UNEXPECTED_ERROR);
                let protocol_resource = resources
                    .get(&self.protocol_resource.address())
                    .map(|item| Bucket(item.0))
                    .expect(UNEXPECTED_ERROR);
                drop(resources);
                (user_resource, protocol_resource)
            };

            let user_resource_bucket_amount = user_resource_bucket.amount();
            let protocol_resource_bucket_amount =
                protocol_resource_bucket.amount();

            let (user_resource_fees, _) = {
                let user_resource = fees
                    .get(&liquidity_receipt_data.user_resource_address)
                    .copied()
                    .unwrap_or(Decimal::ZERO);
                let protocol_resource = fees
                    .get(&self.protocol_resource.address())
                    .copied()
                    .unwrap_or(Decimal::ZERO);
                drop(fees);
                (user_resource, protocol_resource)
            };

            // Determine the amount of resources that the user should be given
            // back.
            //
            // Branch 1: There is enough of the user asset to give the user back
            // the same amount that they put in.
            let (
                amount_of_protocol_resource_to_give_user,
                amount_of_user_resource_to_give_user,
            ) = if user_resource_bucket_amount
                >= liquidity_receipt_data.user_contribution_amount
            {
                let amount_of_protocol_resource_to_give_user = dec!(0);
                let amount_of_user_resource_to_give_user = min(
                    user_resource_bucket_amount,
                    liquidity_receipt_data.user_contribution_amount
                        + user_resource_fees,
                );

                (
                    amount_of_protocol_resource_to_give_user,
                    amount_of_user_resource_to_give_user,
                )
            }
            // Branch 2: There is not enough of the user token to given them
            // back the same amount that they put in.
            else {
                let amount_of_protocol_resource_to_give_user = {
                    let user_amount_missing = liquidity_receipt_data
                        .user_contribution_amount
                        - user_resource_bucket_amount;
                    let (_, protocol_resources_required_for_buy_back) =
                        oracle_reported_price
                            .exchange(
                                liquidity_receipt_data.user_resource_address,
                                user_amount_missing,
                            )
                            .expect(UNEXPECTED_ERROR);
                    min(
                        protocol_resources_required_for_buy_back,
                        protocol_resource_bucket_amount,
                    )
                };
                let amount_of_user_resource_to_give_user =
                    user_resource_bucket_amount;

                (
                    amount_of_protocol_resource_to_give_user,
                    amount_of_user_resource_to_give_user,
                )
            };

            let mut bucket_returns = others;
            bucket_returns.push(user_resource_bucket.take_advanced(
                amount_of_user_resource_to_give_user,
                WithdrawStrategy::Rounded(RoundingMode::ToZero),
            ));
            bucket_returns.push(protocol_resource_bucket.take_advanced(
                amount_of_protocol_resource_to_give_user,
                WithdrawStrategy::Rounded(RoundingMode::ToZero),
            ));

            // Deposit the remaining resources back into the protocol.
            self.deposit_resources(user_resource_bucket.as_fungible());
            self.deposit_resources(protocol_resource_bucket.as_fungible());

            // Return the buckets back
            bucket_returns
        }

        /// Updates the oracle adapter used by the protocol to a different
        /// adapter.
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
        pub fn set_oracle_adapter(&mut self, oracle_adapter: ComponentAddress) {
            self.oracle_adapter = oracle_adapter.into();
        }

        /// Sets the pool adapter that should be used by a pools belonging to a
        /// particular blueprint.
        ///
        /// Given the blueprint id of a pool whose information is already known
        /// to the protocol, this method changes it to use a new adapter instead
        /// of its existing one. All future opening and closing of liquidity
        /// positions happens through the new adapter.
        ///
        /// This method does not check that the provided adapter conforms to the
        /// [`PoolAdapter`] interface. It is the job of the caller to perform
        /// this check off-ledger.
        ///
        /// # Panics
        ///
        /// This function panics in the following cases:
        ///
        /// * If the provided address's blueprint has no corresponding
        /// blueprint.
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
        /// `pool_adapter`: [`ComponentAddress`] - The address of the adapter
        /// component.
        ///
        /// # Note
        ///
        /// This performs no interface checks and can theoretically accept the
        /// address of a component that does not implement the oracle interface.
        pub fn set_pool_adapter(
            &mut self,
            blueprint_id: BlueprintId,
            pool_adapter: ComponentAddress,
        ) {
            self.pool_information
                .get_mut(&blueprint_id)
                .expect(NO_ADAPTER_FOUND_FOR_POOL_ERROR)
                .adapter = pool_adapter.into();
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
        /// # Panics
        ///
        /// This function panics in two main cases:
        ///
        /// * If the provided address's blueprint has no corresponding
        /// blueprint.
        /// * If neither side of the pool is the protocol resource.
        ///
        /// # Access
        ///
        /// Requires the `protocol_manager` or `protocol_owner` roles.
        ///
        /// # Example Scenario
        ///
        /// We may wish to incentivize liquidity for a new bridged resource and a
        /// new set of pools. An even more compelling scenario, we may wish to
        /// provide incentives for a newly released DEX.
        ///
        /// # Arguments
        ///
        /// * `component`: [`ComponentAddress`] - The address of the pool
        /// component to add to the set of allowed pools.
        pub fn add_allowed_pool(&mut self, pool_address: ComponentAddress) {
            let protocol_resource_address = self.protocol_resource.address();
            self.with_pool_blueprint_information_mut(
                pool_address,
                |pool_information| {
                    let resources = pool_information
                        .adapter
                        .resource_addresses(pool_address);

                    assert!(
                        resources.0 == protocol_resource_address
                            || resources.1 == protocol_resource_address,
                        "{}",
                        NEITHER_POOL_RESOURCE_IS_PROTOCOL_RESOURCE_ERROR
                    );

                    pool_information.allowed_pools.insert(pool_address);
                },
            )
            .expect(NO_ADAPTER_FOUND_FOR_POOL_ERROR)
        }

        /// Removes one of the existing allowed liquidity pools.
        ///
        /// Given the component address of the liquidity pool, this method
        /// removes that liquidity pool from the list of allowed liquidity
        /// pools.
        ///
        /// # Panics
        ///
        /// This function panics in the following cases:
        ///
        /// * If the provided address's blueprint has no corresponding
        /// blueprint.
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
        pub fn remove_allowed_pool(&mut self, pool_address: ComponentAddress) {
            self.with_pool_blueprint_information_mut(
                pool_address,
                |pool_information| {
                    pool_information.allowed_pools.remove(&pool_address);
                },
            )
            .expect(NO_ADAPTER_FOUND_FOR_POOL_ERROR)
        }

        /// Sets the liquidity receipt resource associated with a particular
        /// pool blueprint.
        ///
        /// # Panics
        ///
        /// This function panics in the following cases:
        ///
        /// * If the provided address's blueprint has no corresponding
        /// blueprint.
        ///
        /// # Access
        ///
        /// Requires the `protocol_manager` or `protocol_owner` roles.
        ///
        /// # Arguments
        ///
        /// `blueprint_id``: [`BlueprintId`] - The blueprint id of the pool
        /// blueprint.
        /// `liquidity_receipt``: [`ResourceManager`] - The resource address of
        /// the new liquidity receipt resource to use.
        pub fn set_liquidity_receipt(
            &mut self,
            blueprint_id: BlueprintId,
            liquidity_receipt: ResourceManager,
        ) {
            self.pool_information
                .get_mut(&blueprint_id)
                .expect(NO_ADAPTER_FOUND_FOR_POOL_ERROR)
                .liquidity_receipt = liquidity_receipt;
        }

        /// Inserts the pool information, adding it to the protocol, performing
        /// an upsert.
        ///
        /// # Access
        ///
        /// Requires the `protocol_manager` or `protocol_owner` roles.
        ///
        /// # Arguments
        ///
        /// * `blueprint_id`: [`BlueprintId`] - The id of the pool blueprint
        /// to add the information for.
        /// * `PoolBlueprintInformation`: [`PoolBlueprintInformation`] The
        /// protocol information related to the blueprint.
        pub fn insert_pool_information(
            &mut self,
            blueprint_id: BlueprintId,
            pool_information: PoolBlueprintInformation,
        ) {
            self.pool_information.insert(blueprint_id, pool_information)
        }

        /// Removes the pool's blueprint information from the protocol.
        ///
        /// # Access
        ///
        /// Requires the `protocol_manager` or `protocol_owner` roles.
        ///
        /// # Arguments
        ///
        /// * `blueprint_id`: [`BlueprintId`] - The id of the pool blueprint
        /// to remove the information for.
        pub fn remove_pool_information(&mut self, blueprint_id: BlueprintId) {
            self.pool_information.remove(&blueprint_id);
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
        /// deposit other resources as well.
        ///
        /// # Arguments
        ///
        /// * `bucket`: [`FungibleBucket`] - A bucket of resources to deposit
        /// into the protocol.
        pub fn deposit_resources(&mut self, bucket: FungibleBucket) {
            let entry = self.vaults.get_mut(&bucket.resource_address());
            if let Some(mut vault) = entry {
                vault.put(bucket);
            } else {
                drop(entry);
                self.vaults.insert(
                    bucket.resource_address(),
                    FungibleVault::with_bucket(bucket),
                )
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
        pub fn withdraw_resources(
            &mut self,
            resource_address: ResourceAddress,
            amount: Decimal,
        ) -> FungibleBucket {
            self.vaults
                .get_mut(&resource_address)
                .expect(NO_ASSOCIATED_VAULT_ERROR)
                .take(amount)
        }

        /// Deposits pool units into the protocol.
        ///
        /// # Access
        ///
        /// Requires the `protocol_owner` role.
        ///
        /// # Arguments
        ///
        /// * `global_id`: [`NonFungibleGlobalId`] - The global id of the
        /// non-fungible liquidity position NFT whose associated pool units
        /// are to be deposited.
        /// * `pool_units`: [`Bucket`] - The pool units to deposit into the
        /// protocol.
        pub fn deposit_pool_units(
            &mut self,
            global_id: NonFungibleGlobalId,
            pool_units: Bucket,
        ) {
            let entry = self.pool_units.get_mut(&global_id);
            if let Some(mut vault) = entry {
                vault.put(pool_units);
            } else {
                drop(entry);
                self.pool_units
                    .insert(global_id, Vault::with_bucket(pool_units))
            }
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
        /// * [`Bucket`] - A bucket of the withdrawn tokens.
        pub fn withdraw_pool_units(
            &mut self,
            global_id: NonFungibleGlobalId,
        ) -> Bucket {
            self.pool_units
                .get_mut(&global_id)
                .expect(NO_ASSOCIATED_LIQUIDITY_RECEIPT_VAULT_ERROR)
                .take_all()
        }

        /// Updates the value of the maximum allowed price staleness used by
        /// the protocol.
        ///
        /// This means that any price checks that happen when opening or closing
        /// liquidity positions will be subjected to the new maximum allowed
        /// staleness.
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
        pub fn set_maximum_allowed_price_staleness(&mut self, value: i64) {
            self.maximum_allowed_price_staleness = value
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
        /// * `lockup_period`: [`LockupPeriod`] - The lockup period.
        /// * `rate`: [`Decimal`] - The rewards rate as a percent. This is a
        /// percentage value where 0 represents 0%, 0.5 represents 50% and 1
        /// represents 100%.
        pub fn add_reward_rate(
            &mut self,
            lockup_period: LockupPeriod,
            percentage: Decimal,
        ) {
            self.reward_rates.insert(lockup_period, percentage)
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
        /// * `lockup_period`: [`LockupPeriod`] - The lockup period in seconds
        /// associated with the rewards rate that we would like to remove.
        pub fn remove_reward_rate(&mut self, lockup_period: LockupPeriod) {
            self.reward_rates.remove(&lockup_period);
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
        /// * `value`: [`bool`] - Controls whether opening of liquidity
        /// positions is enabled or disabled.
        pub fn set_is_open_position_enabled(&mut self, value: bool) {
            self.is_open_position_enabled = value
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
        /// * `value`: [`bool`] - Controls whether closing of liquidity
        /// positions is enabled or disabled.
        pub fn set_is_close_position_enabled(&mut self, value: bool) {
            self.is_close_position_enabled = value
        }

        /// Updates the value of the maximum allowed price difference between
        /// the pool and the oracle.
        ///
        /// # Access
        ///
        /// Requires the `protocol_owner` or `protocol_manager` role.
        ///
        /// # Example Scenario
        ///
        /// As more and more arbitrage bots get created, we may want to make the
        /// price difference allowed narrower and narrower.
        ///
        /// # Arguments
        ///
        /// `value`: [`Decimal`] - The maximum allowed percentage difference.
        /// This is a percentage value where 0 represents 0%, 0.5 represents
        /// 50% and 1 represents 100%.
        pub fn set_maximum_allowed_price_difference_percentage(
            &mut self,
            value: Decimal,
        ) {
            self.maximum_allowed_price_difference_percentage = value
        }

        /// An internal method that is used to execute callbacks against the
        /// blueprint of some pool.
        fn with_pool_blueprint_information_mut<F, O>(
            &mut self,
            pool_address: ComponentAddress,
            callback: F,
        ) -> Option<O>
        where
            F: FnOnce(
                &mut KeyValueEntryRefMut<'_, PoolBlueprintInformation>,
            ) -> O,
        {
            let blueprint_id = ScryptoVmV1Api::object_get_blueprint_id(
                pool_address.as_node_id(),
            );
            let entry = self.pool_information.get_mut(&blueprint_id);
            entry.map(|mut entry| callback(&mut entry))
        }

        /// Gets the adapter and the liquidity receipt given a pool address.
        ///
        /// This method first gets the pool information associated with the pool
        /// blueprint and then checks to ensure that the pool is in the allow
        /// list of pools. If it is, it returns the adapter and the resource
        /// manager reference of the liquidity receipt.
        ///
        /// If a [`None`] is returned it means that no pool information was
        /// found for the pool and that it has no corresponding adapter that
        /// we can use.
        ///
        /// # Panics
        ///
        /// * If the pool is not in the list of allowed pools.
        ///
        /// # Arguments
        ///
        /// `pool_address`: [`ComponentAddress`] - The address of the component
        /// to get the adapter and liquidity receipt for.
        ///
        /// # Returns
        ///
        /// * [`PoolAdapter`] - The adapter to use for the pool.
        /// * [`ResourceManager`] - The resource manager reference of the
        /// liquidity receipt token.
        ///
        /// # Note
        ///
        /// The [`KeyValueEntryRef<'_, PoolBlueprintInformation>`] is returned
        /// to allow the references of the addresses to remain.
        fn checked_get_pool_adapter_and_liquidity_receipt(
            &self,
            pool_address: ComponentAddress,
        ) -> Option<(
            PoolAdapter,
            ResourceManager,
            KeyValueEntryRef<'_, PoolBlueprintInformation>,
        )> {
            let blueprint_id = ScryptoVmV1Api::object_get_blueprint_id(
                pool_address.as_node_id(),
            );
            let entry = self.pool_information.get(&blueprint_id);

            entry.map(|entry| {
                assert!(
                    entry.allowed_pools.contains(&pool_address),
                    "{}",
                    POOL_IS_NOT_IN_ALLOW_LIST_ERROR
                );

                (entry.adapter, entry.liquidity_receipt, entry)
            })
        }

        /// Gets the price of the `base` resource in terms of the `quote` resource
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
        /// [`Price`] - The price of the base resource in terms of the quote
        /// resource.
        fn checked_get_price(
            &self,
            base: ResourceAddress,
            quote: ResourceAddress,
        ) -> Price {
            // Get the price
            let (price, last_update) =
                self.oracle_adapter.get_price(base, quote);
            let final_price_validity = last_update
                .add_seconds(self.maximum_allowed_price_staleness)
                .unwrap();

            // Check for staleness
            assert!(
                Clock::current_time_is_at_or_before(
                    final_price_validity,
                    TimePrecision::Minute
                ),
                "{}",
                ORACLE_REPORTED_PRICE_IS_STALE_ERROR
            );

            // Return price
            price
        }
    }
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
