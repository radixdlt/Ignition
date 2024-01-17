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
    } = Environment::new()?;

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
