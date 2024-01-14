use caviarnine_adapter_v1::*;
use ignition::ignition::*;
use ignition::{LiquidityReceipt, LockupPeriod, PoolBlueprintInformation};
use scrypto::prelude::*;

use caviarnine_adapter_v1::adapter::*;

#[blueprint]
#[events(TestingBootstrapInformation, EncodedTestingBootstrapInformation)]
mod bootstrap {
    struct Bootstrap;

    impl Bootstrap {
        pub fn bootstrap_for_testing(
            /* Core packages */
            ignition_package_address: PackageAddress,
            oracle_package_address: PackageAddress,
            /* Dexes */
            caviarnine_package_address: PackageAddress,
            /* Adapters */
            _ociswap_adapter_v1_package_address: PackageAddress,
            caviarnine_adapter_v1_package_address: PackageAddress,
        ) -> (TestingBootstrapInformation, Vec<Bucket>) {
            // A vector of buckets that must be returned by the end of this
            // process. These are obtained as a result of the creation of pools,
            // contribution to pools, or from other sources.
            let mut buckets = vec![];

            // Instantiating an oracle component.
            let oracle = scrypto_decode::<ComponentAddress>(
                &ScryptoVmV1Api::blueprint_call(
                    oracle_package_address,
                    "TestOracle",
                    "instantiate",
                    scrypto_args!(
                        rule!(allow_all),
                        OwnerRole::None,
                        None::<GlobalAddressReservation>
                    ),
                ),
            )
            .unwrap();

            // Defining the protocol's resource. This is the resource that the
            // protocol will be lending out to users.
            let protocol_resource_information = ResourceInformation {
                divisibility: 18,
                name: "Fake XRD".into(),
                symbol: "fakeXRD".into(),
                icon_url:
                    "https://assets.radixdlt.com/icons/icon-xrd-32x32.png"
                        .into(),
            };
            let protocol_resource =
                protocol_resource_information.create_resource();

            // Defining the information of the user resources. We're creating
            // these resources for testing purposed only.
            let user_resources_information = vec![
                ResourceInformation {
                    divisibility: 6,
                    name: "Fake Bitcoin".into(),
                    symbol: "fakeBTC".into(),
                    icon_url:
                        "https://assets.instabridge.io/tokens/icons/xwBTC.png"
                            .into(),
                },
                ResourceInformation {
                    divisibility: 18,
                    name: "Fake Ethereum".into(),
                    symbol: "fakeETH".into(),
                    icon_url:
                        "https://assets.instabridge.io/tokens/icons/xETH.png"
                            .into(),
                },
                ResourceInformation {
                    divisibility: 6,
                    name: "Fake USDC".into(),
                    symbol: "fakeUSDC".into(),
                    icon_url:
                        "https://assets.instabridge.io/tokens/icons/xUSDC.png"
                            .into(),
                },
                ResourceInformation {
                    divisibility: 6,
                    name: "Fake USDT".into(),
                    symbol: "fakeUSDT".into(),
                    icon_url:
                        "https://assets.instabridge.io/tokens/icons/xUSDT.png"
                            .into(),
                },
            ];
            let mut user_resources = user_resources_information
                .into_iter()
                .map(|information| {
                    let resource = information.create_resource();
                    (information, resource)
                })
                .collect::<Vec<_>>();

            // Creating the pools of all of the user resources.
            let mut caviarnine_pools = indexmap! {};
            for (_, resource_manager) in user_resources.iter_mut() {
                let mut pool = CaviarNinePoolInterfaceScryptoStub::new(
                    rule!(allow_all),
                    rule!(allow_all),
                    resource_manager.address(),
                    protocol_resource.address(),
                    50,
                    None,
                    caviarnine_package_address,
                );

                let user_resource = resource_manager.mint(100_000_000);
                let protocol_resource = protocol_resource.mint(100_000_000);
                let (receipt, change_x, change_y) = pool.add_liquidity(
                    user_resource,
                    protocol_resource,
                    vec![(30_000, dec!(100_000_000), dec!(100_000_000))],
                );
                buckets.push(receipt);
                buckets.push(change_x);
                buckets.push(change_y);

                caviarnine_pools.insert(
                    resource_manager.address(),
                    ComponentAddress::try_from(pool).unwrap(),
                );
            }

            // Instantiating the adapters of the various exchanges.
            let caviarnine_adapter = scrypto_decode::<ComponentAddress>(
                &ScryptoVmV1Api::blueprint_call(
                    caviarnine_adapter_v1_package_address,
                    "CaviarNineAdapter",
                    "instantiate",
                    scrypto_args!(
                        OwnerRole::None,
                        None::<GlobalAddressReservation>
                    ),
                ),
            )
            .map(|address| {
                Global::<CaviarNineAdapter>(CaviarNineAdapterObjectStub {
                    handle: ObjectStubHandle::Global(address.into()),
                })
            })
            .unwrap();

            // Creating the liquidity receipt resource of the various exchanges.
            let caviar_nine_liquidity_receipt =
                ResourceBuilder::new_ruid_non_fungible::<LiquidityReceipt>(OwnerRole::None)
                    .mint_roles(mint_roles! {
                        minter => rule!(allow_all);
                        minter_updater => rule!(allow_all);
                    })
                    .burn_roles(burn_roles! {
                        burner => rule!(allow_all);
                        burner_updater => rule!(allow_all);
                    })
                    .metadata(metadata! {
                        init {
                            "name" => "CaviarNine Ignition Liquidity Receipt", locked;
                            "description" => "A receipt of liquidity provided to CaviarNine through project Ignition.", locked;
                        }
                    })
                    .create_with_no_initial_supply();

            // Creating the ignition component and initializing it to the
            // expected state.
            let ignition = {
                let ignition = scrypto_decode::<ComponentAddress>(
                    &ScryptoVmV1Api::blueprint_call(
                        ignition_package_address,
                        "Ignition",
                        "instantiate",
                        scrypto_args!(
                            OwnerRole::None,
                            rule!(allow_all),
                            rule!(allow_all),
                            protocol_resource,
                            oracle,
                            300i64,
                            Decimal::MAX,
                            None::<GlobalAddressReservation>
                        ),
                    ),
                )
                .map(|address| {
                    Global::<Ignition>(IgnitionObjectStub {
                        handle: ObjectStubHandle::Global(address.into()),
                    })
                })
                .unwrap();

                // Allow the opening and closing of liquidity positions.
                ignition.set_is_close_position_enabled(true);
                ignition.set_is_open_position_enabled(true);

                // Fund ignition with the protocol resources that it needs to
                // start lending out to users.
                let protocol_resources_bucket =
                    protocol_resource.mint(dec!(100_000_000_000_000));
                ignition.deposit_resources(FungibleBucket(
                    protocol_resources_bucket,
                ));

                // Add the reward rates that Ignition will use.
                ignition
                    .add_reward_rate(LockupPeriod::from_seconds(0), dec!(0.10));
                ignition.add_reward_rate(
                    LockupPeriod::from_seconds(60),
                    dec!(0.20),
                );

                // Adding the pool information for CaviarSwap
                ignition.insert_pool_information(
                    CaviarNinePoolInterfaceScryptoStub::blueprint_id(
                        caviarnine_package_address,
                    ),
                    PoolBlueprintInformation {
                        adapter: caviarnine_adapter.address().into(),
                        allowed_pools: caviarnine_pools
                            .values()
                            .copied()
                            .collect(),
                        liquidity_receipt: caviar_nine_liquidity_receipt,
                    },
                );

                ignition
            };

            // Creating a dApp definition account for the protocol
            let (dapp_definition, bucket) = Blueprint::<Account>::create();
            buckets.push(bucket);

            let bootstrap_information = TestingBootstrapInformation {
                resources: user_resources
                    .into_iter()
                    .map(|(information, manager)| {
                        (manager.address(), information)
                    })
                    .collect(),
                protocol: ProtocolEntities {
                    ignition_package_address,
                    ignition: ignition.address(),
                    protocol_resource: protocol_resource.address(),
                    oracle_package_address,
                    oracle,
                    dapp_definition: dapp_definition.address(),
                },
                caviarnine: DexEntities {
                    package: caviarnine_package_address,
                    pools: caviarnine_pools,
                    adapter_package: caviarnine_adapter_v1_package_address,
                    adapter: caviarnine_adapter.address(),
                },
            };
            Runtime::emit_event(EncodedTestingBootstrapInformation::from(
                bootstrap_information.clone(),
            ));

            (bootstrap_information, buckets)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ScryptoEvent)]
pub struct EncodedTestingBootstrapInformation(Vec<u8>);

impl From<TestingBootstrapInformation> for EncodedTestingBootstrapInformation {
    fn from(value: TestingBootstrapInformation) -> Self {
        Self(scrypto_encode(&value).unwrap())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ScryptoEvent)]
pub struct TestingBootstrapInformation {
    pub resources: IndexMap<ResourceAddress, ResourceInformation>,
    pub protocol: ProtocolEntities,
    pub caviarnine: DexEntities,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ProtocolEntities {
    /* Ignition */
    pub ignition_package_address: PackageAddress,
    pub ignition: ComponentAddress,
    pub protocol_resource: ResourceAddress,
    /* Oracle */
    pub oracle_package_address: PackageAddress,
    pub oracle: ComponentAddress,
    /* Misc */
    pub dapp_definition: ComponentAddress,
}

/// A struct that defines the entities that belong to a Decentralized Exchange.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct DexEntities {
    /* Packages */
    pub package: PackageAddress,
    /* Pools */
    pub pools: IndexMap<ResourceAddress, ComponentAddress>,
    /* Adapter */
    pub adapter_package: PackageAddress,
    pub adapter: ComponentAddress,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ResourceInformation {
    pub divisibility: u8,
    pub name: String,
    pub symbol: String,
    pub icon_url: String,
}

impl ResourceInformation {
    pub fn create_resource(&self) -> ResourceManager {
        ResourceBuilder::new_fungible(OwnerRole::None)
            .divisibility(self.divisibility)
            .mint_roles(mint_roles! {
                minter => rule!(allow_all);
                minter_updater => rule!(allow_all);
            })
            .burn_roles(burn_roles! {
                burner => rule!(allow_all);
                burner_updater => rule!(allow_all);
            })
            .metadata(metadata! {
                init {
                    "name" => self.name.as_str(), locked;
                    "symbol" => self.symbol.as_str(), locked;
                    "description" => "This is a fake resource made just for testing, this has no value", locked;
                    "icon_url" => UncheckedUrl(self.icon_url.clone()), locked;
                }
            })
            .create_with_no_initial_supply()
    }
}
