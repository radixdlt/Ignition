// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

#![allow(clippy::arithmetic_side_effects)]

use tests::prelude::*;

#[test]
fn can_open_a_simple_position_against_an_ociswap_pool(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        ociswap_v2,
        resources,
        ..
    } = ScryptoTestEnv::new()?;

    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;

    // Act
    let rtn = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        ociswap_v2.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    );

    // Assert
    let _ = rtn.expect("Should succeed!");

    Ok(())
}

#[test]
fn price_reported_by_pool_is_equal_to_price_reported_by_adapter(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut ociswap_v2,
        resources,
        ..
    } = ScryptoTestEnv::new()?;

    let bitcoin_bucket = ResourceManager(resources.bitcoin)
        .mint_fungible(dec!(10_000_000), env)?;
    let _ = ociswap_v2.pools.bitcoin.swap(bitcoin_bucket, env)?;

    // Act
    let pool_reported_price = ociswap_v2
        .pools
        .bitcoin
        .price_sqrt(env)?
        .checked_powi(2)
        .unwrap()
        .checked_truncate(RoundingMode::ToZero)
        .unwrap();
    let adapter_reported_price = ociswap_v2
        .adapter
        .price(ociswap_v2.pools.bitcoin.try_into().unwrap(), env)?
        .price;

    // Assert
    assert_eq!(pool_reported_price, adapter_reported_price);

    Ok(())
}

#[test]
fn can_open_a_liquidity_position_in_ociswap_that_fits_into_fee_limits() {
    // Arrange
    let ScryptoUnitEnv {
        environment: mut ledger,
        resources,
        protocol,
        ociswap_v2,
        ..
    } = ScryptoUnitEnv::new_with_configuration(Configuration {
        maximum_allowed_relative_price_difference: dec!(0.03),
        ..Default::default()
    });
    let (_, private_key, account_address, _) = protocol.protocol_owner_badge;

    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .mint_fungible(resources.bitcoin, dec!(100_000_000_000_000))
                .try_deposit_entire_worktop_or_abort(account_address, None)
                .build(),
            vec![],
        )
        .expect_commit_success();

    // Act
    let receipt = ledger.construct_and_execute_notarized_transaction(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(
                account_address,
                resources.bitcoin,
                dec!(100_000),
            )
            .take_all_from_worktop(resources.bitcoin, "bitcoin")
            .with_bucket("bitcoin", |builder, bucket| {
                builder.call_method(
                    protocol.ignition,
                    "open_liquidity_position",
                    (
                        bucket,
                        ociswap_v2.pools.bitcoin,
                        LockupPeriod::from_months(6).unwrap(),
                    ),
                )
            })
            .try_deposit_entire_worktop_or_abort(account_address, None)
            .build(),
        &private_key,
    );

    // Assert
    receipt.expect_commit_success();
    let TransactionFeeSummary {
        total_execution_cost_in_xrd,
        total_finalization_cost_in_xrd,
        total_tipping_cost_in_xrd,
        total_storage_cost_in_xrd,
        total_royalty_cost_in_xrd,
        ..
    } = receipt.fee_summary;

    assert!(
        dbg!(
            total_execution_cost_in_xrd
                + total_finalization_cost_in_xrd
                + total_tipping_cost_in_xrd
                + total_storage_cost_in_xrd
                + total_royalty_cost_in_xrd
        ) <= dec!(7)
    );
    assert!(total_execution_cost_in_xrd <= dec!(4.5))
}

