//! A module that defines the environment that the tests run in.

use radix_engine_interface::prelude::*;
use scrypto::prelude::{RoleDefinition, ToRoleEntry};
use scrypto_test::prelude::*;

use adapters_interface::common::*;
use ociswap_adapter::test_bindings::*;
use ociswap_adapter::*;
use olympus::test_bindings::*;
use olympus::LockupPeriod;
use test_oracle::test_bindings::*;

/// The environment that Olympus is tested in.
///
/// This offers a set of convince methods for creating and initializing the
/// environment such that it is in a state that can easily be tested. First,
/// the appropriate package's substates are loaded and then flashed to the
/// environment making them available for use there, these substates come from
/// the mainnet state tree. Additionally, the needed resources are created,
/// and an oracle is created and registered in Olympus.
///
/// Olympus will be initialized with a protocol owner and a protocol manager
/// whose badges will be created in the initialization of the environment and
/// returned back to the caller. Additionally, the auth module will be disabled
/// by default for the created test environment. If it needs to be enabled then
/// that must happen after the creation of the environment.  
pub struct Environment {
    pub environment: TestEnvironment,
    pub resources: Resources,
    pub protocol: ProtocolEntities,
    pub ociswap:
        DexEntities<OciswapPoolInterfaceScryptoTestStub, OciswapAdapter>,
}

impl Environment {
    const PACKAGES_BINARY: &'static [u8] =
        include_bytes!(concat!(env!("OUT_DIR"), "/uncompressed_state.bin"));

