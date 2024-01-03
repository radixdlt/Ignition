//! Protocol related behavior tests.

mod utils;
use utils::*;

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
    assert!(is_wasm_panic(&rtn));

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

    Ok(())
}
