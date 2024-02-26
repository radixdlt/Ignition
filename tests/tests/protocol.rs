use tests::prelude::*;
use Volatility::*;

#[test]
fn simple_testing_environment_can_be_created() {
    ScryptoTestEnv::new().expect("Must succeed!");
}

#[test]
fn cant_open_a_liquidity_position_when_opening_is_disabled(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        resources,
        mut protocol,
        ociswap_v1,
        ..
    } = ScryptoTestEnv::new()?;

    protocol.ignition.set_is_open_position_enabled(false, env)?;
    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;

    // Act
    let rtn = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        ociswap_v1.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    );

    // Assert
    assert_is_ignition_opening_liquidity_positions_is_closed_error(&rtn);

    Ok(())
}

#[test]
fn cant_open_a_liquidity_position_on_a_pool_that_has_no_adapter(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        resources,
        mut protocol,
        ..
    } = ScryptoTestEnv::new()?;

    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;

    // Act
    let rtn = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        FAUCET,
        LockupPeriod::from_months(6).unwrap(),
        env,
    );

    // Assert
    assert_is_ignition_no_adapter_found_for_pool_error(&rtn);

    Ok(())
}

#[test]
fn cant_open_liquidity_position_against_a_pool_outside_of_the_allow_list(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        resources,
        mut protocol,
        ociswap_v1,
        ..
    } = ScryptoTestEnv::new()?;

    let new_pool = OciswapV1PoolInterfaceScryptoTestStub::instantiate(
        resources.bitcoin,
        XRD,
        dec!(0),
        FAUCET,
        ociswap_v1.package,
        env,
    )?;
    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;

    // Act
    let rtn = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        new_pool.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    );

    // Assert
    assert_is_ignition_pool_is_not_in_allow_list_error(&rtn);

    Ok(())
}

#[test]
fn cant_open_a_liquidity_position_in_a_pool_after_it_has_been_removed_from_allowed_list(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        resources,
        mut protocol,
        ociswap_v1,
        ..
    } = ScryptoTestEnv::new()?;

    protocol.ignition.remove_allowed_pool(
        ociswap_v1.pools.bitcoin.try_into().unwrap(),
        env,
    )?;
    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;

    // Act
    let rtn = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        ociswap_v1.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    );

    // Assert
    assert_is_ignition_pool_is_not_in_allow_list_error(&rtn);

    Ok(())
}

#[test]
fn cant_open_a_liquidity_position_with_some_random_resource(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        ociswap_v1,
        ..
    } = ScryptoTestEnv::new()?;

    let random_resource = ResourceBuilder::new_fungible(OwnerRole::None)
        .mint_initial_supply(100, env)?;

    // Act
    let rtn = protocol.ignition.open_liquidity_position(
        FungibleBucket(random_resource),
        ociswap_v1.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    );

    // Assert
    assert_is_ignition_user_resources_volatility_unknown_error(&rtn);

    Ok(())
}

#[test]
fn cant_open_a_liquidity_position_by_providing_the_protocol_resource(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        ociswap_v1,
        ..
    } = ScryptoTestEnv::new()?;

    let protocol_resource =
        ResourceManager(XRD).mint_fungible(dec!(100), env)?;
    protocol.ignition.insert_user_resource_volatility(
        XRD,
        Volatility::NonVolatile,
        env,
    )?;

    // Act
    let rtn = protocol.ignition.open_liquidity_position(
        FungibleBucket(protocol_resource),
        ociswap_v1.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    );

    // Assert
    assert_is_ignition_user_must_not_provide_protocol_asset_error(&rtn);

    Ok(())
}

#[test]
pub fn can_open_a_liquidity_position_before_the_price_is_stale(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        ociswap_v1,
        resources,
        ..
    } = ScryptoTestEnv::new_with_configuration(Configuration {
        maximum_allowed_price_staleness_seconds: 5 * 60,
        ..Default::default()
    })?;

    // Act
    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;

    // Act
    let rtn = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        ociswap_v1.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    );

    // Assert
    let _ = rtn.expect("Should succeed!");

    Ok(())
}

#[test]
pub fn can_open_a_liquidity_position_right_before_price_goes_stale(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        ociswap_v1,
        resources,
        ..
    } = ScryptoTestEnv::new_with_configuration(Configuration {
        maximum_allowed_price_staleness_seconds: 5 * 60,
        ..Default::default()
    })?;

    // Act
    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;

    let current_time = env.get_current_time();
    env.set_current_time(current_time.add_minutes(5).unwrap());

    // Act
    let rtn = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        ociswap_v1.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    );

    // Assert
    let _ = rtn.expect("Should succeed!");

    Ok(())
}

#[test]
pub fn cant_open_a_liquidity_position_right_after_price_goes_stale(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        ociswap_v1,
        resources,
        ..
    } = ScryptoTestEnv::new_with_configuration(Configuration {
        maximum_allowed_price_staleness_seconds: 5 * 60,
        ..Default::default()
    })?;

    // Act
    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;

    let current_time = env.get_current_time();
    env.set_current_time(current_time.add_minutes(6).unwrap());

    // Act
    let rtn = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        ociswap_v1.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    );

    // Assert
    assert_is_ignition_oracle_reported_price_is_stale_error(&rtn);

    Ok(())
}

