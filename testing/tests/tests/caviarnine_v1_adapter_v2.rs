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
fn can_open_a_simple_position_against_a_caviarnine_pool(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        mut caviarnine_v1,
        resources,
        ..
    } = ScryptoTestEnv::new_with_configuration(Configuration {
        caviarnine_adapter_version: CaviarnineAdapterVersion::Two,
        ..Default::default()
    })?;
    protocol
        .ignition
        .set_maximum_allowed_price_difference_percentage(dec!(0.50), env)?;
    caviarnine_v1
        .adapter
        .upsert_pool_contribution_bin_configuration(
            caviarnine_v1.pools.bitcoin.try_into().unwrap(),
            ContributionBinConfiguration {
                start_tick: 26900,
                end_tick: 27100,
            },
            env,
        )?;

    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;

    // Act
    let rtn = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        caviarnine_v1.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    );

    // Assert
    let _ = rtn.expect("Should succeed!");

    Ok(())
}

#[test]
fn liquidity_receipt_information_can_be_read_through_adapter(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        mut caviarnine_v1,
        resources,
        ..
    } = ScryptoTestEnv::new_with_configuration(Configuration {
        caviarnine_adapter_version: CaviarnineAdapterVersion::Two,
        ..Default::default()
    })?;
    protocol
        .ignition
        .set_maximum_allowed_price_difference_percentage(dec!(0.50), env)?;
    caviarnine_v1
        .adapter
        .upsert_pool_contribution_bin_configuration(
            caviarnine_v1.pools.bitcoin.try_into().unwrap(),
            ContributionBinConfiguration {
                start_tick: 26900,
                end_tick: 27100,
            },
            env,
        )?;

    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;

    let (receipt, ..) = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        caviarnine_v1.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    )?;

    // Act
    let data = caviarnine_v1.adapter.liquidity_receipt_data(
        NonFungibleGlobalId::new(
            receipt.0.resource_address(env)?,
            receipt
                .0
                .non_fungible_local_ids(env)?
                .first()
                .unwrap()
                .clone(),
        ),
        env,
    )?;

    // Assert
    assert_eq!(data.adapter_specific_information.bin_contributions.len(), 3);

    Ok(())
}

#[test]
fn can_open_a_liquidity_position_in_caviarnine_that_fits_into_fee_limits() {
    // Arrange
    let ScryptoUnitEnv {
        environment: mut ledger,
        resources,
        protocol,
        caviarnine_v1,
        ..
    } = ScryptoUnitEnv::new_with_configuration(Configuration {
        maximum_allowed_relative_price_difference: dec!(0.03),
        caviarnine_adapter_version: CaviarnineAdapterVersion::Two,
        ..Default::default()
    });
    let (_, private_key, account_address, _) = protocol.protocol_owner_badge;

    ledger
        .execute_manifest_without_auth(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_method(
                    caviarnine_v1.adapter,
                    "upsert_pool_contribution_bin_configuration",
                    (
                        caviarnine_v1.pools.bitcoin,
                        ContributionBinConfiguration {
                            start_tick: 22000,
                            end_tick: 27000,
                        },
                    ),
                )
                .mint_fungible(resources.bitcoin, dec!(100_000_000_000_000))
                .try_deposit_entire_worktop_or_abort(account_address, None)
                .build(),
        )
        .expect_commit_success();

    ledger
        .execute_manifest_with_enabled_modules(
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
                            caviarnine_v1.pools.bitcoin,
                            LockupPeriod::from_months(6).unwrap(),
                        ),
                    )
                })
                .try_deposit_entire_worktop_or_abort(account_address, None)
                .build(),
            EnabledModules::for_test_transaction()
                & !EnabledModules::AUTH
                & !EnabledModules::COSTING,
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
                        caviarnine_v1.pools.bitcoin,
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
        ..
    } = receipt.fee_summary;
    println!(
        "Execution cost to open a position: {} XRD",
        total_execution_cost_in_xrd
    );

    assert!(total_execution_cost_in_xrd <= dec!(4.8))
}

