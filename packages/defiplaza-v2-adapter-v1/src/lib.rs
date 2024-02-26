mod blueprint_interface;
pub use blueprint_interface::*;

use common::prelude::*;
use ports_interface::prelude::*;
use scrypto::prelude::*;
use scrypto_interface::*;

macro_rules! define_error {
    (
        $(
            $name: ident => $item: expr;
        )*
    ) => {
        $(
            pub const $name: &'static str = concat!("[DefiPlaza v2 Adapter v2]", " ", $item);
        )*
    };
}

define_error! {
    RESOURCE_DOESNT_BELONG_TO_POOL => "Resources don't belong to pool";
    OVERFLOW_ERROR => "Calculation overflowed.";
    UNEXPECTED_ERROR => "Unexpected Error.";
    INVALID_NUMBER_OF_BUCKETS => "Invalid number of buckets.";
}

macro_rules! pool {
    ($address: expr) => {
        $crate::blueprint_interface::DefiPlazaV2PoolInterfaceScryptoStub::from(
            $address,
        )
    };
}

#[blueprint_with_traits]
pub mod adapter {
    struct DefiPlazaV2Adapter;

    impl DefiPlazaV2Adapter {
        pub fn instantiate(
            metadata_init: MetadataInit,
            owner_role: OwnerRole,
            address_reservation: Option<GlobalAddressReservation>,
        ) -> Global<DefiPlazaV2Adapter> {
            let address_reservation =
                address_reservation.unwrap_or_else(|| {
                    Runtime::allocate_component_address(BlueprintId {
                        package_address: Runtime::package_address(),
                        blueprint_name: Runtime::blueprint_name(),
                    })
                    .0
                });

            Self {}
                .instantiate()
                .prepare_to_globalize(owner_role)
                .metadata(ModuleConfig {
                    init: metadata_init,
                    roles: Default::default(),
                })
                .with_address(address_reservation)
                .globalize()
        }

        pub fn liquidity_receipt_data(
            // Does not depend on state, this is kept in case this is required
            // in the future for whatever reason.
            &self,
            global_id: NonFungibleGlobalId,
        ) -> LiquidityReceipt<DefiPlazaV2AdapterSpecificInformation> {
            // Read the non-fungible data.
            let LiquidityReceipt {
                name,
                lockup_period,
                pool_address,
                user_resource_address,
                user_contribution_amount,
                user_resource_volatility_classification,
                protocol_contribution_amount,
                maturity_date,
                adapter_specific_information,
            } = ResourceManager::from_address(global_id.resource_address())
                .get_non_fungible_data::<LiquidityReceipt<AnyValue>>(
                global_id.local_id(),
            );
            let adapter_specific_information = adapter_specific_information
                .as_typed::<DefiPlazaV2AdapterSpecificInformation>()
                .unwrap();

            LiquidityReceipt {
                name,
                lockup_period,
                pool_address,
                user_resource_address,
                user_contribution_amount,
                user_resource_volatility_classification,
                protocol_contribution_amount,
                maturity_date,
                adapter_specific_information,
            }
        }
    }