#[test]
pub fn can_open_liquidity_position_when_oracle_price_is_lower_than_pool_but_within_allowed_relative_difference(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        ociswap_v1,
        resources,
        ..
    } = ScryptoTestEnv::new_with_configuration(Configuration {
        maximum_allowed_relative_price_difference: dec!(0.01),
        ..Default::default()
    })?;

    // Act
    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;

    let current_time = env.get_current_time();
    env.set_current_time(current_time.add_minutes(6).unwrap());

    // Act
    let rtn = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        ociswap_v1.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    );

    // Assert
    assert_is_ignition_oracle_reported_price_is_stale_error(&rtn);

    Ok(())
}

#[test]
#[allow(unused_must_use)]
fn oracle_price_cutoffs_for_opening_liquidity_positions_are_implemented_correctly(
) {
    const SMALL_DECIMAL: Decimal = dec!(0.000000000000000010);

    test_open_position_oracle_price_cutoffs(dec!(1) / dec!(1.01), dec!(0.01))
        .expect("Should succeed!");
    test_open_position_oracle_price_cutoffs(dec!(1) / dec!(0.99), dec!(0.01))
        .expect("Should succeed!");
    test_open_position_oracle_price_cutoffs(
        dec!(1) / dec!(0.99) - SMALL_DECIMAL,
        dec!(0.01),
    )
    .expect("Should succeed!");
    test_open_position_oracle_price_cutoffs(
        dec!(1) / dec!(1.01) + SMALL_DECIMAL,
        dec!(0.01),
    )
    .expect("Should succeed!");

    assert_is_ignition_relative_price_difference_larger_than_allowed_error(
        &test_open_position_oracle_price_cutoffs(
            dec!(1) / dec!(0.99) + SMALL_DECIMAL,
            dec!(0.01),
        ),
    );
    assert_is_ignition_relative_price_difference_larger_than_allowed_error(
        &test_open_position_oracle_price_cutoffs(
            dec!(1) / dec!(1.01) - SMALL_DECIMAL,
            dec!(0.01),
        ),
    );
}

fn test_open_position_oracle_price_cutoffs(
    oracle_price: Decimal,
    allowed_price_difference: Decimal,
) -> Result<(NonFungibleBucket, FungibleBucket, Vec<Bucket>), RuntimeError> {
    let Environment {
        environment: ref mut env,
        mut protocol,
        ociswap_v1,
        resources,
        ..
    } = ScryptoTestEnv::new_with_configuration(Configuration {
        maximum_allowed_relative_price_difference: allowed_price_difference,
        ..Default::default()
    })?;

    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;

    protocol
        .oracle
        .set_price(resources.bitcoin, XRD, oracle_price, env)?;

    // Act
    protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        ociswap_v1.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    )
}

#[test]
#[allow(unused_must_use)]
fn oracle_price_cutoffs_for_closing_liquidity_positions_are_implemented_correctly(
) {
    const SMALL_DECIMAL: Decimal = dec!(0.000000000000000010);

    test_close_position_oracle_price_cutoffs(dec!(1) / dec!(1.01), dec!(0.01))
        .expect("Should succeed!");
    test_close_position_oracle_price_cutoffs(dec!(1) / dec!(0.99), dec!(0.01))
        .expect("Should succeed!");
    test_close_position_oracle_price_cutoffs(
        dec!(1) / dec!(0.99) - SMALL_DECIMAL,
        dec!(0.01),
    )
    .expect("Should succeed!");
    test_close_position_oracle_price_cutoffs(
        dec!(1) / dec!(1.01) + SMALL_DECIMAL,
        dec!(0.01),
    )
    .expect("Should succeed!");

    assert_is_ignition_relative_price_difference_larger_than_allowed_error(
        &test_close_position_oracle_price_cutoffs(
            dec!(1) / dec!(0.99) + SMALL_DECIMAL,
            dec!(0.01),
        ),
    );
    assert_is_ignition_relative_price_difference_larger_than_allowed_error(
        &test_close_position_oracle_price_cutoffs(
            dec!(1) / dec!(1.01) - SMALL_DECIMAL,
            dec!(0.01),
        ),
    );
}

fn test_close_position_oracle_price_cutoffs(
    oracle_price: Decimal,
    allowed_price_difference: Decimal,
) -> Result<Vec<Bucket>, RuntimeError> {
    let Environment {
        environment: ref mut env,
        mut protocol,
        ociswap_v1,
        resources,
        ..
    } = ScryptoTestEnv::new_with_configuration(Configuration {
        maximum_allowed_relative_price_difference: allowed_price_difference,
        ..Default::default()
    })?;

    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;

    // Act
    let (receipt, ..) = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        ociswap_v1.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    )?;

    let current_time = env.get_current_time();
    env.set_current_time(current_time.add_days(12 * 30).unwrap());

    protocol
        .oracle
        .set_price(resources.bitcoin, XRD, oracle_price, env)?;

    protocol.ignition.close_liquidity_position(receipt, env)
}

#[test]
fn cant_open_a_liquidity_position_with_an_invalid_lockup_period(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        ociswap_v1,
        resources,
        ..
    } = ScryptoTestEnv::new()?;

    // Act
    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;

    // Act
    let rtn = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        ociswap_v1.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_seconds(1),
        env,
    );

    // Assert
    assert_is_ignition_lockup_period_has_no_associated_rewards_rate_error(&rtn);

    Ok(())
}

#[test]
fn cant_set_the_adapter_of_a_blueprint_that_is_not_registered(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        ..
    } = ScryptoTestEnv::new()?;

    // Act
    let rtn = protocol.ignition.set_pool_adapter(
        BlueprintId {
            package_address: FAUCET_PACKAGE,
            blueprint_name: "Faucet".into(),
        },
        FAUCET,
        env,
    );

    // Assert
    assert_is_ignition_no_adapter_found_for_pool_error(&rtn);

    Ok(())
}