#[test]
fn can_close_a_liquidity_position_in_ociswap_that_fits_into_fee_limits() {
    // Arrange
    let ScryptoUnitEnv {
        environment: mut ledger,
        resources,
        protocol,
        ociswap_v2,
        ..
    } = ScryptoUnitEnv::new_with_configuration(Configuration {
        maximum_allowed_relative_price_difference: dec!(0.03),
        ..Default::default()
    });
    let (public_key, private_key, account_address, _) =
        protocol.protocol_owner_badge;

    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .mint_fungible(resources.bitcoin, dec!(100_000_000_000_000))
                .try_deposit_entire_worktop_or_abort(account_address, None)
                .build(),
            vec![],
        )
        .expect_commit_success();

    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .withdraw_from_account(
                    account_address,
                    resources.bitcoin,
                    dec!(100_000),
                )
                .take_all_from_worktop(resources.bitcoin, "bitcoin")
                .with_bucket("bitcoin", |builder, bucket| {
                    builder.call_method(
                        protocol.ignition,
                        "open_liquidity_position",
                        (
                            bucket,
                            ociswap_v2.pools.bitcoin,
                            LockupPeriod::from_months(6).unwrap(),
                        ),
                    )
                })
                .try_deposit_entire_worktop_or_abort(account_address, None)
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        )
        .expect_commit_success();

    let current_time = ledger.get_current_time(TimePrecisionV2::Minute);
    let maturity_instant = current_time
        .add_seconds(*LockupPeriod::from_months(6).unwrap().seconds() as i64)
        .unwrap();
    {
        let db = ledger.substate_db_mut();
        let mut writer = SystemDatabaseWriter::new(db);

        writer
            .write_typed_object_field(
                CONSENSUS_MANAGER.as_node_id(),
                ModuleId::Main,
                ConsensusManagerField::ProposerMilliTimestamp.field_index(),
                ConsensusManagerProposerMilliTimestampFieldPayload::from_content_source(
                    ProposerMilliTimestampSubstate {
                        epoch_milli: maturity_instant.seconds_since_unix_epoch * 1000,
                    },
                ),
            )
            .unwrap();

        writer
            .write_typed_object_field(
                CONSENSUS_MANAGER.as_node_id(),
                ModuleId::Main,
                ConsensusManagerField::ProposerMinuteTimestamp.field_index(),
                ConsensusManagerProposerMinuteTimestampFieldPayload::from_content_source(
                    ProposerMinuteTimestampSubstate {
                        epoch_minute: i32::try_from(maturity_instant.seconds_since_unix_epoch / 60)
                            .unwrap(),
                    },
                ),
            )
            .unwrap();
    }

    ledger
        .execute_manifest_without_auth(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_method(
                    protocol.oracle,
                    "set_price",
                    (resources.bitcoin, XRD, dec!(1)),
                )
                .build(),
        )
        .expect_commit_success();

    // Act
    let receipt = ledger.construct_and_execute_notarized_transaction(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(
                account_address,
                ociswap_v2.liquidity_receipt,
                dec!(1),
            )
            .take_all_from_worktop(ociswap_v2.liquidity_receipt, "receipt")
            .with_bucket("receipt", |builder, bucket| {
                builder.call_method(
                    protocol.ignition,
                    "close_liquidity_position",
                    (bucket,),
                )
            })
            .try_deposit_entire_worktop_or_abort(account_address, None)
            .build(),
        &private_key,
    );

    // Assert
    receipt.expect_commit_success();
    let TransactionFeeSummary {
        total_execution_cost_in_xrd,
        total_finalization_cost_in_xrd,
        total_tipping_cost_in_xrd,
        total_storage_cost_in_xrd,
        total_royalty_cost_in_xrd,
        ..
    } = receipt.fee_summary;

    assert!(
        dbg!(
            total_execution_cost_in_xrd
                + total_finalization_cost_in_xrd
                + total_tipping_cost_in_xrd
                + total_storage_cost_in_xrd
                + total_royalty_cost_in_xrd
        ) <= dec!(7)
    );
    assert!(total_execution_cost_in_xrd <= dec!(4.5))
}

#[test]
fn contributions_to_ociswap_through_adapter_dont_fail_due_to_bucket_ordering(
) -> Result<(), RuntimeError> {
    // Arrange
    let mut results = Vec::<bool>::new();
    for order in [true, false] {
        // Arrange
        let Environment {
            environment: ref mut env,
            resources,
            mut ociswap_v2,
            ..
        } = ScryptoTestEnv::new()?;

        let xrd_bucket = ResourceManager(XRD).mint_fungible(dec!(1), env)?;
        let bitcoin_bucket =
            ResourceManager(resources.bitcoin).mint_fungible(dec!(1), env)?;
        let buckets = if order {
            (xrd_bucket, bitcoin_bucket)
        } else {
            (bitcoin_bucket, xrd_bucket)
        };

        // Act
        let result = ociswap_v2.adapter.open_liquidity_position(
            ociswap_v2.pools.bitcoin.try_into().unwrap(),
            buckets,
            LockupPeriod::default(),
            env,
        );
        results.push(result.is_ok());
    }

    // Assert
    assert_eq!(results.len(), 2);
    assert_eq!(results.iter().filter(|item| **item).count(), 2);

    Ok(())
}

