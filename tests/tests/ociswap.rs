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
    } = Environment::new()?;

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