#[test]
fn cant_add_allowed_pool_of_a_blueprint_that_is_not_registered(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        ..
    } = ScryptoTestEnv::new()?;

    // Act
    let rtn = protocol.ignition.add_allowed_pool(FAUCET, env);

    // Assert
    assert_is_ignition_no_adapter_found_for_pool_error(&rtn);

    Ok(())
}

#[test]
fn cant_add_an_allowed_pool_where_neither_of_the_resources_is_the_protocol_resource_ociswap_v1(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        ociswap_v1,
        ..
    } = ScryptoTestEnv::new()?;

    let fungible1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .mint_initial_supply(100, env)?;
    let fungible2 = ResourceBuilder::new_fungible(OwnerRole::None)
        .mint_initial_supply(100, env)?;
    let pool = OciswapV1PoolInterfaceScryptoTestStub::instantiate(
        fungible1.resource_address(env)?,
        fungible2.resource_address(env)?,
        dec!(0),
        FAUCET,
        ociswap_v1.package,
        env,
    )?;

    // Act
    let rtn = protocol
        .ignition
        .add_allowed_pool(pool.try_into().unwrap(), env);

    // Assert
    assert_is_ignition_neither_pool_resource_is_protocol_resource_error(&rtn);

    Ok(())
}

#[test]
fn cant_add_an_allowed_pool_where_neither_of_the_resources_is_the_protocol_resource_ociswap_v2(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        ociswap_v2,
        ..
    } = ScryptoTestEnv::new()?;

    let fungible1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .mint_initial_supply(100, env)?;
    let fungible2 = ResourceBuilder::new_fungible(OwnerRole::None)
        .mint_initial_supply(100, env)?;
    let (pool, ..) = OciswapV2PoolInterfaceScryptoTestStub::instantiate(
        fungible2.resource_address(env)?,
        fungible1.resource_address(env)?,
        pdec!(1),
        dec!(0.03),
        dec!(0.03),
        FAUCET,
        vec![],
        FAUCET,
        ociswap_v2.package,
        env,
    )?;

    // Act
    let rtn = protocol
        .ignition
        .add_allowed_pool(pool.try_into().unwrap(), env);

    // Assert
    assert_is_ignition_neither_pool_resource_is_protocol_resource_error(&rtn);

    Ok(())
}

#[test]
fn cant_add_an_allowed_pool_where_neither_of_the_resources_is_the_protocol_resource_caviarnine_v1(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        caviarnine_v1,
        ..
    } = ScryptoTestEnv::new()?;

    let fungible1 = ResourceBuilder::new_fungible(OwnerRole::None)
        .mint_initial_supply(100, env)?;
    let fungible2 = ResourceBuilder::new_fungible(OwnerRole::None)
        .mint_initial_supply(100, env)?;
    let pool = CaviarnineV1PoolInterfaceScryptoTestStub::new(
        rule!(allow_all),
        rule!(allow_all),
        fungible1.resource_address(env)?,
        fungible2.resource_address(env)?,
        50,
        None,
        caviarnine_v1.package,
        env,
    )?;

    // Act
    let rtn = protocol
        .ignition
        .add_allowed_pool(pool.try_into().unwrap(), env);

    // Assert
    assert_is_ignition_neither_pool_resource_is_protocol_resource_error(&rtn);

    Ok(())
}

#[test]
fn cant_remove_an_allowed_pool_for_a_blueprint_with_no_registered_adapter(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        ..
    } = ScryptoTestEnv::new()?;

    // Act
    let rtn = protocol.ignition.remove_allowed_pool(FAUCET, env);

    // Assert
    assert_is_ignition_no_adapter_found_for_pool_error(&rtn);

    Ok(())
}

#[test]
fn cant_set_liquidity_receipt_of_a_pool_with_no_adapter(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        ..
    } = ScryptoTestEnv::new()?;

    // Act
    let rtn = protocol.ignition.set_liquidity_receipt(
        BlueprintId {
            package_address: FAUCET_PACKAGE,
            blueprint_name: "Faucet".into(),
        },
        ACCOUNT_OWNER_BADGE.into(),
        env,
    );

    // Assert
    assert_is_ignition_no_adapter_found_for_pool_error(&rtn);

    Ok(())
}

#[test]
fn cant_open_a_liquidity_position_with_volatile_user_resource_when_volatile_vault_is_empty(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        resources,
        ociswap_v1,
        ..
    } = ScryptoTestEnv::new()?;

    let _ = env.with_component_state::<IgnitionState, _, _, _>(
        protocol.ignition,
        |state, env| state.protocol_resource_reserves.volatile.0.take_all(env),
    )?;

    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;

    // Act
    let rtn = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        ociswap_v1.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    );

    // Assert
    assert!(rtn.is_err());

    Ok(())
}

#[test]
fn cant_open_a_liquidity_position_with_non_volatile_user_resource_when_non_volatile_vault_is_empty(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        resources,
        ociswap_v1,
        ..
    } = ScryptoTestEnv::new()?;

    let _ = env.with_component_state::<IgnitionState, _, _, _>(
        protocol.ignition,
        |state, env| {
            state
                .protocol_resource_reserves
                .non_volatile
                .0
                .take_all(env)
        },
    )?;

    let usdc_bucket =
        ResourceManager(resources.usdc).mint_fungible(dec!(100), env)?;

    // Act
    let rtn = protocol.ignition.open_liquidity_position(
        FungibleBucket(usdc_bucket),
        ociswap_v1.pools.usdc.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    );

    // Assert
    assert!(rtn.is_err());

    Ok(())
}