#[test]
fn can_close_a_liquidity_position_in_caviarnine_that_fits_into_fee_limits() {
    // Arrange
    let ScryptoUnitEnv {
        environment: mut ledger,
        resources,
        protocol,
        caviarnine_v1,
        ..
    } = ScryptoUnitEnv::new_with_configuration(Configuration {
        maximum_allowed_relative_price_difference: dec!(0.03),
        caviarnine_adapter_version: CaviarnineAdapterVersion::Two,
        ..Default::default()
    });
    let (_, private_key, account_address, _) = protocol.protocol_owner_badge;

    ledger
        .execute_manifest_without_auth(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_method(
                    caviarnine_v1.adapter,
                    "upsert_pool_contribution_bin_configuration",
                    (
                        caviarnine_v1.pools.bitcoin,
                        ContributionBinConfiguration {
                            start_tick: 22000,
                            end_tick: 27000,
                        },
                    ),
                )
                .mint_fungible(resources.bitcoin, dec!(100_000_000_000_000))
                .try_deposit_entire_worktop_or_abort(account_address, None)
                .build(),
        )
        .expect_commit_success();

    for _ in 0..2 {
        ledger
            .execute_manifest_with_enabled_modules(
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
                                caviarnine_v1.pools.bitcoin,
                                LockupPeriod::from_months(6).unwrap(),
                            ),
                        )
                    })
                    .try_deposit_entire_worktop_or_abort(account_address, None)
                    .build(),
                EnabledModules::for_test_transaction()
                    & !EnabledModules::AUTH
                    & !EnabledModules::COSTING,
            )
            .expect_commit_success();
    }

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
                caviarnine_v1.liquidity_receipt,
                dec!(1),
            )
            .take_all_from_worktop(caviarnine_v1.liquidity_receipt, "receipt")
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
        ..
    } = receipt.fee_summary;
    println!(
        "Execution cost to close a position: {} XRD",
        total_execution_cost_in_xrd
    );

    assert!(total_execution_cost_in_xrd <= dec!(4.8))
}

#[test]
fn contributions_directly_to_caviarnine_could_fail_due_to_bucket_order(
) -> Result<(), RuntimeError> {
    // Arrange
    let mut results = Vec::<bool>::new();
    for order in [true, false] {
        // Arrange
        let Environment {
            environment: ref mut env,
            resources,
            mut caviarnine_v1,
            ..
        } = ScryptoTestEnv::new_with_configuration(Configuration {
            caviarnine_adapter_version: CaviarnineAdapterVersion::Two,
            ..Default::default()
        })?;

        let xrd_bucket = ResourceManager(XRD).mint_fungible(dec!(1), env)?;
        let bitcoin_bucket =
            ResourceManager(resources.bitcoin).mint_fungible(dec!(1), env)?;
        let buckets = if order {
            (xrd_bucket, bitcoin_bucket)
        } else {
            (bitcoin_bucket, xrd_bucket)
        };

        // Act
        let result = caviarnine_v1.pools.bitcoin.add_liquidity(
            buckets.0,
            buckets.1,
            vec![(27000, dec!(1), dec!(1))],
            env,
        );
        results.push(result.is_ok());
    }

    // Assert
    assert_eq!(results.len(), 2);
    assert_eq!(results.iter().filter(|item| **item).count(), 1);
    assert_eq!(results.iter().filter(|item| !**item).count(), 1);

    Ok(())
}