    impl PoolAdapterInterfaceTrait for DefiPlazaV2Adapter {
        fn open_liquidity_position(
            &mut self,
            pool_address: ComponentAddress,
            buckets: (Bucket, Bucket),
        ) -> OpenLiquidityPositionOutput {
            // When opening a liquidity position we follow the algorithm that
            // Jazzer described to us:
            //
            // 1) state = pair.get_state()
            // 2) see which token is in shortage by inspecting state.shortage
            // 3) store lp1_original_target = state.target_ratio * bucket1.amount()
            //    where bucket1 is the token in shortage.
            // 4) (lp1, remainder_bucket) = pair.add_liquidity(bucket1, bucket2)
            //    and store the resulting lp1 tokens
            // 5) store lp2_original_target = remainder_bucket.amount()
            // 6) call (lp2, remainder2) =
            //    pair.add_liquidity(remainder_bucket.expect(), None) and store
            //    the resulting lp2 tokens (remainder2 will be None)

            let mut pool = pool!(pool_address);
            let (base_resource_address, quote_resource_address) =
                pool.get_tokens();

            // Ensure that the passed buckets belong to the pool and sort them
            // into base and quote buckets.
            let (base_bucket, quote_bucket) = {
                let bucket_address1 = buckets.0.resource_address();
                let bucket_address2 = buckets.1.resource_address();

                if bucket_address1 == base_resource_address
                    && bucket_address2 == quote_resource_address
                {
                    (buckets.0, buckets.1)
                } else if bucket_address2 == base_resource_address
                    && bucket_address1 == quote_resource_address
                {
                    (buckets.1, buckets.0)
                } else {
                    panic!("{}", RESOURCE_DOESNT_BELONG_TO_POOL)
                }
            };

            // Step 1: Get the pair's state
            let pair_state = pool.get_state();

            // Step 2: Determine which of the resources is in shortage. The one
            // in shortage is the one that we will be contributing first to the
            // pool. If the pool is in equilibrium then we can pick any of the
            // two resources as the first (shortage) resource. In the code here
            // "first" and "second" refer to which one will be contributed first
            // and which will be contributed second.
            let shortage = pair_state.shortage;
            let shortage_state = ShortageState::from(shortage);

            let [(first_resource_address, first_bucket), (second_resource_address, second_bucket)] =
                match shortage_state {
                    ShortageState::Equilibrium => [
                        (base_resource_address, base_bucket),
                        (quote_resource_address, quote_bucket),
                    ],
                    ShortageState::Shortage(Asset::Base) => [
                        (base_resource_address, base_bucket),
                        (quote_resource_address, quote_bucket),
                    ],
                    ShortageState::Shortage(Asset::Quote) => [
                        (quote_resource_address, quote_bucket),
                        (base_resource_address, base_bucket),
                    ],
                };

            // Step 3: Calculate tate.target_ratio * bucket1.amount() where
            // bucket1 is the bucket currently in shortage or the resource that
            // will be contributed first.
            let first_original_target = pair_state
                .target_ratio
                .checked_mul(first_bucket.amount())
                .expect(OVERFLOW_ERROR);

            // Step 4: Contribute to the pool. The first bucket to provide the
            // pool is the bucket of the asset in shortage or the asset that we
            // now refer to as "first" and then followed by the "second" bucket.
            let (first_pool_units, second_change) =
                pool.add_liquidity(first_bucket, Some(second_bucket));

            // Step 5: Calculate and store the original target of the second
            // liquidity position. This is calculated as the amount of assets
            // that are in the remainder (change) bucket.
            let second_bucket = second_change.expect(UNEXPECTED_ERROR);
            let second_original_target = second_bucket.amount();

            // Step 6: Add liquidity with the second resource & no co-liquidity.
            let (second_pool_units, change) =
                pool.add_liquidity(second_bucket, None);

            // TODO: Should we subtract the change from the second original
            // target? Seems like we should if the price if not the same in
            // some way?

            // A sanity check to make sure that everything is correct. The pool
            // units obtained from the first contribution should be different
            // from those obtained in the second contribution.
            assert_ne!(
                first_pool_units.resource_address(),
                second_pool_units.resource_address(),
            );

            // The procedure for adding liquidity to the pool is now complete.
            // We can now construct the output.
            OpenLiquidityPositionOutput {
                pool_units: IndexedBuckets::from_buckets([
                    first_pool_units,
                    second_pool_units,
                ]),
                change: change
                    .map(IndexedBuckets::from_bucket)
                    .unwrap_or_default(),
                others: vec![],
                adapter_specific_information:
                    DefiPlazaV2AdapterSpecificInformation {
                        original_targets: indexmap! {
                            first_resource_address => first_original_target,
                            second_resource_address => second_original_target
                        },
                    }
                    .into(),
            }
        }

