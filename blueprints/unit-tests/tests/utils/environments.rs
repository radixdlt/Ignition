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

type BranchStore =
    HashMap<DbNodeKey, HashMap<DbPartitionNum, HashMap<DbSortKey, Vec<u8>>>>;

const PACKAGES_BINARY: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/uncompressed_state.bin"));

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
    let (addresses, branch_store) =
        scrypto_decode::<(Vec<NodeId>, BranchStore)>(PACKAGES_BINARY)
            .expect("Can't fail!");

    let caviarnine_package = PackageAddress::try_from(addresses[0]).unwrap();
    let ociswap_package = PackageAddress::try_from(addresses[1]).unwrap();
    let defiplaza_package = PackageAddress::try_from(addresses[2]).unwrap();

    let mut env = TestEnvironment::new_custom(|substate_database| {
        flash_branch_store(branch_store, substate_database)
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

    let substates =
        scrypto_decode::<(Vec<PackageAddress>, BranchStore)>(PACKAGES_BINARY)
            .expect("Can't fail!");

    let caviarnine_package = substates.0[0];
    let ociswap_package = substates.0[1];
    let defiplaza_package = substates.0[2];

    flash_branch_store(substates.1, substate_database);

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

fn flash_branch_store<S: CommittableSubstateDatabase>(
    branch_store: BranchStore,
    substate_database: &mut S,
) {
    let database_updates = database_updates(branch_store);
    substate_database.commit(&database_updates);
}

fn database_updates(branch_store: BranchStore) -> DatabaseUpdates {
    DatabaseUpdates {
        node_updates: branch_store
            .into_iter()
            .map(|(db_node_key, partition_num_to_updates_mapping)| {
                (
                    db_node_key,
                    NodeDatabaseUpdates {
                        partition_updates: partition_num_to_updates_mapping
                            .into_iter()
                            .map(|(partition_num, substates)| {
                                (
                                    partition_num,
                                    PartitionDatabaseUpdates::Delta {
                                        substate_updates: substates
                                            .into_iter()
                                            .map(|(db_sort_key, value)| {
                                                (
                                                    db_sort_key,
                                                    DatabaseUpdate::Set(value),
                                                )
                                            })
                                            .collect(),
                                    },
                                )
                            })
                            .collect(),
                    },
                )
            })
            .collect(),
    }
}
