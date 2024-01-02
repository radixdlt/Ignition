//! This module contains the code used to create the various test environments
//! that are used, more specifically the [`TestRunner`] and [`TestEnvironment`].
//! More specifically, this creates those objects and flashes the exchange
//! packages to the substate store making them ready to be used. All of the
//! functions and methods in this module do unwraps and panics just because they
//! will be used in tests where we do not want to deal with their errors.

// TODO: It would be great to have an Environment builder that leverages the
// type system to provide an environment based on what's specified during the
// building process.

#![allow(dead_code)]

use radix_engine_interface::prelude::*;
use radix_engine_store_interface::interface::*;
use scrypto::prelude::{RoleDefinition, ToRoleEntry};
use scrypto_test::prelude::*;

use adapters_interface::oracle::*;
use ociswap_adapter::test_bindings::*;
use olympus::test_bindings::*;
use test_oracle::test_bindings::*;

const PACKAGES_BINARY: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/uncompressed_state.bin"));

type PackageSubstates = HashMap<DbPartitionKey, HashMap<DbSortKey, Vec<u8>>>;

pub struct Environment<T> {
    pub environment: TestEnvironment,
    pub packages: Packages,
    pub resources: Resources,
    pub components: Components,
    pub additional_data: T,
}

pub struct Components {
    /* Core */
    pub olympus: Olympus,
    /* Adapters */
    pub ociswap_adapter: OciswapAdapter,
    /* Test Components */
    pub test_oracle: TestOracle,
}

pub struct Packages {
    /* DEXs */
    pub caviarnine_package: PackageAddress,
    pub ociswap_package: PackageAddress,
    pub defiplaza_package: PackageAddress,
    /* Adapters */
    pub ociswap_adapter_package: PackageAddress,
    /* Test Packages */
    pub test_oracle_package: PackageAddress,
}

pub struct Resources {
    pub bitcoin: ResourceManager,
    pub ethereum: ResourceManager,
    pub usdc: ResourceManager,
    pub usdt: ResourceManager,
}

impl Environment<()> {
    pub fn new() -> Result<Environment<()>, RuntimeError> {
        Self::new_with_olympus_config(|_| {
            Ok((
                OlympusConfiguration {
                    owner_role: OwnerRole::None,
                    protocol_owner_role: rule!(allow_all),
                    protocol_manager_role: rule!(allow_all),
                    oracle: OracleAdapterInterfaceScryptoTestStub::from(FAUCET),
                    usd_resource_address: XRD,
                    address_reservation: None,
                },
                (),
            ))
        })
    }
}

impl Environment<OlympusBadges> {
    pub fn new_create_badges() -> Result<Self, RuntimeError> {
        Environment::new_with_olympus_config(|env| {
            let protocol_manager_badge =
                ::scrypto_test::prelude::ResourceBuilder::new_fungible(
                    OwnerRole::None,
                )
                .divisibility(0)
                .mint_initial_supply(1, env)?;
            let protocol_owner_badge =
                ::scrypto_test::prelude::ResourceBuilder::new_fungible(
                    OwnerRole::None,
                )
                .divisibility(0)
                .mint_initial_supply(1, env)?;

            let protocol_manager_resource_address =
                protocol_manager_badge.resource_address(env)?;
            let protocol_owner_resource_address =
                protocol_owner_badge.resource_address(env)?;

            Ok((
                OlympusConfiguration {
                    owner_role: OwnerRole::None,
                    protocol_owner_role: rule!(require(
                        protocol_owner_resource_address
                    )),
                    protocol_manager_role: rule!(require(
                        protocol_manager_resource_address
                    )),
                    oracle: OracleAdapterInterfaceScryptoTestStub::from(FAUCET),
                    usd_resource_address: XRD,
                    address_reservation: None,
                },
                OlympusBadges {
                    protocol_manager: protocol_manager_badge,
                    protocol_owner: protocol_owner_badge,
                },
            ))
        })
    }
}