#[test]
fn when_price_of_user_asset_stays_the_same_and_k_stays_the_same_the_output_is_the_same_amount_as_the_input(
) -> Result<(), RuntimeError> {
    non_strict_testing_of_fees(
        Movement::Same,
        Movement::Same,
        CloseLiquidityResult::SameAmount,
    )
}

#[test]
fn when_price_of_user_asset_stays_the_same_and_k_goes_down_the_output_is_the_same_amount_as_the_input(
) -> Result<(), RuntimeError> {
    non_strict_testing_of_fees(
        Movement::Down,
        Movement::Same,
        CloseLiquidityResult::SameAmount,
    )
}

#[test]
fn when_price_of_user_asset_stays_the_same_and_k_goes_up_the_output_is_the_same_amount_as_the_input(
) -> Result<(), RuntimeError> {
    non_strict_testing_of_fees(
        Movement::Up,
        Movement::Same,
        CloseLiquidityResult::SameAmount,
    )
}

#[test]
fn when_price_of_user_asset_goes_down_and_k_stays_the_same_the_user_gets_fees(
) -> Result<(), RuntimeError> {
    non_strict_testing_of_fees(
        Movement::Same,
        Movement::Down,
        CloseLiquidityResult::GetFees,
    )
}

#[test]
fn when_price_of_user_asset_goes_down_and_k_goes_down_the_user_gets_fees(
) -> Result<(), RuntimeError> {
    non_strict_testing_of_fees(
        Movement::Down,
        Movement::Down,
        CloseLiquidityResult::GetFees,
    )
}

#[test]
fn when_price_of_user_asset_goes_down_and_k_goes_up_the_user_gets_fees(
) -> Result<(), RuntimeError> {
    non_strict_testing_of_fees(
        Movement::Up,
        Movement::Down,
        CloseLiquidityResult::GetFees,
    )
}

#[test]
fn when_price_of_user_asset_goes_up_and_k_stays_the_same_the_user_gets_reimbursed(
) -> Result<(), RuntimeError> {
    non_strict_testing_of_fees(
        Movement::Same,
        Movement::Up,
        CloseLiquidityResult::Reimbursement,
    )
}

#[test]
fn when_price_of_user_asset_goes_up_and_k_goes_down_the_user_gets_reimbursed(
) -> Result<(), RuntimeError> {
    non_strict_testing_of_fees(
        Movement::Down,
        Movement::Up,
        CloseLiquidityResult::Reimbursement,
    )
}

#[test]
fn when_price_of_user_asset_goes_up_and_k_goes_up_the_user_gets_reimbursed(
) -> Result<(), RuntimeError> {
    non_strict_testing_of_fees(
        Movement::Up,
        Movement::Up,
        CloseLiquidityResult::Reimbursement,
    )
}

