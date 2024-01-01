#![allow(unused_variables)]

mod utils;

use scrypto_test::prelude::*;
use utils::environments::*;

use adapters_interface::oracle::*;
use olympus::types::*;

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
    add_rewards_rate(10, Percent::new(dec!(0.5)).unwrap()),
    protocol_owner
);
test_access_rules!(add_rewards_rate(10, Percent::new(dec!(0.5)).unwrap()));
test_access_rules!(remove_rewards_rate(10), protocol_owner);
test_access_rules!(remove_rewards_rate(10));
test_access_rules!(update_usd_resource_address(XRD), protocol_owner);
test_access_rules!(update_usd_resource_address(XRD));

macro_rules! test_access_rules {
    (
        $method_name: ident ( $($arg: expr),* $(,)? ), $role: ident $(,)?
    ) => {
            paste::paste! {
                #[test]
                fn [< can_call_ $method_name _with_ $role _role >]()
                    -> ::std::result::Result<(), ::scrypto_test::prelude::RuntimeError>
                {
                    // Arrange
                    let Environment {
                        environment: ref mut env,
                        mut olympus,
                        additional_data: (protocol_manager_badge, protocol_owner_badge),
                        ..
                    } = Environment::new_with_olympus_config(
                        |env| {
                            let protocol_manager_badge =
                                ::scrypto_test::prelude::ResourceBuilder::new_fungible(OwnerRole::None)
                                    .divisibility(0)
                                    .mint_initial_supply(1, env)?;
                            let protocol_owner_badge =
                                ::scrypto_test::prelude::ResourceBuilder::new_fungible(OwnerRole::None)
                                    .divisibility(0)
                                    .mint_initial_supply(1, env)?;

                            let protocol_manager_resource_address =
                                protocol_manager_badge.resource_address(env)?;
                            let protocol_owner_resource_address =
                                protocol_owner_badge.resource_address(env)?;

                            Ok((
                                OlympusConfiguration {
                                    owner_role: OwnerRole::None,
                                    protocol_owner_role: rule!(require(protocol_owner_resource_address)),
                                    protocol_manager_role: rule!(require(protocol_manager_resource_address)),
                                    oracle: OracleAdapter(Reference(FAUCET.into_node_id())),
                                    usd_resource_address: XRD,
                                    address_reservation: None,
                                },
                                (
                                    protocol_manager_badge,
                                    protocol_owner_badge
                                )
                            ))
                        }
                    )?;

                    let proof = [< $role _badge >].create_proof_of_all(env)?;
                    LocalAuthZone::push(proof, env)?;

                    // Act
                    let rtn = olympus.$method_name( $( $arg ),* , env);

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
                        mut olympus,
                        additional_data: (protocol_manager_badge, protocol_owner_badge),
                        ..
                    } = Environment::new_with_olympus_config(
                        |env| {
                            let protocol_manager_badge =
                                ::scrypto_test::prelude::ResourceBuilder::new_fungible(OwnerRole::None)
                                    .divisibility(0)
                                    .mint_initial_supply(1, env)?;
                            let protocol_owner_badge =
                                ::scrypto_test::prelude::ResourceBuilder::new_fungible(OwnerRole::None)
                                    .divisibility(0)
                                    .mint_initial_supply(1, env)?;

                            let protocol_manager_resource_address =
                                protocol_manager_badge.resource_address(env)?;
                            let protocol_owner_resource_address =
                                protocol_owner_badge.resource_address(env)?;

                            Ok((
                                OlympusConfiguration {
                                    owner_role: OwnerRole::None,
                                    protocol_owner_role: rule!(require(protocol_owner_resource_address)),
                                    protocol_manager_role: rule!(require(protocol_manager_resource_address)),
                                    oracle: OracleAdapter(Reference(FAUCET.into_node_id())),
                                    usd_resource_address: XRD,
                                    address_reservation: None,
                                },
                                (
                                    protocol_manager_badge,
                                    protocol_owner_badge
                                )
                            ))
                        }
                    )?;

                    // Act
                    let rtn = olympus.$method_name( $( $arg ),* , env);

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
use test_access_rules;
