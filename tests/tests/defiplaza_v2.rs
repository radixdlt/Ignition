#![allow(clippy::arithmetic_side_effects)]

use tests::prelude::*;

#[test]
fn can_open_a_simple_position_against_a_defiplaza_pool(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        defiplaza_v2,
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
        defiplaza_v2.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    );

    // Assert
    let _ = rtn.expect("Should succeed!");

    Ok(())
}

#[test]
fn can_open_a_liquidity_position_in_defiplaza_that_fits_into_fee_limits() {
    // Arrange
    let ScryptoUnitEnv {
        environment: mut test_runner,
        resources,
        protocol,
        defiplaza_v2,
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
                            defiplaza_v2.pools.bitcoin,
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
                        defiplaza_v2.pools.bitcoin,
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
fn can_close_a_liquidity_position_in_defiplaza_that_fits_into_fee_limits() {
    // Arrange
    let ScryptoUnitEnv {
        environment: mut test_runner,
        resources,
        protocol,
        defiplaza_v2,
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
                                defiplaza_v2.pools.bitcoin,
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
                defiplaza_v2.liquidity_receipt,
                dec!(1),
            )
            .take_all_from_worktop(defiplaza_v2.liquidity_receipt, "receipt")
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
fn fees_are_zero_when_no_swaps_take_place() -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut defiplaza_v2,
        resources,
        ..
    } = ScryptoTestEnv::new()?;

    let [bitcoin_bucket, xrd_bucket] = [resources.bitcoin, XRD]
        .map(ResourceManager)
        .map(|mut resource_manager| {
            resource_manager.mint_fungible(dec!(100), env).unwrap()
        });

    let OpenLiquidityPositionOutput {
        pool_units,
        adapter_specific_information,
        ..
    } = defiplaza_v2.adapter.open_liquidity_position(
        defiplaza_v2.pools.bitcoin.try_into().unwrap(),
        (bitcoin_bucket, xrd_bucket),
        env,
    )?;

    // Act
    let CloseLiquidityPositionOutput { fees, .. } =
        defiplaza_v2.adapter.close_liquidity_position(
            defiplaza_v2.pools.bitcoin.try_into().unwrap(),
            pool_units.into_values().collect(),
            adapter_specific_information,
            env,
        )?;

    // Assert
    assert!(fees.values().all(|value| *value == Decimal::ZERO));

    Ok(())
}

#[test]
fn a_swap_with_xrd_input_produces_xrd_fees() -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut defiplaza_v2,
        resources,
        ..
    } = ScryptoTestEnv::new()?;

    let [bitcoin_bucket, xrd_bucket] = [resources.bitcoin, XRD]
        .map(ResourceManager)
        .map(|mut resource_manager| {
            resource_manager.mint_fungible(dec!(100), env).unwrap()
        });

    let OpenLiquidityPositionOutput {
        pool_units,
        adapter_specific_information,
        ..
    } = defiplaza_v2.adapter.open_liquidity_position(
        defiplaza_v2.pools.bitcoin.try_into().unwrap(),
        (bitcoin_bucket, xrd_bucket),
        env,
    )?;

    let _ = ResourceManager(XRD)
        .mint_fungible(dec!(100_000), env)
        .and_then(|bucket| defiplaza_v2.pools.bitcoin.swap(bucket, env))?;

    // Act
    let CloseLiquidityPositionOutput { fees, .. } =
        defiplaza_v2.adapter.close_liquidity_position(
            defiplaza_v2.pools.bitcoin.try_into().unwrap(),
            pool_units.into_values().collect(),
            adapter_specific_information,
            env,
        )?;

    // Assert
    assert_eq!(*fees.get(&resources.bitcoin).unwrap(), dec!(0));
    assert_ne!(*fees.get(&XRD).unwrap(), dec!(0));

    Ok(())
}

#[test]
fn a_swap_with_btc_input_produces_btc_fees() -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut defiplaza_v2,
        resources,
        ..
    } = ScryptoTestEnv::new()?;

    let [bitcoin_bucket, xrd_bucket] = [resources.bitcoin, XRD]
        .map(ResourceManager)
        .map(|mut resource_manager| {
            resource_manager.mint_fungible(dec!(100), env).unwrap()
        });

    let OpenLiquidityPositionOutput {
        pool_units,
        adapter_specific_information,
        ..
    } = defiplaza_v2.adapter.open_liquidity_position(
        defiplaza_v2.pools.bitcoin.try_into().unwrap(),
        (bitcoin_bucket, xrd_bucket),
        env,
    )?;

    let _ = ResourceManager(resources.bitcoin)
        .mint_fungible(dec!(100_000), env)
        .and_then(|bucket| defiplaza_v2.pools.bitcoin.swap(bucket, env))?;

    // Act
    let CloseLiquidityPositionOutput { fees, .. } =
        defiplaza_v2.adapter.close_liquidity_position(
            defiplaza_v2.pools.bitcoin.try_into().unwrap(),
            pool_units.into_values().collect(),
            adapter_specific_information,
            env,
        )?;

    // Assert
    assert_ne!(*fees.get(&resources.bitcoin).unwrap(), dec!(0));
    assert_eq!(*fees.get(&XRD).unwrap(), dec!(0));

    Ok(())
}