fn non_strict_testing_of_fees(
    protocol_coefficient: Movement,
    price_of_user_asset: Movement,
    result: CloseLiquidityResult,
) -> Result<(), RuntimeError> {
    let Environment {
        environment: ref mut env,
        mut protocol,
        mut ociswap_v2,
        resources,
        ..
    } = ScryptoTestEnv::new()?;

    let pool_reported_price = ociswap_v2
        .adapter
        .price(ociswap_v2.pools.bitcoin.try_into().unwrap(), env)?;
    protocol.oracle.set_price(
        pool_reported_price.base,
        pool_reported_price.quote,
        pool_reported_price.price,
        env,
    )?;

    let bitcoin_amount_in = dec!(100);

    let bitcoin_bucket = ResourceManager(resources.bitcoin)
        .mint_fungible(bitcoin_amount_in, env)?;
    let (receipt, _, _) = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        ociswap_v2.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    )?;

    let pool_units = ociswap_v2
        .adapter
        .open_liquidity_position(
            ociswap_v2.pools.bitcoin.try_into().unwrap(),
            (
                ResourceManager(resources.bitcoin)
                    .mint_fungible(dec!(100_000), env)?,
                ResourceManager(XRD).mint_fungible(dec!(100_000), env)?,
            ),
            LockupPeriod::default(),
            env,
        )?
        .pool_units
        .into_values()
        .next()
        .unwrap();

    match price_of_user_asset {
        // User asset price goes down - i.e., we inject it into the pool.
        Movement::Down => {
            let bitcoin_bucket = ResourceManager(resources.bitcoin)
                .mint_fungible(dec!(450_000_000), env)?;
            let _ = ociswap_v2.pools.bitcoin.swap(bitcoin_bucket, env)?;
        }
        // The user asset price stays the same. We do not do anything.
        Movement::Same => {}
        // User asset price goes up - i.e., we reduce it in the pool.
        Movement::Up => {
            let xrd_bucket =
                ResourceManager(XRD).mint_fungible(dec!(450_000_000), env)?;
            let _ = ociswap_v2.pools.bitcoin.swap(xrd_bucket, env)?;
        }
    }

    match protocol_coefficient {
        // Somebody claimed some portion of the pool
        Movement::Down => {
            let _ = ociswap_v2
                .pools
                .bitcoin
                .remove_liquidity(NonFungibleBucket(pool_units), env)?;
        }
        // Nothing
        Movement::Same => {}
        // Somebody contributed to the pool some amount
        Movement::Up => {
            let _ = ociswap_v2
                .adapter
                .open_liquidity_position(
                    ociswap_v2.pools.bitcoin.try_into().unwrap(),
                    (
                        ResourceManager(resources.bitcoin)
                            .mint_fungible(dec!(100_000), env)?,
                        ResourceManager(XRD)
                            .mint_fungible(dec!(100_000), env)?,
                    ),
                    LockupPeriod::default(),
                    env,
                )?
                .pool_units;
        }
    }

    env.set_current_time(Instant::new(
        *LockupPeriod::from_months(12).unwrap().seconds() as i64,
    ));
    let pool_reported_price = ociswap_v2
        .adapter
        .price(ociswap_v2.pools.bitcoin.try_into().unwrap(), env)?;
    protocol.oracle.set_price(
        pool_reported_price.base,
        pool_reported_price.quote,
        pool_reported_price.price,
        env,
    )?;

    let buckets = IndexedBuckets::native_from_buckets(
        protocol.ignition.close_liquidity_position(receipt, env)?,
        env,
    )?;

    let bitcoin_amount_out = buckets
        .get(&resources.bitcoin)
        .map(|bucket| bucket.amount(env).unwrap())
        .unwrap_or_default()
        .checked_round(5, RoundingMode::ToPositiveInfinity)
        .unwrap();
    let xrd_amount_out = buckets
        .get(&XRD)
        .map(|bucket| bucket.amount(env).unwrap())
        .unwrap_or_default()
        .checked_round(5, RoundingMode::ToZero)
        .unwrap();

    match result {
        CloseLiquidityResult::GetFees => {
            // Bitcoin we get back must be strictly greater than what we put in.
            assert!(bitcoin_amount_out > bitcoin_amount_in);
            // When we get back fees we MUST not get back any XRD
            assert_eq!(xrd_amount_out, Decimal::ZERO)
        }
        CloseLiquidityResult::SameAmount => {
            // Bitcoin we get back must be strictly equal to what we put in.
            assert_eq!(bitcoin_amount_out, bitcoin_amount_in);
            // If we get back the same amount then we must NOT get back any XRD.
            assert_eq!(xrd_amount_out, Decimal::ZERO)
        }
        CloseLiquidityResult::Reimbursement => {
            // Bitcoin we get back must be less than what we put in.
            assert!(bitcoin_amount_out < bitcoin_amount_in);
            // We must get back SOME xrd.
            assert_ne!(xrd_amount_out, Decimal::ZERO);
        }
    }

    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Movement {
    Down,
    Same,
    Up,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CloseLiquidityResult {
    GetFees,
    SameAmount,
    Reimbursement,
}

macro_rules! define_price_test {
    (
        $($multiplier: expr),* $(,)?
    ) => {
        paste::paste! {
            $(
                #[test]
                fn [<positions_can_be_opened_at_current_price_and_closed_at_a_ $multiplier x_price_decrease>](
                ) {
                    test_effect_of_price_action_on_fees(- $multiplier )
                }

                #[test]
                fn [<positions_can_be_opened_at_current_price_and_closed_at_a_ $multiplier x_price_increase>](
                ) {
                    test_effect_of_price_action_on_fees($multiplier )
                }
            )*
        }
    };
}

define_price_test! {
    100,
    90,
    80,
    70,
    60,
    50,
    40,
    30,
    20,
    10,
}

fn test_effect_of_price_action_on_fees(multiplier: i32) {
    let ScryptoUnitEnv {
        environment: mut ledger,
        protocol,
        ociswap_v2,
        resources,
        ..
    } = ScryptoUnitEnv::new();
    let (_, private_key, account_address, _) = protocol.protocol_owner_badge;

    // We will be using the bitcoin pool for this test and the bitcoin resource.
    let pool_address = ociswap_v2.pools.bitcoin;
    let pool_resource = resources.bitcoin;

    // Getting the address of the x and y asset to differentiate them from one
    // another
    let (resource_x, resource_y) = {
        let commit_result = ledger
            .execute_manifest(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .ociswap_v2_pool_x_address(pool_address)
                    .ociswap_v2_pool_y_address(pool_address)
                    .build(),
                vec![],
            )
            .expect_commit_success()
            .clone();

        (
            commit_result.output::<ResourceAddress>(1),
            commit_result.output::<ResourceAddress>(2),
        )
    };

    // Adding liquidity between the smallest and largest ticks possible.
    let price = ledger
        .execute_manifest_with_enabled_modules(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .mint_fungible(resource_x, dec!(1_000_000_000_000))
                .mint_fungible(resource_y, dec!(1_000_000_000_000))
                .take_all_from_worktop(resource_x, "resource_x")
                .take_all_from_worktop(resource_y, "resource_y")
                .with_name_lookup(|builder, namer| {
                    let resource_x = namer.bucket("resource_x");
                    let resource_y = namer.bucket("resource_y");

                    builder.ociswap_v2_pool_add_liquidity(
                        pool_address,
                        -887272,
                        887272,
                        resource_x,
                        resource_y,
                    )
                })
                .ociswap_v2_pool_price_sqrt(pool_address)
                .deposit_batch(account_address)
                .build(),
            EnabledModules::for_notarized_transaction() & !EnabledModules::AUTH,
        )
        .expect_commit_success()
        .output::<PreciseDecimal>(6)
        .checked_powi(2)
        .and_then(|value| Decimal::try_from(value).ok())
        .unwrap();

    // Adding this pool to Ignition.
    ledger
        .execute_manifest_without_auth(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_method(
                    protocol.ignition,
                    "add_allowed_pool",
                    (pool_address,),
                )
                .build(),
        )
        .expect_commit_success();

    // Adding this pool to Ignition.
    ledger
        .execute_manifest_without_auth(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_method(
                    protocol.ignition,
                    "add_allowed_pool",
                    (pool_address,),
                )
                .build(),
        )
        .expect_commit_success();

    // Updating the price in the Oracle component.
    ledger
        .execute_manifest_without_auth(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_method(
                    protocol.oracle,
                    "set_price",
                    (resource_x, resource_y, price),
                )
                .call_method(
                    protocol.oracle,
                    "set_price",
                    (resource_y, resource_x, 1 / price),
                )
                .build(),
        )
        .expect_commit_success();

    // Minting some of the resource and depositing them into the user's
    // account.
    ledger
        .execute_manifest_without_auth(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .mint_fungible(pool_resource, dec!(100_000))
                .deposit_batch(account_address)
                .build(),
        )
        .expect_commit_success();

    let receipt = ledger.construct_and_execute_notarized_transaction(
        ManifestBuilder::new()
            .lock_fee(account_address, dec!(10))
            .withdraw_from_account(account_address, pool_resource, dec!(1000))
            .take_all_from_worktop(pool_resource, "bucket")
            .with_bucket("bucket", |builder, bucket| {
                builder.call_method(
                    protocol.ignition,
                    "open_liquidity_position",
                    (
                        bucket,
                        pool_address,
                        LockupPeriod::from_months(6).unwrap(),
                    ),
                )
            })
            .deposit_batch(account_address)
            .build(),
        &private_key,
    );
    receipt.expect_commit_success();
    println!(
        "Open - Multiplier = {}x, Cost = {} XRD, Execution Cost = {} XRD",
        multiplier,
        receipt.fee_summary.total_cost(),
        receipt.fee_summary.total_execution_cost_in_xrd
    );

    // Set the current time to be 6 months from now.
    {
        let current_time = ledger.get_current_time(TimePrecisionV2::Minute);
        let maturity_instant =
            current_time
                .add_seconds(
                    *LockupPeriod::from_months(6).unwrap().seconds() as i64
                )
                .unwrap();
        let db = ledger.substate_db_mut();
        let mut writer = SystemDatabaseWriter::new(db);

        writer
            .write_typed_object_field(
                CONSENSUS_MANAGER.as_node_id(),
                ModuleId::Main,
                ConsensusManagerField::ProposerMilliTimestamp.field_index(),
                ConsensusManagerProposerMilliTimestampFieldPayload::from_content_source(
                    ProposerMilliTimestampSubstate {
                        epoch_milli: maturity_instant.seconds_since_unix_epoch * 1000,
                    },
                ),
            )
            .unwrap();

        writer
            .write_typed_object_field(
                CONSENSUS_MANAGER.as_node_id(),
                ModuleId::Main,
                ConsensusManagerField::ProposerMinuteTimestamp.field_index(),
                ConsensusManagerProposerMinuteTimestampFieldPayload::from_content_source(
                    ProposerMinuteTimestampSubstate {
                        epoch_minute: i32::try_from(maturity_instant.seconds_since_unix_epoch / 60)
                            .unwrap(),
                    },
                ),
            )
            .unwrap();
    }

    // Move the price according to the specified multiplier
    let new_price = if multiplier.is_positive() {
        let target_price = price * multiplier;
        let input_resource = resource_y;
        let amount_in_each_swap = dec!(100_000_000_000);

        let mut new_price = price;
        while new_price < target_price {
            let reported_price = ledger
                .execute_manifest_without_auth(
                    ManifestBuilder::new()
                        .lock_fee_from_faucet()
                        .mint_fungible(input_resource, amount_in_each_swap)
                        .take_all_from_worktop(input_resource, "bucket")
                        .with_bucket("bucket", |builder, bucket| {
                            builder.ociswap_v2_pool_swap(pool_address, bucket)
                        })
                        .deposit_batch(account_address)
                        .ociswap_v2_pool_price_sqrt(pool_address)
                        .build(),
                )
                .expect_commit_success()
                .output::<PreciseDecimal>(5)
                .checked_powi(2)
                .and_then(|value| Decimal::try_from(value).ok())
                .unwrap();

            if reported_price == new_price {
                break;
            } else {
                new_price = reported_price
            }
        }

        new_price
    } else {
        let target_price = price / multiplier * dec!(-1);
        let input_resource = resource_x;
        let amount_in_each_swap = dec!(100_000_000_000);

        let mut new_price = price;
        while new_price > target_price {
            let reported_price = ledger
                .execute_manifest_without_auth(
                    ManifestBuilder::new()
                        .lock_fee_from_faucet()
                        .mint_fungible(input_resource, amount_in_each_swap)
                        .take_all_from_worktop(input_resource, "bucket")
                        .with_bucket("bucket", |builder, bucket| {
                            builder.ociswap_v2_pool_swap(pool_address, bucket)
                        })
                        .deposit_batch(account_address)
                        .ociswap_v2_pool_price_sqrt(pool_address)
                        .build(),
                )
                .expect_commit_success()
                .output::<PreciseDecimal>(5)
                .checked_powi(2)
                .and_then(|value| Decimal::try_from(value).ok())
                .unwrap();

            if reported_price == new_price {
                break;
            } else {
                new_price = reported_price
            }
        }

        new_price
    };

    // Submit the new price to the oracle
    ledger
        .execute_manifest_without_auth(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_method(
                    protocol.oracle,
                    "set_price",
                    (resource_x, resource_y, new_price),
                )
                .call_method(
                    protocol.oracle,
                    "set_price",
                    (resource_y, resource_x, 1 / new_price),
                )
                .build(),
        )
        .expect_commit_success();

    // Close the position
    let receipt = ledger.construct_and_execute_notarized_transaction(
        ManifestBuilder::new()
            .lock_fee(account_address, dec!(10))
            .withdraw_from_account(
                account_address,
                ociswap_v2.liquidity_receipt,
                dec!(1),
            )
            .take_all_from_worktop(ociswap_v2.liquidity_receipt, "bucket")
            .with_bucket("bucket", |builder, bucket| {
                builder.call_method(
                    protocol.ignition,
                    "close_liquidity_position",
                    (bucket,),
                )
            })
            .deposit_batch(account_address)
            .build(),
        &private_key,
    );
    receipt.expect_commit_success();
    println!(
        "Close - Multiplier = {}x, Cost = {} XRD, Execution Cost = {} XRD",
        multiplier,
        receipt.fee_summary.total_cost(),
        receipt.fee_summary.total_execution_cost_in_xrd
    );
}

#[test]
fn user_resources_are_contributed_in_full_when_oracle_price_is_same_as_pool_price(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        mut ociswap_v2,
        resources,
        ..
    } = ScryptoTestEnv::new()?;

    let pool = ComponentAddress::try_from(ociswap_v2.pools.bitcoin).unwrap();
    let user_resource = resources.bitcoin;

    let pool_price = ociswap_v2.adapter.price(pool, env)?;
    protocol.oracle.set_price(
        pool_price.base,
        pool_price.quote,
        pool_price.price,
        env,
    )?;

    let user_resource_bucket =
        ResourceManager(user_resource).mint_fungible(dec!(100), env)?;

    // Act
    let (_, _, mut change) = protocol.ignition.open_liquidity_position(
        FungibleBucket(user_resource_bucket),
        pool,
        LockupPeriod::from_months(6).unwrap(),
        env,
    )?;

    // Assert
    assert_eq!(change.len(), 1);
    let change = change.pop().unwrap();

    let change_resource_address = change.resource_address(env)?;
    let change_amount = change.amount(env)?;
    assert_eq!(change_resource_address, user_resource);
    assert_eq!(
        change_amount,
        dec!(0),
        "Change != 0, Change is {}",
        change_amount
    );

    Ok(())
}