#[test]
fn contributions_to_caviarnine_through_adapter_dont_fail_due_to_bucket_ordering(
) -> Result<(), RuntimeError> {
    // Arrange
    let mut results = Vec::<bool>::new();
    for order in [true, false] {
        // Arrange
        let Environment {
            environment: ref mut env,
            resources,
            mut caviarnine_v1,
            ..
        } = ScryptoTestEnv::new_with_configuration(Configuration {
            caviarnine_adapter_version: CaviarnineAdapterVersion::Two,
            ..Default::default()
        })?;
        caviarnine_v1
            .adapter
            .upsert_pool_contribution_bin_configuration(
                caviarnine_v1.pools.bitcoin.try_into().unwrap(),
                ContributionBinConfiguration {
                    start_tick: 26900,
                    end_tick: 27100,
                },
                env,
            )?;

        let xrd_bucket = ResourceManager(XRD).mint_fungible(dec!(1), env)?;
        let bitcoin_bucket =
            ResourceManager(resources.bitcoin).mint_fungible(dec!(1), env)?;
        let buckets = if order {
            (xrd_bucket, bitcoin_bucket)
        } else {
            (bitcoin_bucket, xrd_bucket)
        };

        // Act
        let result = caviarnine_v1.adapter.open_liquidity_position(
            caviarnine_v1.pools.bitcoin.try_into().unwrap(),
            buckets,
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
fn liquidity_receipt_includes_the_amount_of_liquidity_positions_we_expect_to_see(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        resources,
        mut caviarnine_v1,
        ..
    } = ScryptoTestEnv::new_with_configuration(Configuration {
        caviarnine_adapter_version: CaviarnineAdapterVersion::Two,
        ..Default::default()
    })?;
    caviarnine_v1
        .adapter
        .upsert_pool_contribution_bin_configuration(
            caviarnine_v1.pools.bitcoin.try_into().unwrap(),
            ContributionBinConfiguration {
                start_tick: 26900,
                end_tick: 27100,
            },
            env,
        )?;
    protocol
        .ignition
        .set_maximum_allowed_price_difference_percentage(dec!(0.50), env)?;

    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;

    let (liquidity_receipt, _, _) = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        caviarnine_v1.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    )?;

    // Act
    let liquidity_receipt_data =
        ResourceManager(caviarnine_v1.liquidity_receipt)
            .get_non_fungible_data::<_, _, LiquidityReceipt<AnyValue>>(
                liquidity_receipt
                    .0
                    .non_fungible_local_ids(env)?
                    .first()
                    .unwrap()
                    .clone(),
                env,
            )?;

    // Assert
    let adapter_information = liquidity_receipt_data
        .adapter_specific_information
        .as_typed::<CaviarnineV1AdapterSpecificInformation>()
        .unwrap();
    assert_eq!(adapter_information.bin_contributions.len(), 3);

    Ok(())
}

#[test]
fn contribution_amount_reported_in_receipt_nft_matches_caviarnine_state(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        mut caviarnine_v1,
        resources,
        ..
    } = ScryptoTestEnv::new_with_configuration(Configuration {
        caviarnine_adapter_version: CaviarnineAdapterVersion::Two,
        ..Default::default()
    })?;
    caviarnine_v1
        .adapter
        .upsert_pool_contribution_bin_configuration(
            caviarnine_v1.pools.bitcoin.try_into().unwrap(),
            ContributionBinConfiguration {
                start_tick: 26900,
                end_tick: 27100,
            },
            env,
        )?;
    protocol
        .ignition
        .set_maximum_allowed_price_difference_percentage(dec!(0.50), env)?;

    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;

    // Act
    let (ignition_receipt, ..) = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        caviarnine_v1.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    )?;

    // Assert
    let ignition_receipt_global_id = {
        let local_id = ignition_receipt
            .0
            .non_fungible_local_ids(env)?
            .first()
            .unwrap()
            .clone();
        NonFungibleGlobalId::new(caviarnine_v1.liquidity_receipt, local_id)
    };
    let ignition_receipt_data =
        ResourceManager(caviarnine_v1.liquidity_receipt)
            .get_non_fungible_data::<_, _, LiquidityReceipt<AnyValue>>(
                ignition_receipt
                    .0
                    .non_fungible_local_ids(env)?
                    .first()
                    .unwrap()
                    .clone(),
                env,
            )?;

    let caviarnine_receipt = protocol
        .ignition
        .withdraw_pool_units(ignition_receipt_global_id, env)?
        .pop()
        .unwrap();

    let mut caviarnine_reported_contributions =
        caviarnine_v1.pools.bitcoin.get_redemption_bin_values(
            caviarnine_receipt
                .non_fungible_local_ids(env)?
                .first()
                .unwrap()
                .clone(),
            env,
        )?;
    caviarnine_reported_contributions.sort_by(|a, b| a.0.cmp(&b.0));

    let adapter_reported_contributions = ignition_receipt_data
        .adapter_specific_information
        .as_typed::<CaviarnineV1AdapterSpecificInformation>()
        .unwrap()
        .contributions();

    assert_eq!(
        &adapter_reported_contributions.len(),
        &caviarnine_reported_contributions.len()
    );
    for (x, y) in adapter_reported_contributions
        .into_iter()
        .zip(caviarnine_reported_contributions)
    {
        assert_eq!(x.0, y.0);
        assert!(approximately_equals(x.1, y.1));
        assert!(approximately_equals(x.2, y.2));
    }

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
        mut caviarnine_v1,
        resources,
        ..
    } = ScryptoTestEnv::new_with_configuration(Configuration {
        caviarnine_adapter_version: CaviarnineAdapterVersion::Two,
        ..Default::default()
    })?;
    caviarnine_v1
        .adapter
        .upsert_pool_contribution_bin_configuration(
            caviarnine_v1.pools.bitcoin.try_into().unwrap(),
            ContributionBinConfiguration {
                start_tick: 26900,
                end_tick: 27100,
            },
            env,
        )?;

    let pool_reported_price = caviarnine_v1
        .adapter
        .price(caviarnine_v1.pools.bitcoin.try_into().unwrap(), env)?;
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
        caviarnine_v1.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    )?;

    let pool_units = caviarnine_v1
        .adapter
        .open_liquidity_position(
            caviarnine_v1.pools.bitcoin.try_into().unwrap(),
            (
                ResourceManager(resources.bitcoin)
                    .mint_fungible(dec!(100_000), env)?,
                ResourceManager(XRD).mint_fungible(dec!(100_000), env)?,
            ),
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
                .mint_fungible(dec!(100_000), env)?;
            let _ = caviarnine_v1.pools.bitcoin.swap(bitcoin_bucket, env)?;
        }
        // The user asset price stays the same. We do not do anything.
        Movement::Same => {}
        // User asset price goes up - i.e., we reduce it in the pool.
        Movement::Up => {
            let xrd_bucket =
                ResourceManager(XRD).mint_fungible(dec!(100_000), env)?;
            let _ = caviarnine_v1.pools.bitcoin.swap(xrd_bucket, env)?;
        }
    }

    match protocol_coefficient {
        // Somebody claimed some portion of the pool
        Movement::Down => {
            let _ = caviarnine_v1
                .pools
                .bitcoin
                .remove_liquidity(pool_units, env)?;
        }
        // Nothing
        Movement::Same => {}
        // Somebody contributed to the pool some amount
        Movement::Up => {
            let _ = caviarnine_v1
                .adapter
                .open_liquidity_position(
                    caviarnine_v1.pools.bitcoin.try_into().unwrap(),
                    (
                        ResourceManager(resources.bitcoin)
                            .mint_fungible(dec!(100_000), env)?,
                        ResourceManager(XRD)
                            .mint_fungible(dec!(100_000), env)?,
                    ),
                    env,
                )?
                .pool_units;
        }
    }

    env.set_current_time(Instant::new(
        *LockupPeriod::from_months(12).unwrap().seconds() as i64,
    ));
    let pool_reported_price = caviarnine_v1
        .adapter
        .price(caviarnine_v1.pools.bitcoin.try_into().unwrap(), env)?;
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

