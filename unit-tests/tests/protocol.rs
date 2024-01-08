//! Protocol related behavior tests.

mod utils;
use utils::*;

use ociswap_adapter::*;
use olympus::{LiquidityPosition, LockupPeriod, Percent};

use radix_engine_interface::prelude::*;
use radix_engine_interface::*;
use scrypto_test::prelude::*;

#[test]
fn cant_add_a_pool_with_no_corresponding_adapter() -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        protocol: ProtocolEntities { mut olympus, .. },
        ..
    } = Environment::new()?;

    // Act
    let rtn = olympus.add_allowed_pool(FAUCET, env);

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
        .olympus
        .config_open_liquidity_position(false, env)?;

    // Act
    let bitcoin_contribution_amount = dec!(0.1);
    let bitcoin_bucket = resources
        .bitcoin
        .mint_fungible(bitcoin_contribution_amount, env)?;
    let rtn = protocol.olympus.open_liquidity_position(
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
    protocol.olympus.remove_pool_adapter(
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
    let rtn = protocol.olympus.open_liquidity_position(
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
        .olympus
        .remove_allowed_pool(ociswap.bitcoin_pool.try_into().unwrap(), env)?;

    protocol
        .oracle
        .set_price(resources.bitcoin.0, XRD, dec!(1), env)?;

    // Act
    let bitcoin_contribution_amount = dec!(0.1);
    let bitcoin_bucket = resources
        .bitcoin
        .mint_fungible(bitcoin_contribution_amount, env)?;
    let rtn = protocol.olympus.open_liquidity_position(
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
    let rtn = protocol.olympus.open_liquidity_position(
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
fn can_open_position_on_an_ociswap_pool() -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut resources,
        mut protocol,
        mut ociswap,
    } = Environment::new()?;

    let xrd_price = dec!(0.04204);
    let bitcoin_price = dec!(45108.32);
    let price_bitcoin_base_xrd_quote = bitcoin_price / xrd_price;

    let lockup_period = LockupPeriod::from_months(6);
    let upfront_reward_percentage =
        Percent::new(dec!(0.1)).expect("Must succeed!");

    protocol.oracle.set_price(
        resources.bitcoin.0,
        XRD,
        price_bitcoin_base_xrd_quote,
        env,
    )?;

    {
        let bitcoin_bucket = resources.bitcoin.mint_fungible(dec!(1), env)?;
        let xrd_bucket = ResourceManager(XRD)
            .mint_fungible(price_bitcoin_base_xrd_quote, env)?;
        let _ = ociswap.adapter.open_liquidity_position(
            ociswap.bitcoin_pool.try_into().unwrap(),
            (bitcoin_bucket, xrd_bucket),
            env,
        )?;
    }

    // Act
    let bitcoin_contribution_amount = dec!(0.1);
    let bitcoin_bucket = resources
        .bitcoin
        .mint_fungible(bitcoin_contribution_amount, env)?;
    let (liquidity_position, change, additional_resources) =
        protocol.olympus.open_liquidity_position(
            ociswap.bitcoin_pool.try_into().unwrap(),
            FungibleBucket(bitcoin_bucket),
            lockup_period,
            env,
        )?;

    // Assert
    assert_eq!(
        change.0.amount(env),
        Ok(price_bitcoin_base_xrd_quote
            * bitcoin_contribution_amount
            * *upfront_reward_percentage)
    );
    assert!(additional_resources.is_empty());

    let liquidity_position_resource_address =
        liquidity_position.0.resource_address(env)?;
    let liquidity_position_local_id = liquidity_position
        .0
        .non_fungible_local_ids(env)?
        .first()
        .unwrap()
        .clone();
    let non_fungible_data =
        ResourceManager(liquidity_position_resource_address)
            .get_non_fungible_data::<_, _, LiquidityPosition>(
            liquidity_position_local_id,
            env,
        )?;

    assert_eq!(non_fungible_data.lockup_period, "6months");
    assert_eq!(non_fungible_data.contributed_resource, resources.bitcoin.0);
    assert_eq!(
        non_fungible_data.contributed_amount,
        bitcoin_contribution_amount
    );
    assert_eq!(
        non_fungible_data.matched_xrd_amount,
        price_bitcoin_base_xrd_quote * bitcoin_contribution_amount
    );
    assert_eq!(
        non_fungible_data.maturity_date,
        env.get_current_time()
            .add_seconds(*lockup_period.seconds() as i64)
            .unwrap()
    );

    let adapter_specific_data = non_fungible_data
        .adapter_specific_data
        .as_typed::<OciswapAdapterData>()
        .expect("Must succeed!");
    assert_eq!(
        adapter_specific_data.k_value_when_opening_the_position,
        pdec!(1.1)
            * (PreciseDecimal::from(price_bitcoin_base_xrd_quote)
                + price_bitcoin_base_xrd_quote * bitcoin_contribution_amount)
    );
    assert_eq!(
        adapter_specific_data
            .share_in_pool_when_opening_position
            .checked_round(6, RoundingMode::ToNearestMidpointTowardZero),
        Percent::new(dec!(0.1) / dec!(1.1))
            .unwrap()
            .checked_round(6, RoundingMode::ToNearestMidpointTowardZero),
    );

    Ok(())
}