        fn close_liquidity_position(
            &mut self,
            pool_address: ComponentAddress,
            mut pool_units: Vec<Bucket>,
            adapter_specific_information: AnyValue,
        ) -> CloseLiquidityPositionOutput {
            // When closing a position we follow the algorithm Jazzer described
            // to us:
            //
            // 1) state = pair.get_state()
            // 2) see which token is in shortage by inspecting state.shortage
            // 3) calculate new base target and quote target. Suppose base token
            //    is in shortage: new_base_target = state.target_ratio * current
            //    base tokens represented by base_LP. The new_quote_target will
            //    just be equal to the current quote tokens represented by
            //    quote_LP.
            // 4) base_fees = new_base_target - original_base_target
            // 5) quote_fees = new_quote_target - original_quote_target
            // 6) settle

            let pool = pool!(pool_address);

            let (pool_units1, pool_units2) = {
                let pool_units_bucket1 =
                    pool_units.pop().expect(INVALID_NUMBER_OF_BUCKETS);
                let pool_units_bucket2 =
                    pool_units.pop().expect(INVALID_NUMBER_OF_BUCKETS);
                if !pool_units.is_empty() {
                    panic!("{}", INVALID_NUMBER_OF_BUCKETS)
                }
                (pool_units_bucket1, pool_units_bucket2)
            };

            // Getting the base and quote assets
            let (base_resource_address, quote_resource_address) =
                pool.get_tokens();

            // Decoding the adapter specific information as the type we expect
            // it to be.
            let DefiPlazaV2AdapterSpecificInformation { original_targets } =
                adapter_specific_information.as_typed().unwrap();
            let [old_base_target, old_quote_target] =
                [base_resource_address, quote_resource_address].map(
                    |address| original_targets.get(&address).copied().unwrap(),
                );

            // Step 1: Get the pair's state
            let pair_state = pool.get_state();

            // Step 2 & 3: Determine which of the resources is in shortage and
            // based on that determine what the new target should be.
            let claimed_tokens = IndexedBuckets::from_buckets(
                [pool_units1, pool_units2].into_iter().flat_map(|bucket| {
                    let resource_manager = bucket.resource_manager();
                    let entry = ComponentAddress::try_from(
                        resource_manager
                            .get_metadata::<_, GlobalAddress>("pool")
                            .unwrap()
                            .unwrap(),
                    )
                    .unwrap();
                    let mut two_resource_pool =
                        Global::<TwoResourcePool>::from(entry);
                    let (bucket1, bucket2) = two_resource_pool.redeem(bucket);
                    [bucket1, bucket2]
                }),
            );
            let base_bucket =
                claimed_tokens.get(&base_resource_address).unwrap();
            let quote_bucket =
                claimed_tokens.get(&quote_resource_address).unwrap();

            let base_bucket_amount = base_bucket.amount();
            let quote_bucket_amount = quote_bucket.amount();

            let shortage = pair_state.shortage;
            let shortage_state = ShortageState::from(shortage);
            let (new_base_target, new_quote_target) = match shortage_state {
                ShortageState::Equilibrium => {
                    (base_bucket_amount, quote_bucket_amount)
                }
                ShortageState::Shortage(Asset::Base) => (
                    base_bucket_amount
                        .checked_mul(pair_state.target_ratio)
                        .expect(OVERFLOW_ERROR),
                    quote_bucket_amount,
                ),
                ShortageState::Shortage(Asset::Quote) => (
                    base_bucket_amount,
                    quote_bucket_amount
                        .checked_mul(pair_state.target_ratio)
                        .expect(OVERFLOW_ERROR),
                ),
            };

            // Steps 4 and 5
            let base_fees = std::cmp::min(
                new_base_target
                    .checked_sub(old_base_target)
                    .expect(OVERFLOW_ERROR),
                Decimal::ZERO,
            );
            let quote_fees = std::cmp::min(
                new_quote_target
                    .checked_sub(old_quote_target)
                    .expect(OVERFLOW_ERROR),
                Decimal::ZERO,
            );

            CloseLiquidityPositionOutput {
                resources: claimed_tokens,
                others: vec![],
                fees: indexmap! {
                    base_resource_address => base_fees,
                    quote_resource_address => quote_fees,
                },
            }
        }

        fn price(&mut self, pool_address: ComponentAddress) -> Price {
            // TODO: Still not sure how to find the price of assets in DefiPlaza
            // and I'm working with them on that. For now, I will just say that
            // the price is one. WE MUST CHANGE THIS BEFORE GOING LIVE!
            //
            // More information: The price for selling and buying the asset is
            // different in DefiPlaza just like an order book (they're not an
            // order book though). So, there is no current price that you can
            // buy and sell at but two prices depending on what resource the
            // input is.
            let pool = pool!(pool_address);
            let (base_asset, quote_asset) = pool.get_tokens();
            Price {
                base: base_asset,
                quote: quote_asset,
                price: dec!(1),
            }
        }

        fn resource_addresses(
            &mut self,
            pool_address: ComponentAddress,
        ) -> (ResourceAddress, ResourceAddress) {
            let pool = pool!(pool_address);
            let two_resource_pool =
                Global::<TwoResourcePool>::from(pool.get_pools().0);

            let mut resource_addresses =
                two_resource_pool.get_vault_amounts().into_keys();

            let resource_address1 =
                resource_addresses.next().expect(UNEXPECTED_ERROR);
            let resource_address2 =
                resource_addresses.next().expect(UNEXPECTED_ERROR);

            (resource_address1, resource_address2)
        }
    }
}

#[derive(ScryptoSbor, Debug, Clone)]
pub struct DefiPlazaV2AdapterSpecificInformation {
    pub original_targets: IndexMap<ResourceAddress, Decimal>,
}

impl From<DefiPlazaV2AdapterSpecificInformation> for AnyValue {
    fn from(value: DefiPlazaV2AdapterSpecificInformation) -> Self {
        AnyValue::from_typed(&value).unwrap()
    }
}
