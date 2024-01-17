use crate::prelude::*;

/// The environment that Ignition is tested in.
///
/// This offers a set of convince methods for creating and initializing the
/// environment such that it is in a state that can easily be tested. First,
/// the appropriate package's substates are loaded and then flashed to the
/// environment making them available for use there, these substates come from
/// the mainnet state tree. Additionally, the needed resources are created,
/// and an oracle is created and registered in Ignition.
///
/// Ignition will be initialized with a protocol owner and a protocol manager
/// whose badges will be created in the initialization of the environment and
/// returned back to the caller. Additionally, the auth module will be disabled
/// by default for the created test environment. If it needs to be enabled then
/// that must happen after the creation of the environment.  
pub struct Environment {
    /* Test Environment */
    pub environment: TestEnvironment,
    /* Various entities */
    pub resources: ResourceInformation<ResourceAddress>,
    pub protocol: ProtocolEntities,
    /* Supported Dexes */
    pub ociswap:
        DexEntities<OciswapPoolInterfaceScryptoTestStub, OciswapAdapter>,
    pub caviarnine:
        DexEntities<CaviarNinePoolInterfaceScryptoTestStub, CaviarNineAdapter>,
}

impl Environment {
    const PACKAGES_BINARY: &'static [u8] =
        include_bytes!(concat!(env!("OUT_DIR"), "/uncompressed_state.bin"));

    pub fn new() -> Result<Self, RuntimeError> {
        Self::new_with_configuration(Configuration::default())
    }