#[allow(clippy::arithmetic_side_effects)]
fn approximately_equals(a: Decimal, b: Decimal) -> bool {
    let difference = match (a == Decimal::ZERO, b == Decimal::ZERO) {
        (true, true) => dec!(0),
        (true, false) => (b - a).checked_abs().unwrap() / b,
        (false, true) => (b - a).checked_abs().unwrap() / a,
        (false, false) => (b - a).checked_abs().unwrap() / b,
    };
    difference <= dec!(0.015)
}

#[test]
fn price_and_active_tick_reported_by_adapter_must_match_whats_reported_by_pool(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut caviarnine_v1,
        resources,
        ..
    } = ScryptoTestEnv::new_with_configuration(Configuration {
        caviarnine_adapter_version: CaviarnineAdapterVersion::Two,
        ..Default::default()
    })?;
    caviarnine_v1
        .adapter
        .upsert_pool_contribution_bin_configuration(
            caviarnine_v1.pools.bitcoin.try_into().unwrap(),
            ContributionBinConfiguration {
                start_tick: 26900,
                end_tick: 27100,
            },
            env,
        )?;
    env.disable_limits_module();

    let bin_span = 100u32;
    for bin in (0u32..=54000u32).step_by(bin_span as usize) {
        let mut pool = CaviarnineV1PoolInterfaceScryptoTestStub::new(
            rule!(allow_all),
            rule!(allow_all),
            XRD,
            resources.bitcoin,
            bin_span,
            None,
            caviarnine_v1.package,
            env,
        )?;

        let bucket_x = ResourceManager(XRD).mint_fungible(dec!(100), env)?;
        let bucket_y =
            ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;

        let _ = pool.add_liquidity(
            bucket_x,
            bucket_y,
            vec![(bin, dec!(100), dec!(100))],
            env,
        )?;

        // Act
        let (adapter_reported_price, adapter_reported_active_tick) =
            caviarnine_v1
                .adapter
                .price_and_active_tick(
                    pool.try_into().unwrap(),
                    Some(PoolInformation {
                        bin_span,
                        resources: ResourceIndexedData {
                            resource_x: XRD,
                            resource_y: resources.bitcoin,
                        },
                    }),
                    env,
                )?
                .unwrap();

        // Assert
        let pool_reported_price = pool.get_price(env)?.unwrap();
        let pool_reported_active_tick = pool.get_active_tick(env)?.unwrap();
        assert_eq!(adapter_reported_price, pool_reported_price);
        assert_eq!(adapter_reported_active_tick, pool_reported_active_tick);
    }

    Ok(())
}