#[test]
fn can_open_a_liquidity_position_with_no_protocol_resources_in_user_resources_vaults(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        ociswap_v1,
        resources,
        ..
    } = ScryptoTestEnv::new()?;

    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;

    assert_eq!(
        protocol
            .ignition
            .get_user_resource_reserves_amount(XRD, env)?,
        Decimal::ZERO
    );

    // Act
    let rtn = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        ociswap_v1.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    );

    // Assert
    let _ = rtn.expect("Should succeed!");

    Ok(())
}

#[test]
fn opening_a_liquidity_position_of_a_volatile_resource_consumes_protocol_assets_from_the_volatile_vault(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        ociswap_v1,
        resources,
        ..
    } = ScryptoTestEnv::new()?;

    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;

    let initial_volatile_vault_amount = env
        .with_component_state::<IgnitionState, _, _, _>(
            protocol.ignition,
            |state, env| {
                state.protocol_resource_reserves.volatile.0.amount(env)
            },
        )??;
    let initial_non_volatile_vault_amount = env
        .with_component_state::<IgnitionState, _, _, _>(
            protocol.ignition,
            |state, env| {
                state.protocol_resource_reserves.non_volatile.0.amount(env)
            },
        )??;

    // Act
    let _ = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        ociswap_v1.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    )?;

    // Assert
    let final_volatile_vault_amount = env
        .with_component_state::<IgnitionState, _, _, _>(
            protocol.ignition,
            |state, env| {
                state.protocol_resource_reserves.volatile.0.amount(env)
            },
        )??;
    let final_non_volatile_vault_amount = env
        .with_component_state::<IgnitionState, _, _, _>(
            protocol.ignition,
            |state, env| {
                state.protocol_resource_reserves.non_volatile.0.amount(env)
            },
        )??;

    assert_ne!(initial_volatile_vault_amount, final_volatile_vault_amount);
    assert_eq!(
        initial_non_volatile_vault_amount,
        final_non_volatile_vault_amount
    );

    Ok(())
}

#[test]
fn liquidity_receipt_data_matches_component_state() -> Result<(), RuntimeError>
{
    // Arrange
    const ORACLE_PRICE: Decimal = dec!(0.85);
    const POOL_PRICE: Decimal = dec!(1);
    const BITCOIN_CONTRIBUTION: Decimal = dec!(100);

    const LOCKUP_REWARD: Decimal = dec!(0.2);
    let lockup_period: LockupPeriod = LockupPeriod::from_months(6).unwrap();

    let Environment {
        environment: ref mut env,
        mut protocol,
        ociswap_v1,
        resources,
        ..
    } = ScryptoTestEnv::new()?;
    protocol
        .oracle
        .set_price(resources.bitcoin, XRD, ORACLE_PRICE, env)?;
    protocol
        .ignition
        .set_maximum_allowed_price_difference_percentage(Decimal::MAX, env)?;

    let bitcoin_bucket = ResourceManager(resources.bitcoin)
        .mint_fungible(BITCOIN_CONTRIBUTION, env)?;

    let initial_bitcoin_reserves = protocol
        .ignition
        .get_user_resource_reserves_amount(resources.bitcoin, env)?;
    let initial_volatile_xrd_reserves = protocol
        .ignition
        .get_protocol_resource_reserves_amount(Volatile, env);
    let initial_non_volatile_xrd_reserves = protocol
        .ignition
        .get_protocol_resource_reserves_amount(NonVolatile, env);

    // Act
    let (receipt, upfront_reward, bitcoin_change) =
        protocol.ignition.open_liquidity_position(
            FungibleBucket(bitcoin_bucket),
            ociswap_v1.pools.bitcoin.try_into().unwrap(),
            lockup_period,
            env,
        )?;

    // Assert
    let final_bitcoin_reserves = protocol
        .ignition
        .get_user_resource_reserves_amount(resources.bitcoin, env)?;
    let final_volatile_xrd_reserves = protocol
        .ignition
        .get_protocol_resource_reserves_amount(Volatile, env);
    let final_non_volatile_xrd_reserves = protocol
        .ignition
        .get_protocol_resource_reserves_amount(NonVolatile, env);

    assert_eq!(initial_bitcoin_reserves, final_bitcoin_reserves);
    assert_ne!(initial_volatile_xrd_reserves, final_volatile_xrd_reserves);
    assert_eq!(
        initial_non_volatile_xrd_reserves,
        final_non_volatile_xrd_reserves
    );
    assert!(bitcoin_change.is_empty() || bitcoin_change.len() == 1);
    let bitcoin_change = if bitcoin_change.len() == 1 {
        let bucket = bitcoin_change.first().unwrap();
        let resource_address = bucket.resource_address(env)?;
        assert_eq!(resource_address, resources.bitcoin);
        bucket.amount(env)?
    } else {
        Decimal::ZERO
    };

    let liquidity_receipt_data = ResourceManager(ociswap_v1.liquidity_receipt)
        .get_non_fungible_data::<_, _, LiquidityReceipt<AnyValue>>(
            receipt
                .0
                .non_fungible_local_ids(env)?
                .first()
                .unwrap()
                .clone(),
            env,
        )?;

    assert_eq!(
        liquidity_receipt_data.lockup_period,
        lockup_period.to_string()
    );
    assert_eq!(
        liquidity_receipt_data.pool_address,
        ComponentAddress::try_from(ociswap_v1.pools.bitcoin).unwrap()
    );
    assert_eq!(
        liquidity_receipt_data.user_resource_address,
        resources.bitcoin
    );
    assert_eq!(
        liquidity_receipt_data.user_contribution_amount,
        BITCOIN_CONTRIBUTION - bitcoin_change
    );
    assert_eq!(
        liquidity_receipt_data.user_resource_volatility_classification,
        Volatile
    );
    assert_eq!(
        liquidity_receipt_data.protocol_contribution_amount,
        (BITCOIN_CONTRIBUTION - bitcoin_change) * POOL_PRICE
    );
    assert_eq!(
        liquidity_receipt_data.maturity_date,
        env.get_current_time()
            .add_seconds(*lockup_period.seconds() as i64)
            .unwrap()
    );

    let upfront_reward_resource_address =
        upfront_reward.0.resource_address(env)?;
    let upfront_reward_amount = upfront_reward.0.amount(env)?;
    assert_eq!(upfront_reward_resource_address, XRD);
    assert_eq!(
        upfront_reward_amount,
        (BITCOIN_CONTRIBUTION - bitcoin_change) * ORACLE_PRICE * LOCKUP_REWARD
    );

    Ok(())
}

