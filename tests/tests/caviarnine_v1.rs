#![allow(clippy::arithmetic_side_effects)]

use tests::prelude::*;

#[test]
fn can_open_a_simple_position_against_a_caviarnine_pool(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        caviarnine_v1,
        resources,
        ..
    } = ScryptoTestEnv::new()?;
    protocol
        .ignition
        .set_maximum_allowed_price_difference_percentage(dec!(0.50), env)?;

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
    } = ScryptoTestEnv::new()?;
    protocol
        .ignition
        .set_maximum_allowed_price_difference_percentage(dec!(0.50), env)?;

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
    assert_eq!(
        data.adapter_specific_information.bin_contributions.len(),
        61
    );

    Ok(())
}

#[test]
fn can_open_a_liquidity_position_in_caviarnine_that_fits_into_fee_limits() {
    // Arrange
    let ScryptoUnitEnv {
        environment: mut test_runner,
        resources,
        protocol,
        caviarnine_v1,
        ..
    } = ScryptoUnitEnv::new_with_configuration(Configuration {
        maximum_allowed_relative_price_difference: dec!(0.03),
        ..Default::default()
    });
    let (_, private_key, account_address, _) = protocol.protocol_owner_badge;

    test_runner
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .mint_fungible(resources.bitcoin, dec!(100_000_000_000_000))
                .try_deposit_entire_worktop_or_abort(account_address, None)
                .build(),
            vec![],
        )
        .expect_commit_success();

    test_runner
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
    let receipt = test_runner.construct_and_execute_notarized_transaction(
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
        environment: mut test_runner,
        resources,
        protocol,
        caviarnine_v1,
        ..
    } = ScryptoUnitEnv::new_with_configuration(Configuration {
        maximum_allowed_relative_price_difference: dec!(0.03),
        ..Default::default()
    });
    let (_, private_key, account_address, _) = protocol.protocol_owner_badge;

    test_runner
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .mint_fungible(resources.bitcoin, dec!(100_000_000_000_000))
                .try_deposit_entire_worktop_or_abort(account_address, None)
                .build(),
            vec![],
        )
        .expect_commit_success();

    for _ in 0..2 {
        test_runner
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

    let current_time = test_runner.get_current_time(TimePrecisionV2::Minute);
    let maturity_instant = current_time
        .add_seconds(*LockupPeriod::from_months(6).unwrap().seconds() as i64)
        .unwrap();
    {
        let db = test_runner.substate_db_mut();
        let mut writer = SystemDatabaseWriter::new(db);

        writer
            .write_typed_object_field(
                CONSENSUS_MANAGER.as_node_id(),
                ModuleId::Main,
                ConsensusManagerField::ProposerMilliTimestamp.field_index(),
                ConsensusManagerProposerMilliTimestampFieldPayload::from_content_source(
                    ProposerMilliTimestampSubstate { epoch_milli: maturity_instant.seconds_since_unix_epoch * 1000  }
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
                        epoch_minute: i32::try_from(maturity_instant.seconds_since_unix_epoch / 60).unwrap(),
                    }
                ),
            )
            .unwrap();
    }

    test_runner
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
    let receipt = test_runner.construct_and_execute_notarized_transaction(
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
        caviarnine_v1,
        ..
    } = ScryptoTestEnv::new()?;
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
    assert_eq!(
        adapter_information.bin_contributions.len(),
        (PREFERRED_TOTAL_NUMBER_OF_HIGHER_AND_LOWER_BINS + 1) as usize
    );

    Ok(())
}