#[test]
fn l_is_equal_between_left_and_right() -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        mut caviarnine_v1,
        resources,
        ..
    } = ScryptoTestEnv::new_with_configuration(Configuration {
        caviarnine_adapter_version: CaviarnineAdapterVersion::Two,
        ..Default::default()
    })?;
    caviarnine_v1
        .adapter
        .upsert_pool_contribution_bin_configuration(
            caviarnine_v1.pools.bitcoin.try_into().unwrap(),
            ContributionBinConfiguration {
                start_tick: 26900,
                end_tick: 27100,
            },
            env,
        )?;
    protocol
        .ignition
        .set_maximum_allowed_price_difference_percentage(dec!(0.50), env)?;

    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;
    let pool = caviarnine_v1.pools.bitcoin;
    let bin_span = pool.get_bin_span(env)?;

    // Act
    let (receipt, ..) = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        pool.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    )?;

    // Assert
    let data = caviarnine_v1.adapter.liquidity_receipt_data(
        NonFungibleGlobalId::new(
            receipt.0.resource_address(env)?,
            receipt
                .0
                .non_fungible_local_ids(env)?
                .first()
                .unwrap()
                .clone(),
        ),
        env,
    )?;

    let liquidity_y = data
        .adapter_specific_information
        .bin_contributions
        .iter()
        .map(|(tick, amounts)| {
            let lower_price = tick_to_spot(*tick).unwrap();
            let upper_price = if !amounts.resource_x.is_zero() {
                pool.get_price(env).unwrap().unwrap()
            } else {
                tick_to_spot(*tick + bin_span).unwrap()
            };

            let lower_price_sqrt = lower_price.checked_sqrt().unwrap();
            let upper_price_sqrt = upper_price.checked_sqrt().unwrap();
            amounts.resource_y / (upper_price_sqrt - lower_price_sqrt)
        })
        .fold(Decimal::ZERO, |acc, item| acc + item);

    let liquidity_x = data
        .adapter_specific_information
        .bin_contributions
        .iter()
        .map(|(tick, amounts)| {
            let lower_price = if !amounts.resource_y.is_zero() {
                pool.get_price(env).unwrap().unwrap()
            } else {
                tick_to_spot(*tick).unwrap()
            };
            let upper_price = tick_to_spot(*tick + bin_span).unwrap();

            let lower_price_sqrt = lower_price.checked_sqrt().unwrap();
            let upper_price_sqrt = upper_price.checked_sqrt().unwrap();

            amounts.resource_x * (upper_price_sqrt * lower_price_sqrt)
                / (upper_price_sqrt - lower_price_sqrt)
        })
        .fold(Decimal::ZERO, |acc, item| acc + item);

    assert!(
        std::cmp::min(liquidity_x, liquidity_y)
            / std::cmp::max(liquidity_x, liquidity_y)
            > dec!(0.99)
    );

    Ok(())
}

