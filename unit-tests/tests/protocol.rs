//! Protocol related behavior tests.

mod utils;
use utils::*;

use ignition::LockupPeriod;
use ociswap_adapter::*;

use radix_engine_interface::prelude::*;
use radix_engine_interface::*;
use scrypto_test::prelude::*;

#[test]
fn cant_add_a_pool_with_no_corresponding_adapter() -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        protocol: ProtocolEntities { mut ignition, .. },
        ..
    } = Environment::new()?;

    // Act
    let rtn = ignition.add_allowed_pool(FAUCET, env);

    // Assert
    assert_is_add_allowed_pool_no_adapter_found_for_pool(&rtn);

    Ok(())
}

#[test]
fn cant_open_a_liquidity_position_when_opening_is_disabled(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut resources,
        mut protocol,
        ociswap,
    } = Environment::new()?;
    protocol
        .ignition
        .config_open_liquidity_position(false, env)?;

    // Act
    let bitcoin_contribution_amount = dec!(0.1);
    let bitcoin_bucket = resources
        .bitcoin
        .mint_fungible(bitcoin_contribution_amount, env)?;
    let rtn = protocol.ignition.open_liquidity_position(
        ociswap.bitcoin_pool.try_into().unwrap(),
        FungibleBucket(bitcoin_bucket),
        LockupPeriod::from_months(6),
        env,
    );

    // Assert
    assert_is_open_liquidity_position_opening_disabled_error(&rtn);

    Ok(())
}

#[test]
fn cant_open_a_liquidity_position_to_a_registered_pool_with_no_adapter(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut resources,
        mut protocol,
        ociswap,
    } = Environment::new()?;
    protocol.ignition.remove_pool_adapter(
        OciswapPoolInterfaceScryptoTestStub::blueprint_id(ociswap.package),
        env,
    )?;

    protocol
        .oracle
        .set_price(resources.bitcoin.0, XRD, dec!(1), env)?;

    // Act
    let bitcoin_contribution_amount = dec!(0.1);
    let bitcoin_bucket = resources
        .bitcoin
        .mint_fungible(bitcoin_contribution_amount, env)?;
    let rtn = protocol.ignition.open_liquidity_position(
        ociswap.bitcoin_pool.try_into().unwrap(),
        FungibleBucket(bitcoin_bucket),
        LockupPeriod::from_months(6),
        env,
    );

    // Assert
    assert_is_open_liquidity_position_no_adapter_error(&rtn);

    Ok(())
}

#[test]
fn cant_open_a_liquidity_position_on_a_pool_that_is_not_registered(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut resources,
        mut protocol,
        ociswap,
    } = Environment::new()?;
    protocol
        .ignition
        .remove_allowed_pool(ociswap.bitcoin_pool.try_into().unwrap(), env)?;

    protocol
        .oracle
        .set_price(resources.bitcoin.0, XRD, dec!(1), env)?;

    // Act
    let bitcoin_contribution_amount = dec!(0.1);
    let bitcoin_bucket = resources
        .bitcoin
        .mint_fungible(bitcoin_contribution_amount, env)?;
    let rtn = protocol.ignition.open_liquidity_position(
        ociswap.bitcoin_pool.try_into().unwrap(),
        FungibleBucket(bitcoin_bucket),
        LockupPeriod::from_months(6),
        env,
    );

    // Assert
    assert_is_open_liquidity_position_pool_not_allowed_error(&rtn);

    Ok(())
}

#[test]
fn cant_open_a_liquidity_position_with_an_undefined_lockup_period(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut resources,
        mut protocol,
        ociswap,
    } = Environment::new()?;

    let xrd_price = dec!(0.04204);
    let bitcoin_price = dec!(45108.32);
    let price_bitcoin_base_xrd_quote = bitcoin_price / xrd_price;

    let lockup_period = LockupPeriod::from_months(1);

    protocol.oracle.set_price(
        resources.bitcoin.0,
        XRD,
        price_bitcoin_base_xrd_quote,
        env,
    )?;

    // Act
    let bitcoin_contribution_amount = dec!(0.1);
    let bitcoin_bucket = resources
        .bitcoin
        .mint_fungible(bitcoin_contribution_amount, env)?;
    let rtn = protocol.ignition.open_liquidity_position(
        ociswap.bitcoin_pool.try_into().unwrap(),
        FungibleBucket(bitcoin_bucket),
        lockup_period,
        env,
    );

    // Assert
    assert_is_open_liquidity_position_not_a_valid_lockup_period_error(&rtn);

    Ok(())
}

#[test]
fn cant_open_a_liquidity_position_in_a_pool_where_xrd_is_not_one_of_the_resources(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut resources,
        mut protocol,
        ociswap,
    } = Environment::new()?;

    let pool = OciswapPoolInterfaceScryptoTestStub::instantiate(
        resources.bitcoin.0,
        resources.ethereum.0,
        dec!(0),
        FAUCET,
        ociswap.package,
        env,
    )?;
    protocol
        .ignition
        .add_allowed_pool(pool.try_into().unwrap(), env)?;

    protocol
        .oracle
        .set_price(resources.bitcoin.0, XRD, dec!(100), env)?;

    let btc = resources.bitcoin.mint_fungible(dec!(100), env)?;

    // Act
    let rtn = protocol.ignition.open_liquidity_position(
        pool.try_into().unwrap(),
        FungibleBucket(btc),
        LockupPeriod::from_months(6),
        env,
    );

    // Assert
    assert_is_open_liquidity_position_neither_side_is_xrd_error(&rtn);

    Ok(())
}
