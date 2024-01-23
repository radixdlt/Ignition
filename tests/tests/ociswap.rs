use tests::prelude::*;

#[test]
pub fn can_open_a_simple_position_against_an_ociswap_pool(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        ociswap,
        resources,
        ..
    } = ScryptoTestEnv::new()?;

    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;

    // Act
    let rtn = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        ociswap.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6),
        env,
    );

    // Assert
    let _ = rtn.expect("Should succeed!");

    Ok(())
}

#[test]
pub fn price_reported_by_pool_is_equal_to_price_reported_by_adapter(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut ociswap,
        resources,
        ..
    } = ScryptoTestEnv::new()?;

    let bitcoin_bucket = ResourceManager(resources.bitcoin)
        .mint_fungible(dec!(10_000_000), env)?;
    let _ = ociswap.pools.bitcoin.swap(bitcoin_bucket, env)?;

    // Act
    let pool_reported_price = ociswap
        .pools
        .bitcoin
        .price_sqrt(env)?
        .unwrap()
        .checked_powi(2)
        .unwrap()
        .checked_truncate(RoundingMode::ToZero)
        .unwrap();
    let adapter_reported_price = ociswap
        .adapter
        .price(ociswap.pools.bitcoin.try_into().unwrap(), env)?
        .price;

    // Assert
    assert_eq!(pool_reported_price, adapter_reported_price);

    Ok(())
}

#[test]
fn ociswap_liquidity_receipts_are_ociswap_branded() -> Result<(), RuntimeError>
{
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        resources,
        ociswap,
        ..
    } = ScryptoTestEnv::new()?;

    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;

    let (liquidity_receipt, _, _) = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        ociswap.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6),
        env,
    )?;

    // Act
    let liquidity_receipt_data = ResourceManager(ociswap.liquidity_receipt)
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
    assert_eq!(liquidity_receipt_data.name, "Ociswap Liquidity Receipt");
    assert_eq!(
        liquidity_receipt_data.description,
        "A receipt of liquidity provided to a Ociswap pool through the Ignition protocol"
    );
    assert_eq!(
        liquidity_receipt_data.description,
        "A receipt of liquidity provided to a Ociswap pool through the Ignition protocol"
    );
    assert_eq!(
        liquidity_receipt_data.key_image_url.0,
        "https://ociswap.com/icons/oci.png"
    );
    assert_eq!(
        liquidity_receipt_data.redemption_url.0,
        "https://ociswap.com/"
    );

    Ok(())
}

#[test]
fn can_open_a_liquidity_position_in_ociswap_that_fits_into_fee_limits() {
    // Arrange
    let ScryptoUnitEnv {
        environment: mut test_runner,
        resources,
        protocol,
        ociswap,
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
                        ociswap.pools.bitcoin,
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
        environment: mut test_runner,
        resources,
        protocol,
        ociswap,
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
                            ociswap.pools.bitcoin,
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
                ociswap.liquidity_receipt,
                dec!(1),
            )
            .take_all_from_worktop(ociswap.liquidity_receipt, "receipt")
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
fn contributions_directly_to_ociswap_dont_fail_due_to_bucket_order(
) -> Result<(), RuntimeError> {
    // Arrange
    let mut results = Vec::<bool>::new();
    for order in [true, false] {
        // Arrange
        let Environment {
            environment: ref mut env,
            resources,
            mut ociswap,
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
        let result = ociswap
            .pools
            .bitcoin
            .add_liquidity(buckets.0, buckets.1, env);
        results.push(result.is_ok());
    }

    // Assert
    assert_eq!(results.len(), 2);
    assert_eq!(results.iter().filter(|item| **item).count(), 2);

    Ok(())
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
            mut ociswap,
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
        let result = ociswap.adapter.open_liquidity_position(
            ociswap.pools.bitcoin.try_into().unwrap(),
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
