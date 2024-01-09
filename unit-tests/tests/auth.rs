#![allow(unused_variables)]

mod utils;
use utils::*;

use adapters_interface::common::*;
use olympus::types::*;
use scrypto_test::prelude::*;

macro_rules! test_access_rules {
    (
        $method_name: ident ( $($arg: expr),* $(,)? ), $role: ident $(,)?
    ) => {
            paste::paste! {
                #[test]
                fn [< can_call_ $method_name _with_ $role _role >]()
                    -> ::std::result::Result<
                        (),
                        ::scrypto_test::prelude::RuntimeError
                    >
                {
                    // Arrange
                    let Environment {
                        environment: ref mut env,
                        mut protocol,
                        ociswap,
                        ..
                    } = Environment::new()?;
                    env.enable_auth_module();

                    let proof = protocol.[< $role _badge >].create_proof_of_all(env)?;
                    LocalAuthZone::push(proof, env)?;

                    // Act
                    let rtn = protocol.olympus.$method_name( $( $arg ),* , env);

                    // Assert
                    assert!(!matches!(
                        rtn,
                        Err(RuntimeError::SystemModuleError(
                            SystemModuleError::AuthError(AuthError::Unauthorized(..))
                        ))
                    ));

                    ::std::result::Result::Ok(())
                    }
            }
    };
    (
        $method_name: ident ( $($arg: expr),* $(,)? ) $(,)?
    ) => {
            paste::paste! {
                #[test]
                fn [< cant_call_ $method_name _without_valid_roles >]()
                    -> ::std::result::Result<(), ::scrypto_test::prelude::RuntimeError>
                {
                    // Arrange
                    let Environment {
                        environment: ref mut env,
                        mut protocol,
                        ociswap,
                        ..
                    } = Environment::new()?;
                    env.enable_auth_module();

                    // Act
                    let rtn = protocol.olympus.$method_name( $( $arg ),* , env);

                    // Assert
                    assert!(matches!(
                        rtn,
                        Err(RuntimeError::SystemModuleError(
                            SystemModuleError::AuthError(AuthError::Unauthorized(..))
                        ))
                    ));

                    ::std::result::Result::Ok(())
                    }
            }
    };
}

test_access_rules!(update_oracle(FAUCET), protocol_manager);
test_access_rules!(update_oracle(FAUCET), protocol_owner);
test_access_rules!(update_oracle(FAUCET));
test_access_rules!(add_allowed_pool(FAUCET), protocol_manager);
test_access_rules!(add_allowed_pool(FAUCET), protocol_owner);
test_access_rules!(add_allowed_pool(FAUCET));
test_access_rules!(remove_allowed_pool(FAUCET), protocol_manager);
test_access_rules!(remove_allowed_pool(FAUCET), protocol_owner);
test_access_rules!(remove_allowed_pool(FAUCET));
test_access_rules!(config_open_liquidity_position(true), protocol_manager);
test_access_rules!(config_open_liquidity_position(true), protocol_owner);
test_access_rules!(config_open_liquidity_position(true));
test_access_rules!(config_close_liquidity_position(true), protocol_manager);
test_access_rules!(config_close_liquidity_position(true), protocol_owner);
test_access_rules!(config_close_liquidity_position(true));
test_access_rules!(
    add_pool_adapter(
        BlueprintId {
            package_address: ACCOUNT_PACKAGE,
            blueprint_name: "Account".to_string()
        },
        FAUCET
    ),
    protocol_manager
);
test_access_rules!(
    add_pool_adapter(
        BlueprintId {
            package_address: ACCOUNT_PACKAGE,
            blueprint_name: "Account".to_string()
        },
        FAUCET
    ),
    protocol_owner
);
test_access_rules!(add_pool_adapter(
    BlueprintId {
        package_address: ACCOUNT_PACKAGE,
        blueprint_name: "Account".to_string()
    },
    FAUCET
));
test_access_rules!(
    remove_pool_adapter(BlueprintId {
        package_address: ACCOUNT_PACKAGE,
        blueprint_name: "Account".to_string()
    },),
    protocol_manager
);
test_access_rules!(
    remove_pool_adapter(BlueprintId {
        package_address: ACCOUNT_PACKAGE,
        blueprint_name: "Account".to_string()
    },),
    protocol_owner
);
test_access_rules!(remove_pool_adapter(BlueprintId {
    package_address: ACCOUNT_PACKAGE,
    blueprint_name: "Account".to_string()
},));
test_access_rules!(
    deposit(scrypto::prelude::FungibleBucket(Bucket(Own(
        FAUCET.into_node_id()
    )))),
    protocol_owner
);
test_access_rules!(deposit(scrypto::prelude::FungibleBucket(Bucket(Own(
    FAUCET.into_node_id()
)))));
test_access_rules!(withdraw(XRD, dec!(20)), protocol_owner);
test_access_rules!(withdraw(XRD, dec!(20)));
test_access_rules!(
    withdraw_pool_units(NonFungibleGlobalId::new(
        ACCOUNT_OWNER_BADGE,
        NonFungibleLocalId::integer(0)
    )),
    protocol_owner
);
test_access_rules!(withdraw_pool_units(NonFungibleGlobalId::new(
    ACCOUNT_OWNER_BADGE,
    NonFungibleLocalId::integer(0)
)));
test_access_rules!(
    add_rewards_rate(
        LockupPeriod::from_seconds(10),
        Percent::new(dec!(0.5)).unwrap()
    ),
    protocol_owner
);
test_access_rules!(add_rewards_rate(
    LockupPeriod::from_seconds(10),
    Percent::new(dec!(0.5)).unwrap()
));
test_access_rules!(
    remove_rewards_rate(LockupPeriod::from_seconds(10)),
    protocol_owner
);
test_access_rules!(remove_rewards_rate(LockupPeriod::from_seconds(10)));
test_access_rules!(update_usd_resource_address(XRD), protocol_owner);
test_access_rules!(update_usd_resource_address(XRD));
