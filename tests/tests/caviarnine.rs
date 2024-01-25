use tests::prelude::*;

#[test]
pub fn can_open_a_simple_position_against_an_caviarnine_pool(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        caviarnine,
        resources,
        ..
    } = ScryptoTestEnv::new()?;
    protocol
        .ignition
        .set_maximum_allowed_price_difference_percentage(dec!(0.03), env)?;

    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;

    // Act
    let rtn = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        caviarnine.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6),
        env,
    );

    // Assert
    let _ = rtn.expect("Should succeed!");

    Ok(())
}

#[test]
fn caviarnine_liquidity_receipts_are_caviarnine_branded(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        resources,
        caviarnine,
        ..
    } = ScryptoTestEnv::new()?;
    protocol
        .ignition
        .set_maximum_allowed_price_difference_percentage(dec!(0.03), env)?;

    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;

    let (liquidity_receipt, _, _) = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        caviarnine.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6),
        env,
    )?;

    // Act
    let liquidity_receipt_data = ResourceManager(caviarnine.liquidity_receipt)
        .get_non_fungible_data::<_, _, LiquidityReceipt>(
            liquidity_receipt
                .0
                .non_fungible_local_ids(env)?
                .first()
                .unwrap()
                .clone(),
            env,
        )?;

    // Assert
    assert_eq!(liquidity_receipt_data.name, "Caviarnine Liquidity Receipt");
    assert_eq!(
        liquidity_receipt_data.description,
        "A receipt of liquidity provided to a Caviarnine pool through the Ignition protocol"
    );
    assert_eq!(
        liquidity_receipt_data.description,
        "A receipt of liquidity provided to a Caviarnine pool through the Ignition protocol"
    );
    assert_eq!(
        liquidity_receipt_data.key_image_url.0,
        "https://assets.caviarnine.com/tokens/resource_rdx1t5pyvlaas0ljxy0wytm5gvyamyv896m69njqdmm2stukr3xexc2up9.png"
    );
    assert_eq!(
        liquidity_receipt_data.redemption_url.0,
        "https://www.caviarnine.com/"
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
        caviarnine,
        ..
    } = ScryptoUnitEnv::new_with_configuration(Configuration {
        maximum_allowed_relative_price_difference: dec!(0.03),
        ..Default::default()
    });
    let (public_key, account_address, _) = protocol.protocol_owner_badge;

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

    // Act
    let receipt = test_runner.execute_manifest(
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
                        caviarnine.pools.bitcoin,
                        LockupPeriod::from_months(6),
                    ),
                )
            })
            .try_deposit_entire_worktop_or_abort(account_address, None)
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
    let TransactionFeeSummary {
        total_execution_cost_in_xrd,
        ..
    } = receipt.fee_summary;

    assert!(total_execution_cost_in_xrd <= dec!(4.8))
}

#[test]
fn can_close_a_liquidity_position_in_caviarnine_that_fits_into_fee_limits() {
    // Arrange
    let ScryptoUnitEnv {
        environment: mut test_runner,
        resources,
        protocol,
        caviarnine,
        ..
    } = ScryptoUnitEnv::new_with_configuration(Configuration {
        maximum_allowed_relative_price_difference: dec!(0.03),
        ..Default::default()
    });
    let (public_key, account_address, _) = protocol.protocol_owner_badge;

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
                            caviarnine.pools.bitcoin,
                            LockupPeriod::from_months(6),
                        ),
                    )
                })
                .try_deposit_entire_worktop_or_abort(account_address, None)
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        )
        .expect_commit_success();

    let current_time = test_runner.get_current_time(TimePrecisionV2::Minute);
    let maturity_instant = current_time
        .add_seconds(*LockupPeriod::from_months(6).seconds() as i64)
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
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(
                account_address,
                caviarnine.liquidity_receipt,
                dec!(1),
            )
            .take_all_from_worktop(caviarnine.liquidity_receipt, "receipt")
            .with_bucket("receipt", |builder, bucket| {
                builder.call_method(
                    protocol.ignition,
                    "close_liquidity_position",
                    (bucket,),
                )
            })
            .try_deposit_entire_worktop_or_abort(account_address, None)
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
    let TransactionFeeSummary {
        total_execution_cost_in_xrd,
        ..
    } = receipt.fee_summary;

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
            mut caviarnine,
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
        let result = caviarnine.pools.bitcoin.add_liquidity(
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
            mut caviarnine,
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
        let result = caviarnine.adapter.open_liquidity_position(
            caviarnine.pools.bitcoin.try_into().unwrap(),
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
        caviarnine,
        ..
    } = ScryptoTestEnv::new()?;
    protocol
        .ignition
        .set_maximum_allowed_price_difference_percentage(dec!(0.03), env)?;

    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;

    let (liquidity_receipt, _, _) = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        caviarnine.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6),
        env,
    )?;

    // Act
    let liquidity_receipt_data = ResourceManager(caviarnine.liquidity_receipt)
        .get_non_fungible_data::<_, _, LiquidityReceipt>(
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
        .as_typed::<CaviarnineAdapterSpecificInformation>()
        .unwrap();
    assert_eq!(
        adapter_information
            .bin_information_when_position_opened
            .len(),
        (PREFERRED_TOTAL_NUMBER_OF_HIGHER_AND_LOWER_BINS + 1) as usize
    );

    Ok(())
}