#[test]
fn user_resources_are_contributed_in_full_when_oracle_price_is_same_as_pool_price(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        mut caviarnine_v1,
        resources,
        ..
    } = ScryptoTestEnv::new_with_configuration(Configuration {
        caviarnine_adapter_version: CaviarnineAdapterVersion::Two,
        ..Default::default()
    })?;
    caviarnine_v1
        .adapter
        .upsert_pool_contribution_bin_configuration(
            caviarnine_v1.pools.bitcoin.try_into().unwrap(),
            ContributionBinConfiguration {
                start_tick: 26900,
                end_tick: 27100,
            },
            env,
        )?;

    let pool = ComponentAddress::try_from(caviarnine_v1.pools.bitcoin).unwrap();
    let user_resource = resources.bitcoin;

    let pool_price = caviarnine_v1.adapter.price(pool, env)?;
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
        mut caviarnine_v1,
        resources,
        ..
    } = ScryptoTestEnv::new_with_configuration(Configuration {
        maximum_allowed_relative_price_difference: dec!(0.05),
        caviarnine_adapter_version: CaviarnineAdapterVersion::Two,
        ..Default::default()
    })?;
    caviarnine_v1
        .adapter
        .upsert_pool_contribution_bin_configuration(
            caviarnine_v1.pools.bitcoin.try_into().unwrap(),
            ContributionBinConfiguration {
                start_tick: 26900,
                end_tick: 27100,
            },
            env,
        )?;

    let pool = ComponentAddress::try_from(caviarnine_v1.pools.bitcoin).unwrap();
    let user_resource = resources.bitcoin;

    let pool_price = caviarnine_v1.adapter.price(pool, env)?;
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
        mut caviarnine_v1,
        resources,
        ..
    } = ScryptoTestEnv::new_with_configuration(Configuration {
        maximum_allowed_relative_price_difference: dec!(0.05),
        caviarnine_adapter_version: CaviarnineAdapterVersion::Two,
        ..Default::default()
    })?;
    caviarnine_v1
        .adapter
        .upsert_pool_contribution_bin_configuration(
            caviarnine_v1.pools.bitcoin.try_into().unwrap(),
            ContributionBinConfiguration {
                start_tick: 26900,
                end_tick: 27100,
            },
            env,
        )?;

    let pool = ComponentAddress::try_from(caviarnine_v1.pools.bitcoin).unwrap();
    let user_resource = resources.bitcoin;

    let pool_price = caviarnine_v1.adapter.price(pool, env)?;
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