#[test]
fn user_resources_are_contributed_in_full_when_oracle_price_is_higher_than_pool_price(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        mut ociswap_v2,
        resources,
        ..
    } = ScryptoTestEnv::new_with_configuration(Configuration {
        maximum_allowed_relative_price_difference: dec!(0.05),
        ..Default::default()
    })?;

    let pool = ComponentAddress::try_from(ociswap_v2.pools.bitcoin).unwrap();
    let user_resource = resources.bitcoin;

    let pool_price = ociswap_v2.adapter.price(pool, env)?;
    protocol.oracle.set_price(
        pool_price.base,
        pool_price.quote,
        pool_price.price * dec!(1.05),
        env,
    )?;

    let user_resource_bucket =
        ResourceManager(user_resource).mint_fungible(dec!(100), env)?;

    // Act
    let (_, _, mut change) = protocol.ignition.open_liquidity_position(
        FungibleBucket(user_resource_bucket),
        pool,
        LockupPeriod::from_months(6).unwrap(),
        env,
    )?;

    // Assert
    assert_eq!(change.len(), 1);
    let change = change.pop().unwrap();

    let change_resource_address = change.resource_address(env)?;
    let change_amount = change.amount(env)?;
    assert_eq!(change_resource_address, user_resource);
    assert_eq!(
        change_amount,
        dec!(0),
        "Change != 0, Change is {}",
        change_amount
    );

    Ok(())
}