#[test]
pub fn contribution_amount_reported_in_receipt_nft_matches_caviarnine_state(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        caviarnine,
        resources,
        ..
    } = ScryptoTestEnv::new()?;
    protocol
        .ignition
        .set_maximum_allowed_price_difference_percentage(dec!(0.03), env)?;

    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;

    // Act
    let (ignition_receipt, ..) = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        caviarnine.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6),
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
        NonFungibleGlobalId::new(caviarnine.liquidity_receipt, local_id)
    };
    let ignition_receipt_data = ResourceManager(caviarnine.liquidity_receipt)
        .get_non_fungible_data::<_, _, LiquidityReceipt>(
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
        caviarnine.pools.bitcoin.get_redemption_bin_values(
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
        .as_typed::<CaviarnineAdapterSpecificInformation>()
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
pub fn reserves_amount_reported_in_receipt_nft_matches_caviarnine_state(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        caviarnine,
        resources,
        ..
    } = ScryptoTestEnv::new()?;
    protocol
        .ignition
        .set_maximum_allowed_price_difference_percentage(dec!(0.03), env)?;

    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;

    // Act
    let (ignition_receipt, ..) = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        caviarnine.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6),
        env,
    )?;

    // Assert
    let ignition_receipt_data = ResourceManager(caviarnine.liquidity_receipt)
        .get_non_fungible_data::<_, _, LiquidityReceipt>(
        ignition_receipt
            .0
            .non_fungible_local_ids(env)?
            .first()
            .unwrap()
            .clone(),
        env,
    )?;

    let caviarnine_reported_reserves = {
        let x_reserves = caviarnine
            .pools
            .bitcoin
            .get_bins_above(None, None, None, env)?;
        let y_reserves = caviarnine
            .pools
            .bitcoin
            .get_bins_below(None, None, None, env)?;

        let mut reserves = IndexMap::<u32, (Decimal, Decimal)>::new();
        for (bin, value) in x_reserves {
            reserves.entry(bin).or_default().0 += value;
        }
        for (bin, value) in y_reserves {
            reserves.entry(bin).or_default().1 += value;
        }
        reserves.sort_by(|k1, _, k2, _| k1.cmp(k2));
        reserves
            .into_iter()
            .map(|item| (item.0, item.1 .0, item.1 .1))
            .collect::<Vec<_>>()
    };

    let mut adapter_reported_reserves = ignition_receipt_data
        .adapter_specific_information
        .as_typed::<CaviarnineAdapterSpecificInformation>()
        .unwrap()
        .bin_information_when_position_opened
        .into_iter()
        .map(|(bin, information)| {
            (
                bin,
                information.reserves.resource_x,
                information.reserves.resource_y,
            )
        })
        .collect::<Vec<_>>();
    adapter_reported_reserves.sort_by(|a, b| a.0.cmp(&b.0));

    assert_eq!(adapter_reported_reserves, caviarnine_reported_reserves);

    Ok(())
}

fn approximately_equals(a: Decimal, b: Decimal) -> bool {
    let difference = match (a == Decimal::ZERO, b == Decimal::ZERO) {
        (true, true) => dec!(0),
        (true, false) => (b - a).checked_abs().unwrap() / b,
        (false, true) => (b - a).checked_abs().unwrap() / a,
        (false, false) => (b - a).checked_abs().unwrap() / b,
    };
    difference <= dec!(0.000001)
}