#[test]
fn bin_amounts_reported_on_receipt_match_whats_reported_by_caviarnine(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut caviarnine_v1,
        resources,
        ..
    } = ScryptoTestEnv::new_with_configuration(Configuration {
        maximum_allowed_relative_price_difference: dec!(0.05),
        caviarnine_adapter_version: CaviarnineAdapterVersion::Two,
        ..Default::default()
    })?;
    caviarnine_v1
        .adapter
        .upsert_pool_contribution_bin_configuration(
            caviarnine_v1.pools.bitcoin.try_into().unwrap(),
            ContributionBinConfiguration {
                start_tick: 26900,
                end_tick: 27100,
            },
            env,
        )?;

    let user_resource = resources.bitcoin;
    let pool = caviarnine_v1.pools.bitcoin;

    let [user_resource_bucket, xrd_bucket] =
        [user_resource, XRD].map(|resource| {
            ResourceManager(resource)
                .mint_fungible(dec!(100), env)
                .unwrap()
        });

    // Act
    let OpenLiquidityPositionOutput {
        pool_units,
        adapter_specific_information,
        ..
    } = caviarnine_v1.adapter.open_liquidity_position(
        pool.try_into().unwrap(),
        (user_resource_bucket, xrd_bucket),
        env,
    )?;

    // Assert
    let mut caviarnine_reported_redemption_value = pool
        .get_redemption_bin_values(
            pool_units
                .into_values()
                .next()
                .unwrap()
                .non_fungible_local_ids(env)?
                .first()
                .unwrap()
                .clone(),
            env,
        )?;
    caviarnine_reported_redemption_value.sort_by(|a, b| a.0.cmp(&b.0));
    let adapter_reported_redemption_value = adapter_specific_information
        .as_typed::<CaviarnineV1AdapterSpecificInformation>()
        .unwrap()
        .bin_contributions;

    assert_eq!(
        caviarnine_reported_redemption_value.len(),
        adapter_reported_redemption_value.len(),
    );

    for (
        i,
        (
            caviarnine_reported_bin,
            caviarnine_reported_amount_x,
            caviarnine_reported_amount_y,
        ),
    ) in caviarnine_reported_redemption_value.into_iter().enumerate()
    {
        let Some(ResourceIndexedData {
            resource_x: adapter_reported_amount_x,
            resource_y: adapter_reported_amount_y,
        }) = adapter_reported_redemption_value
            .get(&caviarnine_reported_bin)
            .copied()
        else {
            panic!(
                "Bin {} does not have an entry in the adapter data",
                caviarnine_reported_bin
            )
        };

        assert_eq!(
            round_down_to_5_decimal_places(caviarnine_reported_amount_x),
            round_down_to_5_decimal_places(adapter_reported_amount_x),
            "Failed at bin with index: {i}"
        );
        assert_eq!(
            round_down_to_5_decimal_places(caviarnine_reported_amount_y),
            round_down_to_5_decimal_places(adapter_reported_amount_y),
            "Failed at bin with index: {i}"
        );
    }

    Ok(())
}

#[test]
fn bin_amounts_reported_on_receipt_match_whats_reported_by_caviarnine_with_price_movement1(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut caviarnine_v1,
        resources,
        ..
    } = ScryptoTestEnv::new_with_configuration(Configuration {
        maximum_allowed_relative_price_difference: dec!(0.05),
        caviarnine_adapter_version: CaviarnineAdapterVersion::Two,
        ..Default::default()
    })?;
    caviarnine_v1
        .adapter
        .upsert_pool_contribution_bin_configuration(
            caviarnine_v1.pools.bitcoin.try_into().unwrap(),
            ContributionBinConfiguration {
                start_tick: 26900,
                end_tick: 27100,
            },
            env,
        )?;

    let user_resource = resources.bitcoin;
    let mut pool = caviarnine_v1.pools.bitcoin;

    let _ = ResourceManager(user_resource)
        .mint_fungible(dec!(10_000), env)
        .and_then(|bucket| pool.swap(bucket, env))?;

    let [user_resource_bucket, xrd_bucket] =
        [user_resource, XRD].map(|resource| {
            ResourceManager(resource)
                .mint_fungible(dec!(100), env)
                .unwrap()
        });

    // Act
    let OpenLiquidityPositionOutput {
        pool_units,
        adapter_specific_information,
        ..
    } = caviarnine_v1.adapter.open_liquidity_position(
        pool.try_into().unwrap(),
        (user_resource_bucket, xrd_bucket),
        env,
    )?;

    // Assert
    let mut caviarnine_reported_redemption_value = pool
        .get_redemption_bin_values(
            pool_units
                .into_values()
                .next()
                .unwrap()
                .non_fungible_local_ids(env)?
                .first()
                .unwrap()
                .clone(),
            env,
        )?;
    caviarnine_reported_redemption_value.sort_by(|a, b| a.0.cmp(&b.0));
    let adapter_reported_redemption_value = adapter_specific_information
        .as_typed::<CaviarnineV1AdapterSpecificInformation>()
        .unwrap()
        .bin_contributions;

    assert_eq!(
        caviarnine_reported_redemption_value.len(),
        adapter_reported_redemption_value.len(),
    );

    for (
        i,
        (
            caviarnine_reported_bin,
            caviarnine_reported_amount_x,
            caviarnine_reported_amount_y,
        ),
    ) in caviarnine_reported_redemption_value.into_iter().enumerate()
    {
        let Some(ResourceIndexedData {
            resource_x: adapter_reported_amount_x,
            resource_y: adapter_reported_amount_y,
        }) = adapter_reported_redemption_value
            .get(&caviarnine_reported_bin)
            .copied()
        else {
            panic!(
                "Bin {} does not have an entry in the adapter data",
                caviarnine_reported_bin
            )
        };

        assert_eq!(
            round_down_to_5_decimal_places(caviarnine_reported_amount_x),
            round_down_to_5_decimal_places(adapter_reported_amount_x),
            "Failed at bin with index: {i}"
        );
        assert_eq!(
            round_down_to_5_decimal_places(caviarnine_reported_amount_y),
            round_down_to_5_decimal_places(adapter_reported_amount_y),
            "Failed at bin with index: {i}"
        );
    }

    Ok(())
}

