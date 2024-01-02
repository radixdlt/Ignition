//! Protocol related behavior tests.

mod utils;
use utils::*;

use olympus::LockupPeriod;
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
        ociswap,
    } = Environment::new()?;

    protocol.oracle.set_price(
        resources.bitcoin.0,
        XRD,
        dec!(1_037_305.4202264115),
        env,
    )?;

    // Act
    let bitcoin_bucket = resources.bitcoin.mint_fungible(dec!(0.1), env)?;
    let (_, change, additional_resources) =
        protocol.olympus.open_liquidity_position(
            ociswap.bitcoin_pool.try_into().unwrap(),
            FungibleBucket(bitcoin_bucket),
            LockupPeriod::from_months(6),
            env,
        )?;

    // Assert
    assert_eq!(
        change.0.amount(env),
        Ok(dec!(0.1) * dec!(1_037_305.4202264115) * dec!(0.1))
    );
    assert!(additional_resources.is_empty());

    Ok(())
}
