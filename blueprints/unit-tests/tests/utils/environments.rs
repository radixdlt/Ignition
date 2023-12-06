//! This module contains the code used to create the various test environments
//! that are used, more specifically the [`TestRunner`] and [`TestEnvironment`].
//! More specifically, this creates those objects and flashes the exchange
//! packages to the substate store making them ready to be used. All of the
//! functions and methods in this module do unwraps and panics just because they
//! will be used in tests where we do not want to deal with their errors.

#![allow(dead_code)]

use radix_engine_interface::prelude::*;
use radix_engine_store_interface::interface::*;
use scrypto::prelude::{RoleDefinition, ToRoleEntry};
use scrypto_test::prelude::*;
use scrypto_unit::*;

type PackageSubstates = HashMap<DbPartitionKey, HashMap<DbSortKey, Vec<u8>>>;

pub struct Environment<T> {
    pub environment: T,
    pub packages: Packages,
    pub resources: Resources,
}

pub struct Packages {
    pub caviarnine_package: PackageAddress,
    pub ociswap_package: PackageAddress,
    pub defiplaza_package: PackageAddress,
}

pub struct Resources {
    pub bitcoin: ResourceAddress,
    pub ethereum: ResourceAddress,
    pub usdc: ResourceAddress,
    pub usdt: ResourceAddress,
}

pub fn new_test_environment() -> Environment<TestEnvironment> {
    // Placeholders for initializations.
    let mut caviarnine_package = PACKAGE_PACKAGE;
    let mut ociswap_package = PACKAGE_PACKAGE;
    let mut defiplaza_package = PACKAGE_PACKAGE;

    let mut env = TestEnvironment::new_custom(|substate_database| {
        caviarnine_package = flash(
            include_bytes!("../../assets/caviarnine").as_slice(),
            substate_database,
        );
        ociswap_package = flash(
            include_bytes!("../../assets/ociswap").as_slice(),
            substate_database,
        );
        defiplaza_package = flash(
            include_bytes!("../../assets/defiplaza").as_slice(),
            substate_database,
        );
    });

    // Creating the resources. They are all freely mintable to make the tests
    // easier.
    let [bitcoin, ethereum, usdc, usdt] = [8, 18, 6, 6].map(|divisibility| {
        ResourceBuilder::new_fungible(OwnerRole::Fixed(rule!(allow_all)))
            .divisibility(divisibility)
            .mint_roles(mint_roles! {
                minter => rule!(allow_all);
                minter_updater => rule!(allow_all);
            })
            .burn_roles(burn_roles! {
                burner => rule!(allow_all);
                burner_updater => rule!(allow_all);
            })
            .mint_initial_supply(dec!(1), &mut env)
            .expect("Can't fail to create resource!")
            .resource_address(&mut env)
            .expect("Can't fail to create resource!")
    });

    Environment {
        environment: env,
        packages: Packages {
            caviarnine_package,
            ociswap_package,
            defiplaza_package,
        },
        resources: Resources {
            bitcoin,
            ethereum,
            usdc,
            usdt,
        },
    }
}

pub fn new_test_runner() -> Environment<DefaultTestRunner> {
    let mut test_runner = TestRunnerBuilder::new().build();
    let substate_database = test_runner.substate_db_mut();

    let caviarnine_package = flash(
        include_bytes!("../../assets/caviarnine").as_slice(),
        substate_database,
    );
    let ociswap_package = flash(
        include_bytes!("../../assets/ociswap").as_slice(),
        substate_database,
    );
    let defiplaza_package = flash(
        include_bytes!("../../assets/defiplaza").as_slice(),
        substate_database,
    );

    let [bitcoin, ethereum, usdc, usdt] = [8, 18, 6, 6].map(|divisibility| {
        let manifest = ManifestBuilder::new()
            .create_fungible_resource(
                OwnerRole::Fixed(rule!(allow_all)),
                true,
                divisibility,
                FungibleResourceRoles {
                    mint_roles: mint_roles! {
                        minter => rule!(allow_all);
                        minter_updater => rule!(allow_all);
                    },
                    burn_roles: burn_roles! {
                        burner => rule!(allow_all);
                        burner_updater => rule!(allow_all);
                    },
                    freeze_roles: None,
                    recall_roles: None,
                    withdraw_roles: None,
                    deposit_roles: None,
                },
                Default::default(),
                None,
            )
            .build();
        let receipt = test_runner.execute_manifest(manifest, vec![]);
        *receipt
            .expect_commit_success()
            .new_resource_addresses()
            .first()
            .unwrap()
    });

    Environment {
        environment: test_runner,
        packages: Packages {
            caviarnine_package,
            ociswap_package,
            defiplaza_package,
        },
        resources: Resources {
            bitcoin,
            ethereum,
            usdc,
            usdt,
        },
    }
}

fn flash<S: CommittableSubstateDatabase>(
    package_substates: &[u8],
    substate_database: &mut S,
) -> PackageAddress {
    let package_substates = decode_package_substates(package_substates);
    let package_address = extract_package_address(&package_substates);
    let database_updates = database_updates(package_substates);
    substate_database.commit(&database_updates);
    package_address
}

fn decode_package_substates(
    package_substates: &[u8],
) -> HashMap<DbPartitionKey, HashMap<DbSortKey, Vec<u8>>> {
    scrypto_decode(package_substates)
        .expect("Decoding of package can not fail!")
}

fn extract_package_address(
    package_substates: &PackageSubstates,
) -> PackageAddress {
    package_substates
        .keys()
        .map(|item| {
            PackageAddress::try_from(
                SpreadPrefixKeyMapper::from_db_partition_key(item).0,
            )
            .unwrap()
        })
        .next()
        .unwrap()
}

fn database_updates(package_substates: PackageSubstates) -> DatabaseUpdates {
    let mut database_updates = DatabaseUpdates::default();

    for (partition_key, substate_values) in package_substates.into_iter() {
        for (sort_key, substate_value) in substate_values.into_iter() {
            let PartitionDatabaseUpdates::Delta { substate_updates } =
                database_updates
                    .node_updates
                    .entry(partition_key.node_key.clone())
                    .or_default()
                    .partition_updates
                    .entry(partition_key.partition_num)
                    .or_default()
            else {
                panic!("Can't happen!")
            };
            substate_updates
                .insert(sort_key, DatabaseUpdate::Set(substate_value));
        }
    }

    database_updates
}
