//! Protocol related behavior tests.

mod utils;

use scrypto_test::prelude::*;
use utils::environments::*;
use utils::errors::*;

#[test]
fn cant_add_a_pool_with_no_corresponding_adapter() -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        mut olympus,
        ..
    } = Environment::new()?;

    // Act
    let rtn = olympus.add_allowed_pool(FAUCET, env);

    // Assert
    dbg!(&rtn);
    assert!(is_wasm_panic(&rtn));

    Ok(())
}
