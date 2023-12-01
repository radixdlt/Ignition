//! This module contains the code used to create the various test environments
//! that are used, more specifically the [`TestRunner`] and [`TestEnvironment`].
//! More specifically, this creates those objects and flashes the exchange
//! packages to the substate store making them ready to be used. All of the
//! functions and methods in this module do unwraps and panics just because they
//! will be used in tests where we do not want to deal with their errors.

use radix_engine_interface::prelude::*;
use radix_engine_store_interface::interface::*;
use scrypto_test::prelude::*;
use scrypto_unit::*;

type PackageSubstates = HashMap<DbPartitionKey, HashMap<DbSortKey, Vec<u8>>>;

pub struct Environment<T> {
    pub environment: T,
    pub caviarnine_package: PackageAddress,
    pub ociswap_package: PackageAddress,
    pub defiplaza_package: PackageAddress,
}

pub fn new_test_environment() -> Environment<TestEnvironment> {
    // Placeholders for initializations.
    let mut caviarnine_package = PACKAGE_PACKAGE;
    let mut ociswap_package = PACKAGE_PACKAGE;
    let mut defiplaza_package = PACKAGE_PACKAGE;

    let env = TestEnvironment::new_custom(|substate_database| {
        caviarnine_package = flash(
            include_bytes!("../assets/caviarnine").as_slice(),
            substate_database,
        );
        ociswap_package = flash(
            include_bytes!("../assets/ociswap").as_slice(),
            substate_database,
        );
        defiplaza_package = flash(
            include_bytes!("../assets/defiplaza").as_slice(),
            substate_database,
        );
    });

    Environment {
        environment: env,
        caviarnine_package,
        ociswap_package,
        defiplaza_package,
    }
}

pub fn new_test_runner() -> Environment<DefaultTestRunner> {
    let mut test_runner = TestRunnerBuilder::new().build();
    let substate_database = test_runner.substate_db_mut();

    let caviarnine_package = flash(
        include_bytes!("../assets/caviarnine").as_slice(),
        substate_database,
    );
    let ociswap_package = flash(
        include_bytes!("../assets/ociswap").as_slice(),
        substate_database,
    );
    let defiplaza_package = flash(
        include_bytes!("../assets/defiplaza").as_slice(),
        substate_database,
    );

    Environment {
        environment: test_runner,
        caviarnine_package,
        ociswap_package,
        defiplaza_package,
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
    scrypto_decode(package_substates).expect("Decoding of package can not fail!")
}

fn extract_package_address(package_substates: &PackageSubstates) -> PackageAddress {
    package_substates
        .keys()
        .map(|item| {
            PackageAddress::try_from(SpreadPrefixKeyMapper::from_db_partition_key(item).0).unwrap()
        })
        .next()
        .unwrap()
}

fn database_updates(package_substates: PackageSubstates) -> DatabaseUpdates {
    let mut database_updates = DatabaseUpdates::default();

    for (partition_key, substate_values) in package_substates.into_iter() {
        for (sort_key, substate_value) in substate_values.into_iter() {
            let PartitionDatabaseUpdates::Delta { substate_updates } = database_updates
                .node_updates
                .entry(partition_key.node_key.clone())
                .or_default()
                .partition_updates
                .entry(partition_key.partition_num)
                .or_default()
            else {
                panic!("Can't happen!")
            };
            substate_updates.insert(sort_key, DatabaseUpdate::Set(substate_value));
        }
    }

    database_updates
}

#[cfg(test)]
mod tests {
    use super::*;

    const PACKAGE_SUBSTATES: [&[u8]; 3] = [
        include_bytes!("../assets/caviarnine").as_slice(),
        include_bytes!("../assets/defiplaza").as_slice(),
        include_bytes!("../assets/ociswap").as_slice(),
    ];

    #[test]
    fn packages_can_be_flashed_to_in_memory_substate_database() {
        // Arrange
        let mut substate_database = InMemorySubstateDatabase::standard();
        for substates in PACKAGE_SUBSTATES {
            // Act & Assert
            let _ = flash(substates, &mut substate_database);
        }
    }

    #[test]
    fn test_runner_can_be_created_with_flashed_packages() {
        let Environment {
            environment: _,
            caviarnine_package: _,
            ociswap_package: _,
            defiplaza_package: _,
        } = new_test_runner();
    }

    #[test]
    fn test_env_can_be_created_with_flashed_packages() {
        let Environment {
            environment: _,
            caviarnine_package: _,
            ociswap_package: _,
            defiplaza_package: _,
        } = new_test_environment();
    }
}
