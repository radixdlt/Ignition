//! Protocol related behavior tests.

mod utils;

use adapters::oracle::*;
use olympus::test_bindings::*;

use scrypto_test::prelude::*;
use utils::environments::*;
use utils::errors::*;

#[test]
fn cant_add_a_pool_with_no_corresponding_adapter() -> Result<(), RuntimeError> {
    // Arrange
    let Environment {
        environment: ref mut env,
        ..
    } = new_test_environment();

    let (code, definition) =
        utils::package_loader::PackageLoader::get("olympus");
    let (package_address, _) =
        Package::publish(code, definition, Default::default(), env).unwrap();
    let mut olympus = Olympus::instantiate(
        OwnerRole::None,
        rule!(allow_all),
        rule!(allow_all),
        OracleAdapter(Reference(FAUCET.into_node_id())),
        XRD,
        None,
        package_address,
        env,
    )?;

    // Act
    let rtn = olympus.add_allowed_pool(FAUCET, env);

    // Assert
    dbg!(&rtn);
    assert!(is_wasm_panic(&rtn));

    Ok(())
}