#[test]
fn cant_close_a_liquidity_position_using_a_fake_nft() -> Result<(), RuntimeError>
{
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        resources,
        ociswap_v1,
        ..
    } = ScryptoTestEnv::new()?;

    let fake_liquidity_receipt =
        ResourceBuilder::new_ruid_non_fungible(OwnerRole::None)
            .mint_roles(mint_roles! {
                minter => rule!(allow_all);
                minter_updater => rule!(allow_all);
            })
            .burn_roles(burn_roles! {
                burner => rule!(allow_all);
                burner_updater => rule!(allow_all);
            })
            .mint_initial_supply(
                [utils::liquidity_receipt_data_with_modifier(|receipt| {
                    receipt.pool_address =
                        ociswap_v1.pools.bitcoin.try_into().unwrap();
                    receipt.user_resource_address = resources.bitcoin
                })],
                env,
            )?;

    // Act
    let rtn = protocol.ignition.close_liquidity_position(
        NonFungibleBucket(fake_liquidity_receipt),
        env,
    );

    // Assert
    assert_is_ignition_not_a_valid_liquidity_receipt_error(&rtn);

    Ok(())
}

#[test]
fn cant_close_a_liquidity_position_when_closing_is_closed(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        resources,
        ociswap_v1,
        ..
    } = ScryptoTestEnv::new()?;
    protocol
        .ignition
        .set_is_close_position_enabled(false, env)?;

    let (bucket, _) = ResourceManager(ociswap_v1.liquidity_receipt)
        .mint_non_fungible_single_ruid(
            utils::liquidity_receipt_data_with_modifier(|receipt| {
                receipt.pool_address =
                    ociswap_v1.pools.bitcoin.try_into().unwrap();
                receipt.user_resource_address = resources.bitcoin
            }),
            env,
        )?;

    // Act
    let rtn = protocol
        .ignition
        .close_liquidity_position(NonFungibleBucket(bucket), env);

    // Assert
    assert_is_ignition_closing_liquidity_positions_is_closed_error(&rtn);

    Ok(())
}

#[test]
fn cant_close_a_liquidity_position_with_more_than_one_nft(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        resources,
        ociswap_v1,
        ..
    } = ScryptoTestEnv::new()?;

    let (bucket1, _) = ResourceManager(ociswap_v1.liquidity_receipt)
        .mint_non_fungible_single_ruid(
            utils::liquidity_receipt_data_with_modifier(|receipt| {
                receipt.pool_address =
                    ociswap_v1.pools.bitcoin.try_into().unwrap();
                receipt.user_resource_address = resources.bitcoin
            }),
            env,
        )?;
    let (bucket2, _) = ResourceManager(ociswap_v1.liquidity_receipt)
        .mint_non_fungible_single_ruid(
            utils::liquidity_receipt_data_with_modifier(|receipt| {
                receipt.pool_address =
                    ociswap_v1.pools.bitcoin.try_into().unwrap();
                receipt.user_resource_address = resources.bitcoin
            }),
            env,
        )?;
    bucket1.put(bucket2, env)?;

    // Act
    let rtn = protocol
        .ignition
        .close_liquidity_position(NonFungibleBucket(bucket1), env);

    // Assert
    assert_is_ignition_more_than_one_liquidity_receipt_nfts_error(&rtn);

    Ok(())
}

#[test]
fn cant_close_a_liquidity_position_before_its_maturity_date(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        ociswap_v1,
        resources,
        ..
    } = ScryptoTestEnv::new()?;

    let (bucket, _) = ResourceManager(ociswap_v1.liquidity_receipt)
        .mint_non_fungible_single_ruid(
            utils::liquidity_receipt_data_with_modifier(|receipt| {
                receipt.pool_address =
                    ociswap_v1.pools.bitcoin.try_into().unwrap();
                receipt.user_resource_address = resources.bitcoin;
                env.set_current_time(Instant::new(60));
                receipt.maturity_date = Instant::new(120);
            }),
            env,
        )?;

    // Act
    let rtn = protocol
        .ignition
        .close_liquidity_position(NonFungibleBucket(bucket), env);

    // Assert
    assert_is_ignition_liquidity_position_has_not_matured_error(&rtn);

    Ok(())
}

