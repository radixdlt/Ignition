use tests::prelude::*;

#[test]
pub fn can_open_a_simple_position_against_an_caviarnine_pool(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        caviarnine,
        resources,
        ..
    } = Environment::new()?;
    protocol
        .ignition
        .set_maximum_allowed_price_difference_percentage(dec!(0.03), env)?;

    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;

    // Act
    let rtn = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        caviarnine.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6),
        env,
    );

    // Assert
    let _ = rtn.expect("Should succeed!");

    Ok(())
}

#[test]
fn caviarnine_liquidity_receipts_are_caviarnine_branded(
) -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut protocol,
        resources,
        caviarnine,
        ..
    } = Environment::new()?;
    protocol
        .ignition
        .set_maximum_allowed_price_difference_percentage(dec!(0.03), env)?;

    let bitcoin_bucket =
        ResourceManager(resources.bitcoin).mint_fungible(dec!(100), env)?;

    let (liquidity_receipt, _, _) = protocol.ignition.open_liquidity_position(
        FungibleBucket(bitcoin_bucket),
        caviarnine.pools.bitcoin.try_into().unwrap(),
        LockupPeriod::from_months(6),
        env,
    )?;

    // Act
    let liquidity_receipt_data = ResourceManager(caviarnine.liquidity_receipt)
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
    assert_eq!(liquidity_receipt_data.name, "Caviarnine Liquidity Receipt");
    assert_eq!(
        liquidity_receipt_data.description,
        "A receipt of liquidity provided to a Caviarnine pool through the Ignition protocol"
    );
    assert_eq!(
        liquidity_receipt_data.description,
        "A receipt of liquidity provided to a Caviarnine pool through the Ignition protocol"
    );
    assert_eq!(
        liquidity_receipt_data.key_image_url.0,
        "https://assets.caviarnine.com/tokens/resource_rdx1t5pyvlaas0ljxy0wytm5gvyamyv896m69njqdmm2stukr3xexc2up9.png"
    );
    assert_eq!(
        liquidity_receipt_data.redemption_url.0,
        "https://www.caviarnine.com/"
    );

    Ok(())
}
