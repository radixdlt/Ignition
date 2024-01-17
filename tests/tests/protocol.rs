use tests::prelude::*;

#[test]
fn simple_testing_environment_can_be_created() {
    Environment::new().expect("Must succeed!");
}

#[test]
fn cant_open_a_liquidity_position_when_opening_is_disabled(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        resources,
        mut protocol,
        ociswap,
        ..
    } = Environment::new()?;

    protocol.ignition.set_is_open_position_enabled(false, env)?;
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
    } = Environment::new()?;

    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;

    // Act
    let rtn = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        FAUCET,
        LockupPeriod::from_months(6),
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
        ociswap,
        ..
    } = Environment::new()?;

    let new_pool = OciswapPoolInterfaceScryptoTestStub::instantiate(
        resources.bitcoin,
        XRD,
        dec!(0),
        FAUCET,
        ociswap.package,
        env,
    )?;
    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;

    // Act
    let rtn = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        new_pool.try_into().unwrap(),
        LockupPeriod::from_months(6),
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
        ociswap,
        ..
    } = Environment::new()?;

    protocol
        .ignition
        .remove_allowed_pool(ociswap.pools.bitcoin.try_into().unwrap(), env)?;
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
        ociswap,
        ..
    } = Environment::new()?;

    let random_resource = ResourceBuilder::new_fungible(OwnerRole::None)
        .mint_initial_supply(100, env)?;

    // Act
    let rtn = protocol.ignition.open_liquidity_position(
        FungibleBucket(random_resource),
        ociswap.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6),
        env,
    );

    // Assert
    assert_is_ignition_user_asset_does_not_belong_to_pool_error(&rtn);

    Ok(())
}

#[test]
fn cant_open_a_liquidity_position_by_providing_the_protocol_resource(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        ociswap,
        ..
    } = Environment::new()?;

    let protocol_resource =
        ResourceManager(XRD).mint_fungible(dec!(100), env)?;

    // Act
    let rtn = protocol.ignition.open_liquidity_position(
        FungibleBucket(protocol_resource),
        ociswap.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6),
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
        ociswap,
        resources,
        ..
    } = Environment::new_with_configuration(Configuration {
        maximum_allowed_price_staleness_seconds: 5 * 60,
        ..Default::default()
    })?;

    // Act
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
pub fn can_open_a_liquidity_position_right_before_price_goes_stale(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        ociswap,
        resources,
        ..
    } = Environment::new_with_configuration(Configuration {
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
        ociswap.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6),
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
        ociswap,
        resources,
        ..
    } = Environment::new_with_configuration(Configuration {
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
        ociswap.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6),
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
        ociswap,
        resources,
        ..
    } = Environment::new_with_configuration(Configuration {
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
        ociswap.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6),
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
    test_oracle_price(dec!(1) / dec!(1.01), dec!(0.01))
        .expect("Should succeed!");
    test_oracle_price(dec!(1) / dec!(0.99), dec!(0.01))
        .expect("Should succeed!");
    test_oracle_price(
        dec!(1) / dec!(0.99) - dec!(0.000000000000000010),
        dec!(0.01),
    )
    .expect("Should succeed!");
    test_oracle_price(
        dec!(1) / dec!(1.01) + dec!(0.000000000000000010),
        dec!(0.01),
    )
    .expect("Should succeed!");

    assert_is_ignition_relative_price_difference_larger_than_allowed_error(
        &test_oracle_price(
            dbg!(dec!(1) / dec!(0.99) + dec!(0.000000000000000010)),
            dec!(0.01),
        ),
    );
    assert_is_ignition_relative_price_difference_larger_than_allowed_error(
        &test_oracle_price(
            dec!(1) / dec!(1.01) - dec!(0.000000000000000010),
            dec!(0.01),
        ),
    );
}

fn test_oracle_price(
    oracle_price: Decimal,
    allowed_price_difference: Decimal,
) -> Result<(NonFungibleBucket, FungibleBucket, Vec<Bucket>), RuntimeError> {
    let Environment {
        environment: ref mut env,
        mut protocol,
        ociswap,
        resources,
        ..
    } = Environment::new_with_configuration(Configuration {
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
        ociswap.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6),
        env,
    )
}