#[test]
fn can_close_a_liquidity_position_the_minute_it_matures(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        ociswap_v1,
        resources,
        ..
    } = ScryptoTestEnv::new()?;

    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;
    let (liquidity_receipt, _, _) = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        ociswap_v1.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    )?;

    let liquidity_receipt_data = ResourceManager(ociswap_v1.liquidity_receipt)
        .get_non_fungible_data::<_, _, LiquidityReceipt<AnyValue>>(
            liquidity_receipt
                .0
                .non_fungible_local_ids(env)?
                .first()
                .unwrap()
                .clone(),
            env,
        )?;
    env.set_current_time(liquidity_receipt_data.maturity_date);
    protocol
        .oracle
        .set_price(resources.bitcoin, XRD, dec!(1), env)?;

    // Act
    let rtn = protocol
        .ignition
        .close_liquidity_position(liquidity_receipt, env);

    // Assert
    assert!(rtn.is_ok(), "{rtn:#?}");

    Ok(())
}

#[test]
fn cant_close_a_liquidity_position_of_a_pool_with_no_adapter(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        ociswap_v1,
        ..
    } = ScryptoTestEnv::new()?;
    let (bucket, _) = ResourceManager(ociswap_v1.liquidity_receipt)
        .mint_non_fungible_single_ruid(
            utils::liquidity_receipt_data_with_modifier(|receipt| {
                receipt.pool_address = FAUCET;
            }),
            env,
        )?;

    // Act
    let rtn = protocol
        .ignition
        .close_liquidity_position(NonFungibleBucket(bucket), env);

    // Assert
    assert_is_ignition_no_adapter_found_for_pool_error(&rtn);

    Ok(())
}

#[test]
fn user_gets_back_the_same_amount_they_put_in_when_user_resource_price_goes_down(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        mut ociswap_v1,
        resources,
        ..
    } = ScryptoTestEnv::new()?;

    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;
    let (receipt, _, _) = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        ociswap_v1.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    )?;

    let bitcoin_bucket = ResourceManager(resources.bitcoin)
        .mint_fungible(dec!(100_000_000), env)?;
    let _ = ociswap_v1.pools.bitcoin.swap(bitcoin_bucket, env)?;

    let current_time = env.get_current_time();
    env.set_current_time(
        current_time
            .add_seconds(*LockupPeriod::from_months(6).unwrap().seconds() as i64)
            .unwrap(),
    );

    let pool_price = ociswap_v1
        .adapter
        .price(ociswap_v1.pools.bitcoin.try_into().unwrap(), env)?;
    assert_eq!(pool_price.base, resources.bitcoin);
    assert_eq!(pool_price.quote, XRD);
    protocol.oracle.set_price(
        pool_price.base,
        pool_price.quote,
        pool_price.price,
        env,
    )?;

    // Act
    let assets_back =
        protocol.ignition.close_liquidity_position(receipt, env)?;

    // Assert
    let indexed_buckets =
        IndexedBuckets::native_from_buckets(assets_back, env)?;
    assert_eq!(indexed_buckets.len(), 2);

    assert_eq!(
        indexed_buckets
            .get(&resources.bitcoin)
            .expect("We expect to get bitcoin back!")
            .amount(env)?,
        // Og Amount In + Estimated fees
        dec!(100) + dec!(0.50062606)
    );
    assert_eq!(
        indexed_buckets
            .get(&XRD)
            .expect("We expect to get bitcoin back!")
            .amount(env)?,
        dec!(0)
    );

    Ok(())
}

#[test]
fn user_gets_enough_protocol_resource_to_purchase_back_user_assets_lost_due_to_impermanent_loss(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        mut ociswap_v1,
        resources,
        ..
    } = ScryptoTestEnv::new()?;

    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;
    let (receipt, _, _) = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        ociswap_v1.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    )?;

    let xrd_bucket =
        ResourceManager(XRD).mint_fungible(dec!(10_000_000), env)?;
    let _ = ociswap_v1.pools.bitcoin.swap(xrd_bucket, env)?;

    let current_time = env.get_current_time();
    env.set_current_time(
        current_time
            .add_seconds(*LockupPeriod::from_months(6).unwrap().seconds() as i64)
            .unwrap(),
    );

    let pool_price = ociswap_v1
        .adapter
        .price(ociswap_v1.pools.bitcoin.try_into().unwrap(), env)?;
    let oracle_price = pool_price;
    assert_eq!(pool_price.base, resources.bitcoin);
    assert_eq!(pool_price.quote, XRD);
    protocol.oracle.set_price(
        pool_price.base,
        pool_price.quote,
        oracle_price.price,
        env,
    )?;

    // Act
    let assets_back =
        protocol.ignition.close_liquidity_position(receipt, env)?;

    // Assert
    let indexed_buckets =
        IndexedBuckets::native_from_buckets(assets_back, env)?;
    assert_eq!(indexed_buckets.len(), 2);

    assert_eq!(
        indexed_buckets
            .get(&resources.bitcoin)
            .expect("We expect to get bitcoin back!")
            .amount(env)?,
        dec!(90.99181893)
    );
    assert_eq!(
        indexed_buckets
            .get(&XRD)
            .expect("We expect to get bitcoin back!")
            .amount(env)?,
        (dec!(100) - dec!(90.99181893)) * oracle_price.price
    );

    Ok(())
}