    pub fn new() -> Result<Self, RuntimeError> {
        // Preparing the substates that will be flashed to the environment. Load
        // and decode them.
        let (addresses, db_flash) =
            scrypto_decode::<(Vec<NodeId>, DbFlash)>(Self::PACKAGES_BINARY)
                .expect("Can't fail!");

        // Getting the package addresses from the provided vector of NodeIds.
        // The structure of this is known at compile-time based on the order of
        // export of the branch from the state tree. As in, if we dump branch of
        // package a, then package b, and then package c through ledger-tools,
        // then they will be in that order.
        let _caviarnine_package =
            PackageAddress::try_from(addresses[0]).unwrap();
        let ociswap_package = PackageAddress::try_from(addresses[1]).unwrap();
        let _defiplaza_package =
            PackageAddress::try_from(addresses[2]).unwrap();

        // Creating the environment and flashing the substates to the database.
        let mut env = TestEnvironmentBuilder::new().flash(db_flash).build();

        // This environment instantiation function assumes that we do not want
        // to have the auth module enabled and that we are more interested in
        // just testing the behavior. So, the auth module is disabled in the
        // environment. If somebody want it, they can enable it after they
        // instantiate the environment.
        env.disable_auth_module();

        /* Package publishing */
        let (olympus_package_address, _) =
            Self::publish_package("olympus", &mut env)?;
        let (ociswap_adapter_package_address, _) =
            Self::publish_package("ociswap-adapter", &mut env)?;
        let (test_oracle_package_address, _) =
            Self::publish_package("test-oracle", &mut env)?;

        /* Resource Creation */
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
        let protocol_manager_badge =
            ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(0)
                .mint_initial_supply(1, &mut env)?;
        let protocol_owner_badge =
            ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(0)
                .mint_initial_supply(1, &mut env)?;

        let protocol_manager_rule = protocol_manager_badge
            .resource_address(&mut env)
            .map(|address| rule!(require(address)))?;
        let protocol_owner_rule = protocol_owner_badge
            .resource_address(&mut env)
            .map(|address| rule!(require(address)))?;

        /* Component Instantiation */
        let test_oracle = TestOracle::instantiate(
            protocol_manager_rule.clone(),
            OwnerRole::None,
            None,
            test_oracle_package_address,
            &mut env,
        )?;
        let ociswap_adapter = OciswapAdapter::instantiate(
            OwnerRole::None,
            None,
            ociswap_adapter_package_address,
            &mut env,
        )?;
        let mut olympus = Olympus::instantiate(
            OwnerRole::None,
            protocol_owner_rule,
            protocol_manager_rule,
            test_oracle.try_into().unwrap(),
            usdt,
            None,
            olympus_package_address,
            &mut env,
        )?;

        /* Pool Creation */
        let [ociswap_bitcoin_pool, ociswap_ethereum_pool, ociswap_usdc_pool, ociswap_usdt_pool] =
            [bitcoin, ethereum, usdc, usdt].map(|resource_address| {
                OciswapPoolInterfaceScryptoTestStub::instantiate(
                    resource_address,
                    XRD,
                    dec!(0.01),
                    FAUCET,
                    ociswap_package,
                    &mut env,
                )
                .unwrap()
            });

        /* Olympus configuration */
        // Allow liquidity positions to be opened and closed.
        olympus.config_open_liquidity_position(true, &mut env)?;
        olympus.config_close_liquidity_position(true, &mut env)?;
        // Provide Olympus with its first funding of XRD.
        olympus.deposit(
            FungibleBucket(
                ResourceManager(XRD)
                    .mint_fungible(dec!(100_000_000_000), &mut env)?,
            ),
            &mut env,
        )?;
        // Add the various pool adapters to Olympus
        olympus.add_pool_adapter(
            OciswapPoolInterfaceScryptoTestStub::blueprint_id(ociswap_package),
            ociswap_adapter.try_into().unwrap(),
            &mut env,
        )?;
        // Register the various pools with Olympus
        for allowed_pool in [
            ComponentAddress::try_from(ociswap_bitcoin_pool).unwrap(),
            ComponentAddress::try_from(ociswap_ethereum_pool).unwrap(),
            ComponentAddress::try_from(ociswap_usdc_pool).unwrap(),
            ComponentAddress::try_from(ociswap_usdt_pool).unwrap(),
        ] {
            olympus.add_allowed_pool(allowed_pool, &mut env)?;
        }
        // Add the lockup periods that the protocol will be using. We will add
        // 6 months at 10% and 9 months at 20%.
        olympus.add_rewards_rate(
            LockupPeriod::from_months(6),
            Percent::new(dec!(0.1)).expect("Must succeed!"),
            &mut env,
        )?;
        olympus.add_rewards_rate(
            LockupPeriod::from_months(9),
            Percent::new(dec!(0.2)).expect("Must succeed!"),
            &mut env,
        )?;

        // At this point, everything needed has been created. We can now start
        // to assemble everything together into the test environment.
        Ok(Environment {
            environment: env,
            protocol: ProtocolEntities {
                olympus_package_address,
                olympus,
                oracle_package_address: test_oracle_package_address,
                oracle: test_oracle,
                protocol_owner_badge,
                protocol_manager_badge,
            },
            resources: Resources {
                bitcoin: ResourceManager(bitcoin),
                ethereum: ResourceManager(ethereum),
                usdc: ResourceManager(usdc),
                usdt: ResourceManager(usdt),
            },
            ociswap: DexEntities {
                package: ociswap_package,
                bitcoin_pool: ociswap_bitcoin_pool,
                etherem_pool: ociswap_ethereum_pool,
                usdc_pool: ociswap_usdc_pool,
                usdt_pool: ociswap_usdt_pool,
                adapter_package: ociswap_adapter_package_address,
                adapter: ociswap_adapter,
            },
        })
    }

    fn publish_package(
        name: &str,
        env: &mut TestEnvironment,
    ) -> Result<(PackageAddress, Bucket), RuntimeError> {
        let (code, definition) =
            super::package_loader::PackageLoader::get(name);
        Package::publish(code, definition, Default::default(), env)
    }
}

#[derive(Debug)]
pub struct ProtocolEntities {
    /* Olympus */
    pub olympus_package_address: PackageAddress,
    pub olympus: Olympus,
    /* Oracle */
    pub oracle_package_address: PackageAddress,
    pub oracle: TestOracle,
    /* Badges */
    pub protocol_owner_badge: Bucket,
    pub protocol_manager_badge: Bucket,
}

#[derive(Clone, Debug)]
pub struct Resources {
    pub bitcoin: ResourceManager,
    pub ethereum: ResourceManager,
    pub usdc: ResourceManager,
    pub usdt: ResourceManager,
}

/// A struct that defines the entities that belong to a Decentralized Exchange.
/// it contains the package address as well as generic items [`T`] which are
/// the stubs used to call the pools.
#[derive(Clone, Debug)]
pub struct DexEntities<P, A> {
    /* Packages */
    pub package: PackageAddress,
    /* Pools */
    pub bitcoin_pool: P,
    pub etherem_pool: P,
    pub usdc_pool: P,
    pub usdt_pool: P,
    /* Adapter */
    pub adapter_package: PackageAddress,
    pub adapter: A,
}
