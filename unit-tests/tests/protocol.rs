//! Protocol related behavior tests.

mod utils;
use utils::*;

use ociswap_adapter::*;
use olympus::{LockupPeriod, Percent};
use radix_engine_interface::prelude::*;
use radix_engine_interface::*;
use scrypto_test::prelude::*;

#[test]
fn cant_add_a_pool_with_no_corresponding_adapter() -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        components: Components { mut olympus, .. },
        ..
    } = Environment::new()?;

    // Act
    let rtn = olympus.add_allowed_pool(FAUCET, env);

    // Assert
    dbg!(&rtn);
    assert!(is_wasm_panic(&rtn));

    Ok(())
}

#[test]
fn can_open_position_on_an_ociswap_pool() -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        packages,
        mut resources,
        components:
            Components {
                mut olympus,
                test_oracle: mut oracle,
                ociswap_adapter,
            },
        ..
    } = Environment::new_create_badges()?;
    env.disable_auth_module();

    let initial_xrd =
        ResourceManager(XRD).mint_fungible(dec!(100_000_000_000), env)?;
    olympus.deposit(FungibleBucket(initial_xrd), env)?;
    olympus.update_oracle(oracle.0.try_into().unwrap(), env)?;

    let bucket1 =
        ResourceManager(XRD).mint_fungible(dec!(1_037_305.4202264115), env)?;
    let bucket2 = resources.bitcoin.mint_fungible(dec!(1), env)?;

    let (ociswap, _, _) =
        OciswapPoolInterfaceScryptoTestStub::instantiate_with_liquidity(
            bucket1,
            bucket2,
            dec!(0),
            FAUCET,
            packages.ociswap_package,
            env,
        )?;

    olympus.add_pool_adapter(
        OciswapPoolInterfaceScryptoTestStub::blueprint_id(
            packages.ociswap_package,
        ),
        ociswap_adapter.try_into().unwrap(),
        env,
    )?;
    olympus.add_allowed_pool(ociswap.try_into().unwrap(), env)?;
    olympus.config_open_liquidity_position(true, env)?;
    olympus.add_rewards_rate(
        LockupPeriod::from_seconds(10),
        Percent::new(dec!(0.5)).unwrap(),
        env,
    )?;

    oracle.set_price(
        resources.bitcoin.0,
        XRD,
        dec!(1_037_305.4202264115),
        env,
    )?;

    // Act
    let bitcoin_bucket = resources.bitcoin.mint_fungible(dec!(0.1), env)?;
    let (_, change, additional_resources) = olympus.open_liquidity_position(
        ociswap.try_into().unwrap(),
        FungibleBucket(bitcoin_bucket),
        LockupPeriod::from_seconds(10),
        env,
    )?;

    // Assert
    assert_eq!(
        change.0.amount(env),
        Ok(dec!(0.1) * dec!(1_037_305.4202264115) / dec!(2))
    );
    assert!(additional_resources.is_empty());

    Ok(())
}