impl<T> Environment<T> {
    pub fn new_with_olympus_config<F>(callback: F) -> Result<Self, RuntimeError>
    where
        F: Fn(
            &mut TestEnvironment,
        ) -> Result<(OlympusConfiguration, T), RuntimeError>,
    {
        let (addresses, db_flash) =
            scrypto_decode::<(Vec<NodeId>, DbFlash)>(PACKAGES_BINARY)
                .expect("Can't fail!");

        let caviarnine_package =
            PackageAddress::try_from(addresses[0]).unwrap();
        let ociswap_package = PackageAddress::try_from(addresses[1]).unwrap();
        let defiplaza_package = PackageAddress::try_from(addresses[2]).unwrap();

        let mut env = TestEnvironmentBuilder::new().flash(db_flash).build();

        // Creating the resources. They are all freely mintable to make the tests
        // easier.
        let [bitcoin, ethereum, usdc, usdt] =
            [8, 18, 6, 6].map(|divisibility| {
                ResourceBuilder::new_fungible(OwnerRole::Fixed(rule!(
                    allow_all
                )))
                .divisibility(divisibility)
                .mint_roles(mint_roles! {
                    minter => rule!(allow_all);
                    minter_updater => rule!(allow_all);
                })
                .burn_roles(burn_roles! {
                    burner => rule!(allow_all);
                    burner_updater => rule!(allow_all);
                })
                .mint_initial_supply(dec!(0), &mut env)
                .expect("Can't fail to create resource!")
                .resource_address(&mut env)
                .expect("Can't fail to create resource!")
            });

        // Get the configuration to use for the Olympus component from the
        // callback
        let (configuration, additional_data) = callback(&mut env)?;

        // Publishing the Olympus package and instantiating an Olympus component
        let (code, definition) =
            super::package_loader::PackageLoader::get("olympus");
        let (olympus_package_address, _) =
            Package::publish(code, definition, Default::default(), &mut env)
                .unwrap();

        let olympus = Olympus::instantiate(
            configuration.owner_role,
            configuration.protocol_owner_role,
            configuration.protocol_manager_role.clone(),
            configuration.oracle.try_into().unwrap(),
            configuration.usd_resource_address,
            configuration.address_reservation,
            olympus_package_address,
            &mut env,
        )?;

        let (code, definition) =
            super::package_loader::PackageLoader::get("ociswap-adapter");
        let (ociswap_adapter_package_address, _) =
            Package::publish(code, definition, Default::default(), &mut env)
                .unwrap();

        let ociswap_adapter = OciswapAdapter::instantiate(
            OwnerRole::None,
            None,
            ociswap_adapter_package_address,
            &mut env,
        )?;

        let (code, definition) =
            super::package_loader::PackageLoader::get("test-oracle");
        let (test_oracle_package_address, _) =
            Package::publish(code, definition, Default::default(), &mut env)
                .unwrap();

        let test_oracle = TestOracle::instantiate(
            configuration.protocol_manager_role.clone(),
            OwnerRole::None,
            None,
            test_oracle_package_address,
            &mut env,
        )?;

        Ok(Environment {
            environment: env,
            packages: Packages {
                /* DEXs */
                caviarnine_package,
                ociswap_package,
                defiplaza_package,
                /* Adapters */
                ociswap_adapter_package: ociswap_adapter_package_address,
                /* Test Packages */
                test_oracle_package: test_oracle_package_address,
            },
            resources: Resources {
                bitcoin: ResourceManager(bitcoin),
                ethereum: ResourceManager(ethereum),
                usdc: ResourceManager(usdc),
                usdt: ResourceManager(usdt),
            },
            components: Components {
                /* Core */
                olympus,
                /* Adapters */
                ociswap_adapter,
                /* Test Components */
                test_oracle,
            },
            additional_data,
        })
    }
}

#[derive(Clone, Debug)]
pub struct OlympusConfiguration {
    pub owner_role: OwnerRole,
    pub protocol_owner_role: AccessRule,
    pub protocol_manager_role: AccessRule,
    pub oracle: OracleAdapterInterfaceScryptoTestStub,
    pub usd_resource_address: ResourceAddress,
    pub address_reservation: Option<GlobalAddressReservation>,
}

#[derive(Debug)]
pub struct OlympusBadges {
    pub protocol_owner: Bucket,
    pub protocol_manager: Bucket,
}