#[test]
fn bin_amounts_reported_on_receipt_match_whats_reported_by_caviarnine_with_price_movement2(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut caviarnine_v1,
        resources,
        ..
    } = ScryptoTestEnv::new_with_configuration(Configuration {
        maximum_allowed_relative_price_difference: dec!(0.05),
        caviarnine_adapter_version: CaviarnineAdapterVersion::Two,
        ..Default::default()
    })?;
    caviarnine_v1
        .adapter
        .upsert_pool_contribution_bin_configuration(
            caviarnine_v1.pools.bitcoin.try_into().unwrap(),
            ContributionBinConfiguration {
                start_tick: 26900,
                end_tick: 27100,
            },
            env,
        )?;

    let user_resource = resources.bitcoin;
    let mut pool = caviarnine_v1.pools.bitcoin;

    let _ = ResourceManager(XRD)
        .mint_fungible(dec!(10_000), env)
        .and_then(|bucket| pool.swap(bucket, env))?;

    let [user_resource_bucket, xrd_bucket] =
        [user_resource, XRD].map(|resource| {
            ResourceManager(resource)
                .mint_fungible(dec!(100), env)
                .unwrap()
        });

    // Act
    let OpenLiquidityPositionOutput {
        pool_units,
        adapter_specific_information,
        ..
    } = caviarnine_v1.adapter.open_liquidity_position(
        pool.try_into().unwrap(),
        (user_resource_bucket, xrd_bucket),
        env,
    )?;

    // Assert
    let mut caviarnine_reported_redemption_value = pool
        .get_redemption_bin_values(
            pool_units
                .into_values()
                .next()
                .unwrap()
                .non_fungible_local_ids(env)?
                .first()
                .unwrap()
                .clone(),
            env,
        )?;
    caviarnine_reported_redemption_value.sort_by(|a, b| a.0.cmp(&b.0));
    let adapter_reported_redemption_value = adapter_specific_information
        .as_typed::<CaviarnineV1AdapterSpecificInformation>()
        .unwrap()
        .bin_contributions;

    assert_eq!(
        caviarnine_reported_redemption_value.len(),
        adapter_reported_redemption_value.len(),
    );

    for (
        i,
        (
            caviarnine_reported_bin,
            caviarnine_reported_amount_x,
            caviarnine_reported_amount_y,
        ),
    ) in caviarnine_reported_redemption_value.into_iter().enumerate()
    {
        let Some(ResourceIndexedData {
            resource_x: adapter_reported_amount_x,
            resource_y: adapter_reported_amount_y,
        }) = adapter_reported_redemption_value
            .get(&caviarnine_reported_bin)
            .copied()
        else {
            panic!(
                "Bin {} does not have an entry in the adapter data",
                caviarnine_reported_bin
            )
        };

        assert_eq!(
            round_down_to_5_decimal_places(caviarnine_reported_amount_x),
            round_down_to_5_decimal_places(adapter_reported_amount_x),
            "Failed at bin with index: {i}"
        );
        assert_eq!(
            round_down_to_5_decimal_places(caviarnine_reported_amount_y),
            round_down_to_5_decimal_places(adapter_reported_amount_y),
            "Failed at bin with index: {i}"
        );
    }

    Ok(())
}

fn round_down_to_5_decimal_places(decimal: Decimal) -> Decimal {
    decimal
        .checked_round(5, RoundingMode::ToNegativeInfinity)
        .unwrap()
}
