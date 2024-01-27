use crate::prelude::*;

pub type ScryptoTestEnv = Environment<ScryptoTestEnvironmentSpecifier>;
pub type ScryptoUnitEnv = Environment<ScryptoUnitEnvironmentSpecifier>;

pub trait EnvironmentSpecifier {
    // Environment
    type Environment;

    // Components
    type Ignition;
    type SimpleOracle;
    type OciswapPool;
    type CaviarninePool;

    type OciswapAdapter;
    type CaviarnineAdapter;

    // Badges
    type Badge;
}

pub struct ScryptoTestEnvironmentSpecifier;

impl EnvironmentSpecifier for ScryptoTestEnvironmentSpecifier {
    // Environment
    type Environment = TestEnvironment<InMemorySubstateDatabase>;

    // Components
    type Ignition = Ignition;
    type SimpleOracle = SimpleOracle;
    type OciswapPool = OciswapPoolInterfaceScryptoTestStub;
    type CaviarninePool = CaviarNinePoolInterfaceScryptoTestStub;

    type OciswapAdapter = OciswapAdapter;
    type CaviarnineAdapter = CaviarNineAdapter;

    // Badges
    type Badge = Bucket;
}

pub struct ScryptoUnitEnvironmentSpecifier;

impl EnvironmentSpecifier for ScryptoUnitEnvironmentSpecifier {
    // Environment
    type Environment = DefaultTestRunner;

    // Components
    type Ignition = ComponentAddress;
    type SimpleOracle = ComponentAddress;
    type OciswapPool = ComponentAddress;
    type CaviarninePool = ComponentAddress;

    type OciswapAdapter = ComponentAddress;
    type CaviarnineAdapter = ComponentAddress;

    // Badges
    type Badge = (PublicKey, ComponentAddress, ResourceAddress);
}

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
pub struct Environment<S>
where
    S: EnvironmentSpecifier,
{
    /* Test Environment */
    pub environment: S::Environment,
    /* Various entities */
    pub resources: ResourceInformation<ResourceAddress>,
    pub protocol: ProtocolEntities<S>,
    /* Supported Dexes */
    pub ociswap: DexEntities<S::OciswapPool, S::OciswapAdapter>,
    pub caviarnine: DexEntities<S::CaviarninePool, S::CaviarnineAdapter>,
}

impl<S> Environment<S>
where
    S: EnvironmentSpecifier,
{
    const PACKAGES_BINARY: &'static [u8] =
        include_bytes!(concat!(env!("OUT_DIR"), "/uncompressed_state.bin"));

    const PACKAGE_NAMES: [&'static str; 4] = [
        "ignition",
        "simple-oracle",
        "ociswap-adapter-v1",
        "caviarnine-adapter-v1",
    ];

    const RESOURCE_DIVISIBILITIES: ResourceInformation<u8> =
        ResourceInformation::<u8> {
            bitcoin: 8,
            ethereum: 18,
            usdc: 6,
            usdt: 6,
        };
}

impl ScryptoTestEnv {
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
            Self::PACKAGE_NAMES
                .map(|name| Self::publish_package(name, &mut env).unwrap());