#[test]
fn contributions_to_defiplaza_through_adapter_dont_fail_due_to_bucket_ordering(
) -> Result<(), RuntimeError> {
    // Arrange
    let mut results = Vec::<bool>::new();
    for order in [true, false] {
        // Arrange
        let Environment {
            environment: ref mut env,
            resources,
            mut defiplaza_v2,
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
        let result = defiplaza_v2.adapter.open_liquidity_position(
            defiplaza_v2.pools.bitcoin.try_into().unwrap(),
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
        mut defiplaza_v2,
        resources,
        ..
    } = ScryptoTestEnv::new()?;

    let pool_reported_price = defiplaza_v2
        .adapter
        .price(defiplaza_v2.pools.bitcoin.try_into().unwrap(), env)?;
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
        defiplaza_v2.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    )?;

    let OpenLiquidityPositionOutput {
        pool_units,
        adapter_specific_information,
        ..
    } = defiplaza_v2.adapter.open_liquidity_position(
        defiplaza_v2.pools.bitcoin.try_into().unwrap(),
        (
            ResourceManager(resources.bitcoin)
                .mint_fungible(dec!(100_000), env)?,
            ResourceManager(XRD).mint_fungible(dec!(100_000), env)?,
        ),
        env,
    )?;

    match price_of_user_asset {
        // User asset price goes down - i.e., we inject it into the pool.
        Movement::Down => {
            let bitcoin_bucket = ResourceManager(resources.bitcoin)
                .mint_fungible(dec!(10_000_000), env)?;
            let _ = defiplaza_v2.pools.bitcoin.swap(bitcoin_bucket, env)?;
        }
        // The user asset price stays the same. We do not do anything.
        Movement::Same => {}
        // User asset price goes up - i.e., we reduce it in the pool.
        Movement::Up => {
            let xrd_bucket =
                ResourceManager(XRD).mint_fungible(dec!(10_000_000), env)?;
            let _ = defiplaza_v2.pools.bitcoin.swap(xrd_bucket, env)?;
        }
    }

    match protocol_coefficient {
        // Somebody claimed some portion of the pool
        Movement::Down => {
            defiplaza_v2.adapter.close_liquidity_position(
                defiplaza_v2.pools.bitcoin.try_into().unwrap(),
                pool_units.into_values().collect(),
                adapter_specific_information,
                env,
            )?;
        }
        // Nothing
        Movement::Same => {}
        // Somebody contributed to the pool some amount
        Movement::Up => {
            let _ = defiplaza_v2
                .adapter
                .open_liquidity_position(
                    defiplaza_v2.pools.bitcoin.try_into().unwrap(),
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
    let pool_reported_price = defiplaza_v2
        .adapter
        .price(defiplaza_v2.pools.bitcoin.try_into().unwrap(), env)?;
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

#[test]
fn user_resources_are_contributed_in_full_when_oracle_price_is_same_as_pool_price(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        mut defiplaza_v2,
        resources,
        ..
    } = ScryptoTestEnv::new()?;

    let pool = ComponentAddress::try_from(defiplaza_v2.pools.bitcoin).unwrap();
    let user_resource = resources.bitcoin;

    let pool_price = defiplaza_v2.adapter.price(pool, env)?;
    protocol.oracle.set_price(
        pool_price.base,
        pool_price.quote,
        pool_price.price,
        env,
    )?;

    let user_resource_bucket =
        ResourceManager(user_resource).mint_fungible(dec!(100), env)?;

    // Act
    let (_, _, change) = protocol.ignition.open_liquidity_position(
        FungibleBucket(user_resource_bucket),
        pool,
        LockupPeriod::from_months(6).unwrap(),
        env,
    )?;

    // Assert
    assert_eq!(change.len(), 0);

    Ok(())
}

#[test]
fn user_resources_are_contributed_in_full_when_oracle_price_is_higher_than_pool_price(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        mut defiplaza_v2,
        resources,
        ..
    } = ScryptoTestEnv::new_with_configuration(Configuration {
        maximum_allowed_relative_price_difference: dec!(0.05),
        ..Default::default()
    })?;

    let pool = ComponentAddress::try_from(defiplaza_v2.pools.bitcoin).unwrap();
    let user_resource = resources.bitcoin;

    let pool_price = defiplaza_v2.adapter.price(pool, env)?;
    protocol.oracle.set_price(
        pool_price.base,
        pool_price.quote,
        pool_price.price * dec!(1.05),
        env,
    )?;

    let user_resource_bucket =
        ResourceManager(user_resource).mint_fungible(dec!(100), env)?;

    // Act
    let (_, _, change) = protocol.ignition.open_liquidity_position(
        FungibleBucket(user_resource_bucket),
        pool,
        LockupPeriod::from_months(6).unwrap(),
        env,
    )?;

    // Assert
    assert_eq!(change.len(), 0);

    Ok(())
}

#[test]
fn user_resources_are_contributed_in_full_when_oracle_price_is_lower_than_pool_price(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        mut defiplaza_v2,
        resources,
        ..
    } = ScryptoTestEnv::new_with_configuration(Configuration {
        maximum_allowed_relative_price_difference: dec!(0.05),
        ..Default::default()
    })?;

    let pool = ComponentAddress::try_from(defiplaza_v2.pools.bitcoin).unwrap();
    let user_resource = resources.bitcoin;

    let pool_price = defiplaza_v2.adapter.price(pool, env)?;
    protocol.oracle.set_price(
        pool_price.base,
        pool_price.quote,
        pool_price.price * dec!(0.96),
        env,
    )?;

    let user_resource_bucket =
        ResourceManager(user_resource).mint_fungible(dec!(100), env)?;

    // Act
    let (_, _, change) = protocol.ignition.open_liquidity_position(
        FungibleBucket(user_resource_bucket),
        pool,
        LockupPeriod::from_months(6).unwrap(),
        env,
    )?;

    // Assert
    assert_eq!(change.len(), 0);

    Ok(())
}

#[test]
fn pool_reported_price_and_quote_reported_price_are_similar_with_base_resource_as_input(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut defiplaza_v2,
        ..
    } = ScryptoTestEnv::new_with_configuration(Configuration {
        maximum_allowed_relative_price_difference: dec!(0.05),
        ..Default::default()
    })?;

    let pool = defiplaza_v2.pools.bitcoin;
    let (base_resource, quote_resource) = pool.get_tokens(env)?;
    let input_amount = dec!(100);
    let input_resource = base_resource;
    let output_resource = if input_resource == base_resource {
        quote_resource
    } else {
        base_resource
    };

    let pool_reported_price = defiplaza_v2
        .adapter
        .price(ComponentAddress::try_from(pool).unwrap(), env)?;

    // Act
    let (output_amount, remainder, ..) =
        pool.quote(input_amount, input_resource == quote_resource, env)?;

    // Assert
    let input_amount = input_amount - remainder;
    let quote_reported_price = Price {
        price: output_amount / input_amount,
        base: input_resource,
        quote: output_resource,
    };
    let relative_difference = pool_reported_price
        .relative_difference(&quote_reported_price)
        .unwrap();

    assert!(relative_difference <= dec!(0.0001));

    Ok(())
}

#[test]
fn pool_reported_price_and_quote_reported_price_are_similar_with_quote_resource_as_input(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut defiplaza_v2,
        ..
    } = ScryptoTestEnv::new_with_configuration(Configuration {
        maximum_allowed_relative_price_difference: dec!(0.05),
        ..Default::default()
    })?;

    let pool = defiplaza_v2.pools.bitcoin;
    let (base_resource, quote_resource) = pool.get_tokens(env)?;
    let input_amount = dec!(100);
    let input_resource = quote_resource;
    let output_resource = if input_resource == base_resource {
        quote_resource
    } else {
        base_resource
    };

    let pool_reported_price = defiplaza_v2
        .adapter
        .price(ComponentAddress::try_from(pool).unwrap(), env)?;

    // Act
    let (output_amount, remainder, ..) =
        pool.quote(input_amount, input_resource == quote_resource, env)?;

    // Assert
    let input_amount = input_amount - remainder;
    let quote_reported_price = Price {
        price: output_amount / input_amount,
        base: input_resource,
        quote: output_resource,
    };
    let relative_difference = pool_reported_price
        .relative_difference(&quote_reported_price)
        .unwrap();

    assert!(relative_difference <= dec!(0.0001));

    Ok(())
}