    pub fn new_with_configuration(
        configuration: Configuration,
    ) -> Result<Self, RuntimeError> {
        // Flash the substates to the ledger so that they can be used in tests.
        let (addresses, db_flash) =
            scrypto_decode::<(Vec<NodeId>, DbFlash)>(Self::PACKAGES_BINARY)
                .expect("Can't fail!");

        let caviarnine_package =
            PackageAddress::try_from(addresses[0]).unwrap();
        let ociswap_package = PackageAddress::try_from(addresses[1]).unwrap();

        let mut env = TestEnvironmentBuilder::new().flash(db_flash).build();

        // This environment instantiation function assumes that we do not want
        // to have the auth module enabled and that we are more interested in
        // just testing the behavior. So, the auth module is disabled in the
        // environment. If somebody want it, they can enable it after they
        // instantiate the environment.
        env.disable_auth_module();

        // Creating the badges and their access rules
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

        // Publishing the various packages to the testing environment
        let [ignition_package, simple_oracle_package, ociswap_adapter_v1_package, caviarnine_adapter_v1_package] =
            [
                "ignition",
                "simple-oracle",
                "ociswap-adapter-v1",
                "caviarnine-adapter-v1",
            ]
            .map(|name| Self::publish_package(name, &mut env).unwrap());

        // Creating the various resources and their associated pools.
        let divisibilities = ResourceInformation::<u8> {
            bitcoin: 8,
            ethereum: 18,
            usdc: 6,
            usdt: 6,
        };

        let resource_addresses = divisibilities.try_map(|divisibility| {
            ResourceBuilder::new_fungible(OwnerRole::Fixed(rule!(allow_all)))
                .divisibility(*divisibility)
                .mint_roles(mint_roles! {
                    minter => rule!(allow_all);
                    minter_updater => rule!(allow_all);
                })
                .burn_roles(burn_roles! {
                    burner => rule!(allow_all);
                    burner_updater => rule!(allow_all);
                })
                .mint_initial_supply(dec!(0), &mut env)
                .and_then(|bucket| bucket.resource_address(&mut env))
        })?;

        // Creating the liquidity receipt resource that each of the exchanges
        // will use.
        let [ociswap_liquidity_receipt_resource, caviarnine_liquidity_receipt_resource] =
            [(), ()].map(|_| {
                ResourceBuilder::new_ruid_non_fungible::<LiquidityReceipt>(
                    OwnerRole::None,
                )
                .mint_roles(mint_roles! {
                    minter => rule!(allow_all);
                    minter_updater => rule!(allow_all);
                })
                .burn_roles(burn_roles! {
                    burner => rule!(allow_all);
                    burner_updater => rule!(allow_all);
                })
                .mint_initial_supply([], &mut env)
                .expect("Must succeed!")
                .resource_address(&mut env)
                .expect("Must succeed!")
            });

        // Creating the Ociswap pools of the resources.
        let ociswap_pools = resource_addresses.try_map(|resource_address| {
            let mut ociswap_pool =
                OciswapPoolInterfaceScryptoTestStub::instantiate(
                    *resource_address,
                    XRD,
                    configuration.fees,
                    FAUCET,
                    ociswap_package,
                    &mut env,
                )?;

            let resource_x = ResourceManager(*resource_address)
                .mint_fungible(dec!(100_000_000), &mut env)?;
            let resource_y = ResourceManager(XRD)
                .mint_fungible(dec!(100_000_000), &mut env)?;
            let _ =
                ociswap_pool.add_liquidity(resource_x, resource_y, &mut env)?;

            Ok::<_, RuntimeError>(ociswap_pool)
        })?;

        // Creating the Caviarnine pools of the resources.
        let caviarnine_pools =
            resource_addresses.try_map(|resource_address| {
                CaviarNinePoolInterfaceScryptoTestStub::new(
                    rule!(allow_all),
                    rule!(allow_all),
                    *resource_address,
                    XRD,
                    50,
                    None,
                    caviarnine_package,
                    &mut env,
                )
            })?;

        // Instantiating the components.
        let mut simple_oracle = SimpleOracle::instantiate(
            protocol_manager_rule.clone(),
            OwnerRole::None,
            None,
            simple_oracle_package,
            &mut env,
        )?;
        let mut ignition = Ignition::instantiate(
            OwnerRole::None,
            protocol_owner_rule,
            protocol_manager_rule,
            XRD.into(),
            simple_oracle.try_into().unwrap(),
            configuration.maximum_allowed_price_staleness_seconds,
            configuration.maximum_allowed_relative_price_difference,
            None,
            ignition_package,
            &mut env,
        )?;
        let ociswap_adapter_v1 = OciswapAdapter::instantiate(
            OwnerRole::None,
            None,
            ociswap_adapter_v1_package,
            &mut env,
        )?;
        let caviarnine_adapter_v1 = CaviarNineAdapter::instantiate(
            OwnerRole::None,
            None,
            caviarnine_adapter_v1_package,
            &mut env,
        )?;

        // Submitting some dummy prices to the oracle to get things going.
        resource_addresses.try_map(|resource_address| {
            simple_oracle.set_price(*resource_address, XRD, dec!(1), &mut env)
        })?;

        // Initializing ignition with information
        {
            ignition.set_is_open_position_enabled(true, &mut env)?;
            ignition.set_is_close_position_enabled(true, &mut env)?;

            ignition.add_reward_rate(
                LockupPeriod::from_months(6),
                dec!(0.2),
                &mut env,
            )?;
            ignition.add_reward_rate(
                LockupPeriod::from_months(12),
                dec!(0.4),
                &mut env,
            )?;

            let xrd_bucket = ResourceManager(XRD)
                .mint_fungible(dec!(100_000_000_000_000), &mut env)?;
            ignition.deposit_resources(FungibleBucket(xrd_bucket), &mut env)?;

            ignition.insert_pool_information(
                OciswapPoolInterfaceScryptoTestStub::blueprint_id(
                    ociswap_package,
                ),
                PoolBlueprintInformation {
                    adapter: ociswap_adapter_v1.into(),
                    allowed_pools: ociswap_pools
                        .iter()
                        .map(|pool| pool.try_into().unwrap())
                        .collect(),
                    liquidity_receipt: ociswap_liquidity_receipt_resource
                        .into(),
                },
                &mut env,
            )?;

            ignition.insert_pool_information(
                CaviarNinePoolInterfaceScryptoTestStub::blueprint_id(
                    caviarnine_package,
                ),
                PoolBlueprintInformation {
                    adapter: caviarnine_adapter_v1.into(),
                    allowed_pools: caviarnine_pools
                        .iter()
                        .map(|pool| pool.try_into().unwrap())
                        .collect(),
                    liquidity_receipt: caviarnine_liquidity_receipt_resource
                        .into(),
                },
                &mut env,
            )?;
        }

        Ok(Self {
            environment: env,
            resources: resource_addresses,
            protocol: ProtocolEntities {
                ignition_package_address: ignition_package,
                ignition,
                oracle_package_address: simple_oracle_package,
                oracle: simple_oracle,
                protocol_owner_badge,
                protocol_manager_badge,
            },
            ociswap: DexEntities {
                package: ociswap_package,
                pools: ociswap_pools,
                adapter_package: ociswap_adapter_v1_package,
                adapter: ociswap_adapter_v1,
                liquidity_receipt: ociswap_liquidity_receipt_resource,
            },
            caviarnine: DexEntities {
                package: caviarnine_package,
                pools: caviarnine_pools,
                adapter_package: caviarnine_adapter_v1_package,
                adapter: caviarnine_adapter_v1,
                liquidity_receipt: caviarnine_liquidity_receipt_resource,
            },
        })
    }

