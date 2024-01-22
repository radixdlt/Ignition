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
    } = Environment::new()?;

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
        "https://assets.caviarnine.com/tokens/resource_rdx1t5pyvlaas0ljxy0wytm5gvyamyv896m69njqdmm2stukr3xexc2up9.png"
    );
    assert_eq!(
        liquidity_receipt_data.redemption_url.0,
        "https://ociswap.com/icons/oci.png"
    );

    Ok(())
}