        // Creating the various resources and their associated pools.
        let resource_addresses =
            Self::RESOURCE_DIVISIBILITIES.try_map(|divisibility| {
                ResourceBuilder::new_fungible(OwnerRole::Fixed(rule!(
                    allow_all
                )))
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
                let mut caviarnine_pool =
                    CaviarNinePoolInterfaceScryptoTestStub::new(
                        rule!(allow_all),
                        rule!(allow_all),
                        *resource_address,
                        XRD,
                        50,
                        None,
                        caviarnine_package,
                        &mut env,
                    )?;

                let resource_x = ResourceManager(*resource_address)
                    .mint_fungible(dec!(2_000_000_000), &mut env)?;
                let resource_y = ResourceManager(XRD)
                    .mint_fungible(dec!(2_000_000_000), &mut env)?;
                let _ = caviarnine_pool.add_liquidity(
                    resource_x,
                    resource_y,
                    (0..10)
                        .flat_map(|offset| {
                            [
                                (
                                    27000 - offset * 50,
                                    dec!(100_000_000),
                                    dec!(100_000_000),
                                ),
                                (
                                    27000 + offset * 50,
                                    dec!(100_000_000),
                                    dec!(100_000_000),
                                ),
                            ]
                        })
                        .chain([(27000, dec!(100_000_000), dec!(100_000_000))])
                        .rev()
                        .collect(),
                    &mut env,
                )?;

                Ok::<_, RuntimeError>(caviarnine_pool)
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
            InitializationParameters::default(),
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
            ignition.deposit_protocol_resources(
                FungibleBucket(xrd_bucket),
                Volatility::Volatile,
                &mut env,
            )?;
            let xrd_bucket = ResourceManager(XRD)
                .mint_fungible(dec!(100_000_000_000_000), &mut env)?;
            ignition.deposit_protocol_resources(
                FungibleBucket(xrd_bucket),
                Volatility::NonVolatile,
                &mut env,
            )?;

            {
                let ResourceInformation {
                    bitcoin,
                    ethereum,
                    usdc,
                    usdt,
                } = resource_addresses;
                ignition.insert_user_resource_volatility(
                    bitcoin,
                    Volatility::Volatile,
                    &mut env,
                )?;
                ignition.insert_user_resource_volatility(
                    ethereum,
                    Volatility::Volatile,
                    &mut env,
                )?;

                ignition.insert_user_resource_volatility(
                    usdc,
                    Volatility::NonVolatile,
                    &mut env,
                )?;
                ignition.insert_user_resource_volatility(
                    usdt,
                    Volatility::NonVolatile,
                    &mut env,
                )?;
            }

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
                    liquidity_receipt: ociswap_liquidity_receipt_resource,
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
                    liquidity_receipt: caviarnine_liquidity_receipt_resource,
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
        env: &mut TestEnvironment<InMemorySubstateDatabase>,
    ) -> Result<PackageAddress, RuntimeError> {
        let (code, definition) = package_loader::PackageLoader::get(name);
        Package::publish(code, definition, Default::default(), env)
            .map(|item| item.0)
    }
}

impl ScryptoUnitEnv {
    pub fn new() -> Self {
        Self::new_with_configuration(Configuration::default())
    }