    fn publish_package(
        name: &str,
        env: &mut TestEnvironment,
    ) -> Result<PackageAddress, RuntimeError> {
        let (code, definition) = package_loader::PackageLoader::get(name);
        Package::publish(code, definition, Default::default(), env)
            .map(|item| item.0)
    }
}

#[derive(Debug)]
pub struct ProtocolEntities {
    /* Ignition */
    pub ignition_package_address: PackageAddress,
    pub ignition: Ignition,
    /* Oracle */
    pub oracle_package_address: PackageAddress,
    pub oracle: SimpleOracle,
    /* Badges */
    pub protocol_owner_badge: Bucket,
    pub protocol_manager_badge: Bucket,
}

/// A struct that defines the entities that belong to a Decentralized Exchange.
/// it contains the package address as well as generic items [`T`] which are
/// the stubs used to call the pools.
#[derive(Clone, Debug)]
pub struct DexEntities<P, A> {
    /* Packages */
    pub package: PackageAddress,
    /* Pools */
    pub pools: ResourceInformation<P>,
    /* Adapter */
    pub adapter_package: PackageAddress,
    pub adapter: A,
    /* Receipt */
    pub liquidity_receipt: ResourceAddress,
}

#[derive(Clone, Debug, Copy)]
pub struct ResourceInformation<T> {
    pub bitcoin: T,
    pub ethereum: T,
    pub usdc: T,
    pub usdt: T,
}

impl<T> ResourceInformation<T> {
    pub fn map<F, O>(&self, map: F) -> ResourceInformation<O>
    where
        F: Fn(&T) -> O,
    {
        ResourceInformation::<O> {
            bitcoin: map(&self.bitcoin),
            ethereum: map(&self.ethereum),
            usdc: map(&self.usdc),
            usdt: map(&self.usdt),
        }
    }

    pub fn try_map<F, O, E>(
        &self,
        mut map: F,
    ) -> Result<ResourceInformation<O>, E>
    where
        F: FnMut(&T) -> Result<O, E>,
    {
        Ok(ResourceInformation::<O> {
            bitcoin: map(&self.bitcoin)?,
            ethereum: map(&self.ethereum)?,
            usdc: map(&self.usdc)?,
            usdt: map(&self.usdt)?,
        })
    }

    pub fn iter(self) -> impl Iterator<Item = T> {
        [self.bitcoin, self.ethereum, self.usdc, self.usdt].into_iter()
    }
}

#[derive(Clone, Debug)]
pub struct Configuration {
    pub fees: Decimal,
    pub maximum_allowed_price_staleness_seconds: i64,
    pub maximum_allowed_relative_price_difference: Decimal,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            // 1%
            fees: dec!(0.01),
            // 5 Minutes
            maximum_allowed_price_staleness_seconds: 300i64,
            // 1%
            maximum_allowed_relative_price_difference: dec!(0.01),
        }
    }
}