#[test]
fn contribution_amount_reported_in_receipt_nft_matches_caviarnine_state(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        caviarnine_v1,
        resources,
        ..
    } = ScryptoTestEnv::new()?;
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
        .withdraw_pool_units(ignition_receipt_global_id, env)?;

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
        adapter_reported_contributions.len(),
        caviarnine_reported_contributions.len()
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
    } = ScryptoTestEnv::new()?;

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
        .pool_units;

    match price_of_user_asset {
        // User asset price goes down - i.e., we inject it into the pool.
        Movement::Down => {
            let bitcoin_bucket = ResourceManager(resources.bitcoin)
                .mint_fungible(dec!(450_000_000), env)?;
            let _ = caviarnine_v1.pools.bitcoin.swap(bitcoin_bucket, env)?;
        }
        // The user asset price stays the same. We do not do anything.
        Movement::Same => {}
        // User asset price goes up - i.e., we reduce it in the pool.
        Movement::Up => {
            let xrd_bucket =
                ResourceManager(XRD).mint_fungible(dec!(450_000_000), env)?;
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

    let buckets = IndexedBuckets::from_buckets(
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
    difference <= dec!(0.000001)
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
    } = ScryptoTestEnv::new()?;
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
fn positions_can_be_opened_at_current_price_and_closed_at_a_100x_price_decrease_on_100_bps_pools(
) {
    test_effect_of_price_action_on_fees(-100, 100)
}

fn test_effect_of_price_action_on_fees(multiplier: i32, bin_span: u32) {
    let ScryptoUnitEnv {
        environment: mut test_runner,
        resources,
        protocol,
        caviarnine_v1,
        ..
    } = ScryptoUnitEnv::new_with_configuration(Configuration {
        maximum_allowed_relative_price_difference: dec!(0.03),
        ..Default::default()
    });
    let (_, private_key, account_address, _) = protocol.protocol_owner_badge;

    let resource_x = XRD;
    let resource_y = resources.bitcoin;

    let pool_address = test_runner
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .caviarnine_v1_pool_new(
                    caviarnine_v1.package,
                    rule!(allow_all),
                    rule!(allow_all),
                    resource_x,
                    resource_y,
                    bin_span,
                    None,
                )
                .build(),
            vec![],
        )
        .expect_commit_success()
        .new_component_addresses()
        .first()
        .copied()
        .unwrap();

    // We're allowed to contribute to 200 bins. So, we will contribute to all of
    // them. This ensures that the maximum amount of price range is covered by
    // our liquidity.
    {
        let positions = vec![(27000u32, dec!(100_000_000), dec!(100_000_000))]
            .into_iter()
            .chain((1..=99).flat_map(|i| {
                vec![
                    (
                        27000 - i * bin_span,
                        dec!(0),
                        dec!(100_000_000) - Decimal::from(i),
                    ),
                    (
                        27000 + i * bin_span,
                        dec!(100_000_000) + Decimal::from(i),
                        dec!(0),
                    ),
                ]
            }))
            .collect::<Vec<_>>();
        let x_amount_required = positions
            .iter()
            .map(|v| v.1)
            .reduce(|acc, item| acc + item)
            .unwrap_or_default();
        let y_amount_required = positions
            .iter()
            .map(|v| v.2)
            .reduce(|acc, item| acc + item)
            .unwrap_or_default();

        test_runner
            .execute_manifest_with_enabled_modules(
                ManifestBuilder::new()
                    .mint_fungible(resource_x, x_amount_required)
                    .mint_fungible(resource_y, y_amount_required)
                    .take_all_from_worktop(resource_x, "resources_x")
                    .take_all_from_worktop(resource_y, "resources_y")
                    .with_name_lookup(|builder, namer| {
                        let resources_x = namer.bucket("resources_x");
                        let resources_y = namer.bucket("resources_y");

                        builder.caviarnine_v1_pool_add_liquidity(
                            pool_address,
                            resources_x,
                            resources_y,
                            positions,
                        )
                    })
                    .deposit_batch(account_address)
                    .build(),
                EnabledModules::for_notarized_transaction()
                    & !EnabledModules::AUTH
                    & !EnabledModules::COSTING,
            )
            .expect_commit_success();
    }

    // Adding this pool to Ignition.
    test_runner
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
    test_runner
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
    let price = test_runner
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .caviarnine_v1_pool_get_price(pool_address)
                .build(),
            vec![],
        )
        .expect_commit_success()
        .output::<Option<Decimal>>(1)
        .unwrap();
    test_runner
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
    test_runner
        .execute_manifest_without_auth(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .mint_fungible(resource_y, dec!(100_000))
                .deposit_batch(account_address)
                .build(),
        )
        .expect_commit_success();

    let receipt = test_runner.construct_and_execute_notarized_transaction(
        ManifestBuilder::new()
            .lock_fee(account_address, dec!(10))
            .withdraw_from_account(account_address, resource_y, dec!(1000))
            .take_all_from_worktop(resource_y, "bucket")
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
        "Open - Multiplier = {}x, Bin Span = {}, Cost = {} XRD, Execution Cost = {} XRD",
        multiplier,
        bin_span,
        receipt.fee_summary.total_cost(),
        receipt.fee_summary.total_execution_cost_in_xrd
    );

    // Set the current time to be 6 months from now.
    {
        let current_time =
            test_runner.get_current_time(TimePrecisionV2::Minute);
        let maturity_instant =
            current_time
                .add_seconds(
                    *LockupPeriod::from_months(6).unwrap().seconds() as i64
                )
                .unwrap();
        let db = test_runner.substate_db_mut();
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
                        epoch_minute: i32::try_from(
                            maturity_instant.seconds_since_unix_epoch / 60,
                        )
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
        let amount_in_each_swap = dec!(100_000_000);

        let mut new_price = price;
        while new_price < target_price {
            let reported_price = test_runner
                .execute_manifest_without_auth(
                    ManifestBuilder::new()
                        .lock_fee_from_faucet()
                        .mint_fungible(input_resource, amount_in_each_swap)
                        .take_all_from_worktop(input_resource, "bucket")
                        .with_bucket("bucket", |builder, bucket| {
                            builder
                                .caviarnine_v1_pool_swap(pool_address, bucket)
                        })
                        .deposit_batch(account_address)
                        .caviarnine_v1_pool_get_price(pool_address)
                        .build(),
                )
                .expect_commit_success()
                .output::<Option<Decimal>>(5)
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
        let amount_in_each_swap = dec!(1_000_000_000);

        let mut new_price = price;
        while new_price > target_price {
            let reported_price = test_runner
                .execute_manifest_without_auth(
                    ManifestBuilder::new()
                        .lock_fee_from_faucet()
                        .mint_fungible(input_resource, amount_in_each_swap)
                        .take_all_from_worktop(input_resource, "bucket")
                        .with_bucket("bucket", |builder, bucket| {
                            builder
                                .caviarnine_v1_pool_swap(pool_address, bucket)
                        })
                        .deposit_batch(account_address)
                        .caviarnine_v1_pool_get_price(pool_address)
                        .build(),
                )
                .expect_commit_success()
                .output::<Option<Decimal>>(5)
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
    test_runner
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
    let receipt = test_runner.construct_and_execute_notarized_transaction(
        ManifestBuilder::new()
            .lock_fee(account_address, dec!(10))
            .withdraw_from_account(
                account_address,
                caviarnine_v1.liquidity_receipt,
                dec!(1),
            )
            .take_all_from_worktop(caviarnine_v1.liquidity_receipt, "bucket")
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
        "Close - Multiplier = {}x, Bin Span = {}, Cost = {} XRD, Execution Cost = {} XRD",
        multiplier,
        bin_span,
        receipt.fee_summary.total_cost(),
        receipt.fee_summary.total_execution_cost_in_xrd
    );
}

// Tests that the k in each of the bins we've contributed to is equal. We cant
// test for strict equality between them because of the loss of precision that
// happens between calculations. Therefore, we test that the standard deviation
// of the various `L` values is lower than 0.0001.
#[test]
fn k_is_equal_in_all_of_the_bins_contributed_to() -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        mut caviarnine_v1,
        resources,
        ..
    } = ScryptoTestEnv::new()?;
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

    let bin_contributions_and_liquidity = data
        .adapter_specific_information
        .bin_contributions
        .into_iter()
        .map(|(tick, amount)| {
            let l = calculate_liquidity(
                amount,
                tick_to_spot(tick).unwrap(),
                tick_to_spot(tick + bin_span).unwrap(),
            )
            .unwrap();

            (tick, (amount, l))
        })
        .collect::<IndexMap<_, _>>();

    let average_liquidity = bin_contributions_and_liquidity
        .iter()
        .map(|(_, (_, liquidity))| *liquidity)
        .reduce(|acc, item| acc + item)
        .and_then(|value| {
            value.checked_div(bin_contributions_and_liquidity.len() as u32)
        })
        .unwrap();

    let standard_deviation = bin_contributions_and_liquidity
        .iter()
        .map(|(_, (_, liquidity))| *liquidity)
        .map(|liquidity| {
            (liquidity - average_liquidity).checked_powi(2).unwrap()
        })
        .reduce(|acc, item| acc + item)
        .and_then(|value| {
            value.checked_div(bin_contributions_and_liquidity.len() as u32)
        })
        .and_then(|value| value.checked_sqrt())
        .unwrap();

    assert!(standard_deviation <= dec!(0.0001));
    Ok(())
}