    pub fn new_with_configuration(configuration: Configuration) -> Self {
        // Flash the substates to the ledger so that they can be used in tests.
        let (addresses, db_flash) =
            scrypto_decode::<(Vec<NodeId>, DbFlash)>(Self::PACKAGES_BINARY)
                .expect("Can't fail!");

        let caviarnine_package =
            PackageAddress::try_from(addresses[0]).unwrap();
        let ociswap_package = PackageAddress::try_from(addresses[1]).unwrap();

        let mut test_runner = {
            let mut in_memory_substate_database =
                InMemorySubstateDatabase::standard();
            in_memory_substate_database.commit(&DatabaseUpdates {
                node_updates: db_flash
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
                                                        (db_sort_key, DatabaseUpdate::Set(value))
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
            });
            TestRunnerBuilder::new()
                .with_custom_database(in_memory_substate_database)
                .without_trace()
                .build()
        };

        // The account that everything gets deposited into throughout the tests.
        let (public_key, _, account) = test_runner.new_account(false);

        let protocol_manager_badge =
            test_runner.create_fungible_resource(dec!(1), 0, account);
        let protocol_owner_badge =
            test_runner.create_fungible_resource(dec!(1), 0, account);

        let protocol_manager_rule = rule!(require(protocol_manager_badge));
        let protocol_owner_rule = rule!(require(protocol_owner_badge));

        let [ignition_package, simple_oracle_package, ociswap_adapter_v1_package, caviarnine_adapter_v1_package] =
            Self::PACKAGE_NAMES.map(|package_name| {
                let (code, definition) =
                    package_loader::PackageLoader::get(package_name);
                test_runner.publish_package(
                    (code, definition),
                    Default::default(),
                    OwnerRole::None,
                )
            });

        let resource_addresses =
            Self::RESOURCE_DIVISIBILITIES.map(|divisibility| {
                test_runner.create_freely_mintable_fungible_resource(
                    OwnerRole::None,
                    None,
                    *divisibility,
                    account,
                )
            });

        let [ociswap_liquidity_receipt_resource, caviarnine_liquidity_receipt_resource] =
            [(), ()].map(|_| {
                test_runner
                    .create_freely_mintable_and_burnable_non_fungible_resource(
                        OwnerRole::None,
                        NonFungibleIdType::RUID,
                        None::<Vec<(NonFungibleLocalId, LiquidityReceipt)>>,
                        account,
                    )
            });

        let ociswap_pools = resource_addresses.map(|resource_address| {
            let manifest = ManifestBuilder::new()
                .lock_fee_from_faucet()
                .ociswap_pool_instantiate(
                    ociswap_package,
                    *resource_address,
                    XRD,
                    configuration.fees,
                    FAUCET,
                )
                .build();
            let component_address = *test_runner
                .execute_manifest(manifest, vec![])
                .expect_commit_success()
                .new_component_addresses()
                .first()
                .unwrap();

            let manifest = ManifestBuilder::new()
                .lock_fee_from_faucet()
                .mint_fungible(XRD, dec!(100_000_000))
                .mint_fungible(*resource_address, dec!(100_000_000))
                .take_all_from_worktop(XRD, "xrd_bucket")
                .take_all_from_worktop(*resource_address, "other_bucket")
                .with_name_lookup(|builder, _| {
                    let xrd_bucket = builder.bucket("xrd_bucket");
                    let other_bucket = builder.bucket("other_bucket");
                    builder.ociswap_pool_add_liquidity(
                        component_address,
                        xrd_bucket,
                        other_bucket,
                    )
                })
                .try_deposit_entire_worktop_or_abort(account, None)
                .build();
            test_runner
                .execute_manifest_without_auth(manifest)
                .expect_commit_success();

            component_address
        });

        let caviarnine_pools = resource_addresses.map(|resource_address| {
            let manifest = ManifestBuilder::new()
                .lock_fee_from_faucet()
                .allocate_global_address(
                    caviarnine_package,
                    "QuantaSwap",
                    "reservation",
                    "address",
                )
                .mint_fungible(XRD, dec!(100_000_000))
                .mint_fungible(*resource_address, dec!(100_000_000))
                .take_all_from_worktop(XRD, "xrd_bucket")
                .take_all_from_worktop(*resource_address, "other_bucket")
                .with_name_lookup(|builder, _| {
                    let reservation =
                        builder.address_reservation("reservation");
                    let address = builder.named_address("address");

                    let xrd_bucket = builder.bucket("xrd_bucket");
                    let other_bucket = builder.bucket("other_bucket");

                    builder
                        .caviar_nine_pool_new(
                            caviarnine_package,
                            rule!(allow_all),
                            rule!(allow_all),
                            *resource_address,
                            XRD,
                            50,
                            Some(reservation),
                        )
                        .caviar_nine_pool_add_liquidity(
                            address,
                            other_bucket,
                            xrd_bucket,
                            vec![(27000, dec!(100_000_000), dec!(100_000_000))],
                        )
                })
                .try_deposit_entire_worktop_or_abort(account, None)
                .build();
            *test_runner
                .execute_manifest_without_auth(manifest)
                .expect_commit_success()
                .new_component_addresses()
                .first()
                .unwrap()
        });

        let simple_oracle = test_runner
            .execute_manifest(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .call_function(
                        simple_oracle_package,
                        "SimpleOracle",
                        "instantiate",
                        (
                            protocol_manager_rule.clone(),
                            OwnerRole::None,
                            None::<ManifestAddressReservation>,
                        ),
                    )
                    .build(),
                vec![],
            )
            .expect_commit_success()
            .new_component_addresses()
            .first()
            .copied()
            .unwrap();

        // Submitting some dummy prices to the oracle to get things going.
        resource_addresses.map(|resource_address| {
            test_runner
                .execute_manifest_without_auth(
                    ManifestBuilder::new()
                        .lock_fee_from_faucet()
                        .call_method(
                            simple_oracle,
                            "set_price",
                            (*resource_address, XRD, dec!(1)),
                        )
                        .build(),
                )
                .expect_commit_success();
        });

        // Initializing ignition with information
        let ignition = test_runner
            .execute_manifest(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .call_function(
                        ignition_package,
                        "Ignition",
                        "instantiate",
                        (
                            OwnerRole::None,
                            protocol_owner_rule,
                            protocol_manager_rule,
                            XRD,
                            simple_oracle,
                            configuration
                                .maximum_allowed_price_staleness_seconds,
                            configuration
                                .maximum_allowed_relative_price_difference,
                            InitializationParametersManifest::default(),
                            None::<ManifestAddressReservation>,
                        ),
                    )
                    .build(),
                vec![],
            )
            .expect_commit_success()
            .new_component_addresses()
            .first()
            .copied()
            .unwrap();

        let [ociswap_adapter_v1, caviarnine_adapter_v1] = [
            (ociswap_adapter_v1_package, "OciswapAdapter"),
            (caviarnine_adapter_v1_package, "CaviarNineAdapter"),
        ]
        .map(|(package_address, blueprint_name)| {
            test_runner
                .execute_manifest(
                    ManifestBuilder::new()
                        .lock_fee_from_faucet()
                        .call_function(
                            package_address,
                            blueprint_name,
                            "instantiate",
                            (
                                OwnerRole::None,
                                None::<ManifestAddressReservation>,
                            ),
                        )
                        .build(),
                    vec![],
                )
                .expect_commit_success()
                .new_component_addresses()
                .first()
                .copied()
                .unwrap()
        });

        {
            let manifest = ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_method(ignition, "set_is_open_position_enabled", (true,))
                .call_method(ignition, "set_is_close_position_enabled", (true,))
                .call_method(
                    ignition,
                    "add_reward_rate",
                    (LockupPeriod::from_months(6), dec!(0.2)),
                )
                .call_method(
                    ignition,
                    "add_reward_rate",
                    (LockupPeriod::from_months(12), dec!(0.4)),
                )
                .mint_fungible(XRD, dec!(200_000_000_000_000))
                .take_from_worktop(XRD, dec!(100_000_000_000_000), "volatile")
                .take_from_worktop(
                    XRD,
                    dec!(100_000_000_000_000),
                    "non_volatile",
                )
                .with_name_lookup(|builder, _| {
                    let volatile = builder.bucket("volatile");
                    let non_volatile = builder.bucket("non_volatile");

                    builder
                        .call_method(
                            ignition,
                            "deposit_protocol_resources",
                            (volatile, Volatility::Volatile),
                        )
                        .call_method(
                            ignition,
                            "deposit_protocol_resources",
                            (non_volatile, Volatility::NonVolatile),
                        )
                })
                .with_name_lookup(|mut builder, _| {
                    let ResourceInformation {
                        bitcoin,
                        ethereum,
                        usdc,
                        usdt,
                    } = resource_addresses;

                    for instruction in [
                        (bitcoin, Volatility::Volatile),
                        (ethereum, Volatility::Volatile),
                        (usdc, Volatility::NonVolatile),
                        (usdt, Volatility::NonVolatile),
                    ]
                    .map(|(address, volatility)| InstructionV1::CallMethod {
                        address: ignition.into(),
                        method_name: "insert_user_resource_volatility"
                            .to_owned(),
                        args: manifest_args!(address, volatility).into(),
                    }) {
                        builder =
                            builder.add_instruction_advanced(instruction).0;
                    }

                    for (
                        adapter_address,
                        pools,
                        liquidity_receipt,
                        package_address,
                        blueprint_name,
                    ) in [
                        (
                            ociswap_adapter_v1,
                            ociswap_pools,
                            ociswap_liquidity_receipt_resource,
                            ociswap_package,
                            "BasicPool",
                        ),
                        (
                            caviarnine_adapter_v1,
                            caviarnine_pools,
                            caviarnine_liquidity_receipt_resource,
                            caviarnine_package,
                            "QuantaSwap",
                        ),
                    ] {
                        builder = builder.call_method(
                            ignition,
                            "insert_pool_information",
                            (
                                BlueprintId {
                                    package_address,
                                    blueprint_name: blueprint_name.to_owned(),
                                },
                                (
                                    adapter_address,
                                    pools.iter().collect::<Vec<_>>(),
                                    liquidity_receipt,
                                ),
                            ),
                        );
                    }

                    builder
                })
                .build();
            test_runner
                .execute_manifest_with_enabled_modules(
                    manifest,
                    EnabledModules::for_test_transaction()
                        & !EnabledModules::AUTH
                        & !EnabledModules::COSTING,
                )
                .expect_commit_success();
        }

        Self {
            environment: test_runner,
            resources: resource_addresses,
            protocol: ProtocolEntities {
                ignition_package_address: ignition_package,
                ignition,
                oracle_package_address: simple_oracle_package,
                oracle: simple_oracle,
                protocol_owner_badge: (
                    public_key.into(),
                    account,
                    protocol_owner_badge,
                ),
                protocol_manager_badge: (
                    public_key.into(),
                    account,
                    protocol_manager_badge,
                ),
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
        }
    }
}

impl Default for ScryptoUnitEnv {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct ProtocolEntities<S>
where
    S: EnvironmentSpecifier,
{
    /* Ignition */
    pub ignition_package_address: PackageAddress,
    pub ignition: S::Ignition,
    /* Oracle */
    pub oracle_package_address: PackageAddress,
    pub oracle: S::SimpleOracle,
    /* Badges */
    pub protocol_owner_badge: S::Badge,
    pub protocol_manager_badge: S::Badge,
}

/// A struct that defines the entities that belong to a Decentralized Exchange.
/// it contains the package address as well as generic items [`T`] which are
/// the stubs used to call the pools.
#[derive(Copy, Clone, Debug)]
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
    pub fn map<F, O>(&self, mut map: F) -> ResourceInformation<O>
    where
        F: FnMut(&T) -> O,
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