#[test]
fn user_gets_enough_protocol_resource_to_purchase_back_user_assets_lost_due_to_impermanent_loss_according_to_oracle_price_not_pool_price(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        mut ociswap_v1,
        resources,
        ..
    } = ScryptoTestEnv::new()?;

    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;
    let (receipt, _, _) = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        ociswap_v1.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    )?;

    let xrd_bucket =
        ResourceManager(XRD).mint_fungible(dec!(10_000_000), env)?;
    let _ = ociswap_v1.pools.bitcoin.swap(xrd_bucket, env)?;

    let current_time = env.get_current_time();
    env.set_current_time(
        current_time
            .add_seconds(*LockupPeriod::from_months(6).unwrap().seconds() as i64)
            .unwrap(),
    );

    let pool_price = ociswap_v1
        .adapter
        .price(ociswap_v1.pools.bitcoin.try_into().unwrap(), env)?;
    let oracle_price = pool_price.price - (dec!(0.005) * pool_price.price);
    assert_eq!(pool_price.base, resources.bitcoin);
    assert_eq!(pool_price.quote, XRD);
    protocol.oracle.set_price(
        pool_price.base,
        pool_price.quote,
        oracle_price,
        env,
    )?;

    // Act
    let assets_back =
        protocol.ignition.close_liquidity_position(receipt, env)?;

    // Assert
    let indexed_buckets =
        IndexedBuckets::native_from_buckets(assets_back, env)?;
    assert_eq!(indexed_buckets.len(), 2);

    assert_eq!(
        indexed_buckets
            .get(&resources.bitcoin)
            .expect("We expect to get bitcoin back!")
            .amount(env)?,
        dec!(90.99181893)
    );
    assert_eq!(
        indexed_buckets
            .get(&XRD)
            .expect("We expect to get bitcoin back!")
            .amount(env)?,
        (dec!(100) - dec!(90.99181893)) * oracle_price
    );

    Ok(())
}

#[test]
fn amount_of_protocol_resources_returned_to_user_has_an_upper_bound_of_the_amount_obtained_from_the_pool(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        mut ociswap_v1,
        resources,
        ..
    } = ScryptoTestEnv::new()?;

    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;
    let (receipt, _, _) = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        ociswap_v1.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    )?;

    let xrd_bucket =
        ResourceManager(XRD).mint_fungible(dec!(10_000_000_000), env)?;
    let _ = ociswap_v1.pools.bitcoin.swap(xrd_bucket, env)?;

    let current_time = env.get_current_time();
    env.set_current_time(
        current_time
            .add_seconds(*LockupPeriod::from_months(6).unwrap().seconds() as i64)
            .unwrap(),
    );

    let pool_price = ociswap_v1
        .adapter
        .price(ociswap_v1.pools.bitcoin.try_into().unwrap(), env)?;
    let oracle_price = pool_price;
    assert_eq!(pool_price.base, resources.bitcoin);
    assert_eq!(pool_price.quote, XRD);
    protocol.oracle.set_price(
        pool_price.base,
        pool_price.quote,
        oracle_price.price,
        env,
    )?;

    // Act
    let assets_back =
        protocol.ignition.close_liquidity_position(receipt, env)?;

    // Assert
    let indexed_buckets =
        IndexedBuckets::native_from_buckets(assets_back, env)?;
    assert_eq!(indexed_buckets.len(), 2);

    assert_eq!(
        indexed_buckets
            .get(&resources.bitcoin)
            .expect("We expect to get bitcoin back!")
            .amount(env)?,
        dec!(1.00000098)
    );
    assert_eq!(
        indexed_buckets
            .get(&XRD)
            .expect("We expect to get bitcoin back!")
            .amount(env)?,
        dec!(10099.99000000999999)
    );

    Ok(())
}

#[test]
fn protocol_manager_cant_perform_forced_liquidation() -> Result<(), RuntimeError>
{
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        ociswap_v1,
        resources,
        ..
    } = ScryptoTestEnv::new()?;
    env.enable_auth_module();

    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;
    let (receipt, ..) = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        ociswap_v1.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    )?;
    let receipt_global_id = NonFungibleGlobalId::new(
        receipt.0.resource_address(env)?,
        receipt
            .0
            .non_fungible_local_ids(env)?
            .first()
            .unwrap()
            .clone(),
    );

    // Act
    LocalAuthZone::push(
        protocol.protocol_manager_badge.create_proof_of_all(env)?,
        env,
    )?;
    let rtn = protocol
        .ignition
        .forcefully_liquidate(receipt_global_id, env);

    // Assert
    matches!(
        rtn,
        Err(RuntimeError::SystemModuleError(
            SystemModuleError::AuthError(AuthError::Unauthorized(..))
        ))
    );

    Ok(())
}

#[test]
fn protocol_owner_can_perform_forced_liquidation() -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        ociswap_v1,
        resources,
        ..
    } = ScryptoTestEnv::new_with_configuration(Configuration {
        maximum_allowed_price_staleness_seconds: i64::MAX,
        ..Default::default()
    })?;
    env.enable_auth_module();

    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;
    let (receipt, ..) = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        ociswap_v1.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    )?;
    let receipt_global_id = NonFungibleGlobalId::new(
        receipt.0.resource_address(env)?,
        receipt
            .0
            .non_fungible_local_ids(env)?
            .first()
            .unwrap()
            .clone(),
    );

    let current_time = env.get_current_time();
    env.set_current_time(current_time.add_days(7 * 30).unwrap());

    // Act
    LocalAuthZone::push(
        protocol.protocol_owner_badge.create_proof_of_all(env)?,
        env,
    )?;
    let rtn = protocol
        .ignition
        .forcefully_liquidate(receipt_global_id, env);

    // Assert
    assert!(rtn.is_ok());
    Ok(())
}