#[test]
fn user_resources_are_contributed_in_full_when_oracle_price_is_lower_than_pool_price(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        mut ociswap_v2,
        resources,
        ..
    } = ScryptoTestEnv::new_with_configuration(Configuration {
        maximum_allowed_relative_price_difference: dec!(0.05),
        ..Default::default()
    })?;

    let pool = ComponentAddress::try_from(ociswap_v2.pools.bitcoin).unwrap();
    let user_resource = resources.bitcoin;

    let pool_price = ociswap_v2.adapter.price(pool, env)?;
    protocol.oracle.set_price(
        pool_price.base,
        pool_price.quote,
        pool_price.price * dec!(0.96),
        env,
    )?;

    let user_resource_bucket =
        ResourceManager(user_resource).mint_fungible(dec!(100), env)?;

    // Act
    let (_, _, mut change) = protocol.ignition.open_liquidity_position(
        FungibleBucket(user_resource_bucket),
        pool,
        LockupPeriod::from_months(6).unwrap(),
        env,
    )?;

    // Assert
    assert_eq!(change.len(), 1);
    let change = change.pop().unwrap();

    let change_resource_address = change.resource_address(env)?;
    let change_amount = change.amount(env)?;
    assert_eq!(change_resource_address, user_resource);
    assert_eq!(
        change_amount,
        dec!(0),
        "Change != 0, Change is {}",
        change_amount
    );

    Ok(())
}