#[test]
fn protocol_owner_can_perform_forced_liquidation_even_when_liquidation_is_closed(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        ociswap_v1,
        resources,
        ..
    } = ScryptoTestEnv::new_with_configuration(Configuration {
        maximum_allowed_price_staleness_seconds: i64::MAX,
        ..Default::default()
    })?;
    env.enable_auth_module();

    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;
    let (receipt, ..) = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        ociswap_v1.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    )?;
    let receipt_global_id = NonFungibleGlobalId::new(
        receipt.0.resource_address(env)?,
        receipt
            .0
            .non_fungible_local_ids(env)?
            .first()
            .unwrap()
            .clone(),
    );

    let current_time = env.get_current_time();
    env.set_current_time(current_time.add_days(7 * 30).unwrap());

    LocalAuthZone::push(
        protocol.protocol_owner_badge.create_proof_of_all(env)?,
        env,
    )?;
    protocol
        .ignition
        .set_is_close_position_enabled(false, env)?;

    // Act
    let rtn = protocol
        .ignition
        .forcefully_liquidate(receipt_global_id, env);

    // Assert
    assert!(rtn.is_ok());
    Ok(())
}

#[test]
fn protocol_owner_cant_perform_forced_liquidation_before_maturity_date(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        ociswap_v1,
        resources,
        ..
    } = ScryptoTestEnv::new_with_configuration(Configuration {
        maximum_allowed_price_staleness_seconds: i64::MAX,
        ..Default::default()
    })?;
    env.enable_auth_module();

    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;
    let (receipt, ..) = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        ociswap_v1.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    )?;
    let receipt_global_id = NonFungibleGlobalId::new(
        receipt.0.resource_address(env)?,
        receipt
            .0
            .non_fungible_local_ids(env)?
            .first()
            .unwrap()
            .clone(),
    );

    // Act
    LocalAuthZone::push(
        protocol.protocol_owner_badge.create_proof_of_all(env)?,
        env,
    )?;
    let rtn = protocol
        .ignition
        .forcefully_liquidate(receipt_global_id, env);

    // Assert
    assert_is_ignition_liquidity_position_has_not_matured_error(&rtn);
    Ok(())
}

#[test]
fn forcefully_liquidated_resources_can_be_claimed_when_closing_liquidity_position(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        ociswap_v1,
        resources,
        ..
    } = ScryptoTestEnv::new_with_configuration(Configuration {
        maximum_allowed_price_staleness_seconds: i64::MAX,
        ..Default::default()
    })?;

    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;
    let (receipt, ..) = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        ociswap_v1.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    )?;
    let receipt_global_id = NonFungibleGlobalId::new(
        receipt.0.resource_address(env)?,
        receipt
            .0
            .non_fungible_local_ids(env)?
            .first()
            .unwrap()
            .clone(),
    );

    let current_time = env.get_current_time();
    env.set_current_time(current_time.add_days(7 * 30).unwrap());

    protocol
        .ignition
        .forcefully_liquidate(receipt_global_id, env)?;

    // Act
    let buckets = protocol.ignition.close_liquidity_position(receipt, env)?;

    // Assert
    let buckets = IndexedBuckets::native_from_buckets(buckets, env)?;
    let bitcoin_bucket = buckets.get(&resources.bitcoin).unwrap();
    let amount = bitcoin_bucket.amount(env)?;
    assert_eq!(amount, dec!(99.99999999));
    Ok(())
}

#[test]
fn forcefully_liquidated_resources_can_be_claimed_when_closing_liquidity_position_even_when_closing_is_disabled(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        ociswap_v1,
        resources,
        ..
    } = ScryptoTestEnv::new_with_configuration(Configuration {
        maximum_allowed_price_staleness_seconds: i64::MAX,
        ..Default::default()
    })?;

    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;
    let (receipt, ..) = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        ociswap_v1.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6).unwrap(),
        env,
    )?;
    let receipt_global_id = NonFungibleGlobalId::new(
        receipt.0.resource_address(env)?,
        receipt
            .0
            .non_fungible_local_ids(env)?
            .first()
            .unwrap()
            .clone(),
    );

    let current_time = env.get_current_time();
    env.set_current_time(current_time.add_days(7 * 30).unwrap());

    protocol
        .ignition
        .forcefully_liquidate(receipt_global_id, env)?;

    protocol
        .ignition
        .set_is_close_position_enabled(false, env)?;

    // Act
    let buckets = protocol.ignition.close_liquidity_position(receipt, env)?;

    // Assert
    let buckets = IndexedBuckets::native_from_buckets(buckets, env)?;
    let bitcoin_bucket = buckets.get(&resources.bitcoin).unwrap();
    let amount = bitcoin_bucket.amount(env)?;
    assert_eq!(amount, dec!(99.99999999));
    Ok(())
}

mod utils {
    use super::*;

    pub fn liquidity_receipt_data() -> LiquidityReceipt<AnyValue> {
        LiquidityReceipt {
            name: "Some name".to_owned(),
            lockup_period: "6 months".to_owned(),
            pool_address: FAUCET,
            user_resource_address: XRD,
            user_contribution_amount: dec!(100_000_000_000),
            user_resource_volatility_classification: NonVolatile,
            protocol_contribution_amount: dec!(1),
            maturity_date: Instant::new(1),
            adapter_specific_information: OciswapV1AdapterSpecificInformation {
                pool_k_when_position_opened: pdec!(100),
                user_share_in_pool_when_position_opened: dec!(0.01),
            }
            .into(),
        }
    }

    pub fn liquidity_receipt_data_with_modifier(
        modifier: impl FnOnce(&mut LiquidityReceipt<AnyValue>),
    ) -> LiquidityReceipt<AnyValue> {
        let mut data = liquidity_receipt_data();
        modifier(&mut data);
        data
    }
}
