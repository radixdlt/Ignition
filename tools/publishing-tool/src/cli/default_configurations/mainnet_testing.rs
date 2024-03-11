use common::prelude::*;
use publishing_tool::publishing::*;
use publishing_tool::utils::*;
use publishing_tool::*;
use radix_engine_interface::prelude::*;
use transaction::prelude::*;

pub fn mainnet_testing(
    notary_private_key: &PrivateKey,
) -> PublishingConfiguration {
    let notary_account_address =
        ComponentAddress::virtual_account_from_public_key(
            &notary_private_key.public_key(),
        );

    // cSpell:disable
    PublishingConfiguration {
        protocol_configuration: ProtocolConfiguration {
            protocol_resource: resource_address!(
                "resource_rdx1t4dekrf58h0r28s3c93z92w3jt5ngx87jzd63mgc597zmf3534rxfv"
            ),
            user_resource_volatility: UserResourceIndexedData {
                bitcoin: Volatility::Volatile,
                ethereum: Volatility::Volatile,
                usdc: Volatility::NonVolatile,
                usdt: Volatility::NonVolatile,
            },
            reward_rates: indexmap! {
                LockupPeriod::from_minutes(0).unwrap() => dec!(0.125),  // 12.5%
                LockupPeriod::from_minutes(1).unwrap() => dec!(0.15),   // 15.0%
            },
            allow_opening_liquidity_positions: true,
            allow_closing_liquidity_positions: true,
            maximum_allowed_price_staleness: i64::MAX,
            maximum_allowed_price_difference_percentage: Decimal::MAX,
            entities_metadata: Entities {
                protocol_entities: ProtocolIndexedData {
                    ignition: metadata_init! {
                        "name" => "Ignition", updatable;
                        "description" => "The main entrypoint into the Ignition liquidity incentive program.", updatable;
                        // Dapp definition will be automatically added by the
                        // publisher accordingly.
                    },
                    simple_oracle: metadata_init! {
                        "name" => "Ignition Oracle", updatable;
                        "description" => "The oracle used by the Ignition protocol.", updatable;
                        "tags" => vec!["oracle"], updatable;
                        // Dapp definition will be automatically added by the
                        // publisher accordingly.
                    },
                },
                exchange_adapter_entities: ExchangeIndexedData {
                    ociswap_v2: metadata_init! {
                        "name" => "Ignition Ociswap v2 Adapter", updatable;
                        "description" => "An adapter used by the Ignition protocol to communicate with Ociswap v2 pools.", updatable;
                        // Dapp definition will be automatically added by the
                        // publisher accordingly.
                    },
                    defiplaza_v2: metadata_init! {
                        "name" => "Ignition DefiPlaza v2 Adapter", updatable;
                        "description" => "An adapter used by the Ignition protocol to communicate with DefiPlaza v2 pools.", updatable;
                        // Dapp definition will be automatically added by the
                        // publisher accordingly.
                    },
                    caviarnine_v1: metadata_init! {
                        "name" => "Ignition Caviarnine v1 Adapter", updatable;
                        "description" => "An adapter used by the Ignition protocol to communicate with Caviarnine v1 pools.", updatable;
                        // Dapp definition will be automatically added by the
                        // publisher accordingly.
                    },
                },
            },
        },
        dapp_definition_metadata: indexmap! {
            "name".to_owned() => MetadataValue::String("Project Ignition".to_owned()),
            "description".to_owned() => MetadataValue::String("A Radix liquidity incentives program, offered in partnership with select decentralized exchange dApps in the Radix ecosystem.".to_owned()),
            "icon_url".to_owned() => MetadataValue::Url(UncheckedUrl::of("https://assets.radixdlt.com/icons/icon-Ignition-LP.png"))
        },
        transaction_configuration: TransactionConfiguration {
            notary: clone_private_key(notary_private_key),
            fee_payer_information: AccountAndControllingKey::new_virtual_account(
                clone_private_key(notary_private_key),
            ),
        },
        // TODO: Determine where they should be sent to.
        badges: BadgeIndexedData {
            oracle_manager_badge: BadgeHandling::CreateAndSend {
                account_address: component_address!(
                    "account_rdx168nr5dwmll4k2x5apegw5dhrpejf3xac7khjhgjqyg4qddj9tg9v4d"
                ),
                metadata_init: metadata_init! {
                    "name" => "Ignition Oracle Manager", updatable;
                    "symbol" => "IGNOM", updatable;
                    "description" => "A badge with the authority to update the Oracle prices of the Ignition oracle.", updatable;
                    "tags" => vec!["badge"], updatable;
                    // Dapp definition will be automatically added by the
                    // publisher accordingly.
                },
            },
            protocol_manager_badge: BadgeHandling::CreateAndSend {
                account_address: notary_account_address,
                metadata_init: metadata_init! {
                    "name" => "Ignition Protocol Manager", updatable;
                    "symbol" => "IGNPM", updatable;
                    "description" => "A badge with the authority to manage the Ignition protocol.", updatable;
                    "tags" => vec!["badge"], updatable;
                    // Dapp definition will be automatically added by the
                    // publisher accordingly.
                },
            },
            protocol_owner_badge: BadgeHandling::CreateAndSend {
                account_address: notary_account_address,
                metadata_init: metadata_init! {
                    "name" => "Ignition Protocol Owner", updatable;
                    "symbol" => "IGNPO", updatable;
                    "description" => "A badge with owner authority over the Ignition protocol.", updatable;
                    "tags" => vec!["badge"], updatable;
                    // Dapp definition will be automatically added by the
                    // publisher accordingly.
                },
            },
        },
        // TODO: Not real resources, just the notXYZ resources.
        user_resources: UserResourceIndexedData {
            bitcoin: UserResourceHandling::UseExisting {
                resource_address: resource_address!(
                    "resource_rdx1t58dla7ykxzxe5es89wlhgzatqla0gceukg0eeduzvtj4cxd55etn8"
                ),
            },
            ethereum: UserResourceHandling::UseExisting {
                resource_address: resource_address!(
                    "resource_rdx1tkscrlztcyn82ej5z3n232f0qqp0qur69arjf279ppmg5usa3xhnsm"
                ),
            },
            usdc: UserResourceHandling::UseExisting {
                resource_address: resource_address!(
                    "resource_rdx1th7nx2hy0cf6aea6mz7zhkdmy4p45s488xutltnp7296zxj8hwchpf"
                ),
            },
            usdt: UserResourceHandling::UseExisting {
                resource_address: resource_address!(
                    "resource_rdx1tkafx32lu72mcxr85gjx0rh3rx9q89zqffg4phmv5rxdqg5fnd0w7s"
                ),
            },
        },
        packages: Entities {
            protocol_entities: ProtocolIndexedData {
                ignition: PackageHandling::LoadAndPublish {
                    crate_package_name: "ignition".to_owned(),
                    metadata: metadata_init! {
                        "name" => "Ignition Package", updatable;
                        "description" => "The implementation of the Ignition protocol.", updatable;
                        "tags" => Vec::<String>::new(), updatable;
                        // Dapp definition will be automatically added by the
                        // publisher accordingly.
                    },
                    blueprint_name: "Ignition".to_owned(),
                },
                simple_oracle: PackageHandling::LoadAndPublish {
                    crate_package_name: "simple-oracle".to_owned(),
                    metadata: metadata_init! {
                        "name" => "Ignition Simple Oracle Package", updatable;
                        "description" => "The implementation of the Oracle used by the Ignition protocol.", updatable;
                        "tags" => vec!["oracle"], updatable;
                        // Dapp definition will be automatically added by the
                        // publisher accordingly.
                    },
                    blueprint_name: "SimpleOracle".to_owned(),
                },
            },
            exchange_adapter_entities: ExchangeIndexedData {
                ociswap_v2: PackageHandling::LoadAndPublish {
                    crate_package_name: "ociswap-v2-adapter-v1".to_owned(),
                    metadata: metadata_init! {
                        "name" => "Ignition Ociswap v2 Adapter Package", updatable;
                        "description" => "The implementation of an adapter for Ociswap v2 for the Ignition protocol.", updatable;
                        "tags" => vec!["adapter"], updatable;
                        // Dapp definition will be automatically added by the
                        // publisher accordingly.
                    },
                    blueprint_name: "OciswapV2Adapter".to_owned(),
                },
                defiplaza_v2: PackageHandling::LoadAndPublish {
                    crate_package_name: "defiplaza-v2-adapter-v1".to_owned(),
                    metadata: metadata_init! {
                        "name" => "Ignition DefiPlaza v2 Adapter Package", updatable;
                        "description" => "The implementation of an adapter for DefiPlaza v1 for the Ignition protocol.", updatable;
                        "tags" => vec!["adapter"], updatable;
                        // Dapp definition will be automatically added by the
                        // publisher accordingly.
                    },
                    blueprint_name: "DefiPlazaV2Adapter".to_owned(),
                },
                caviarnine_v1: PackageHandling::LoadAndPublish {
                    crate_package_name: "caviarnine-v1-adapter-v1".to_owned(),
                    metadata: metadata_init! {
                        "name" => "Ignition Caviarnine v1 Adapter Package", updatable;
                        "description" => "The implementation of an adapter for Caviarnine v1 for the Ignition protocol.", updatable;
                        "tags" => vec!["adapter"], updatable;
                        // Dapp definition will be automatically added by the
                        // publisher accordingly.
                    },
                    blueprint_name: "CaviarnineV1Adapter".to_owned(),
                },
            },
        },
        exchange_information: ExchangeIndexedData {
            // No ociswap v2 currently on mainnet.
            ociswap_v2: None,
            defiplaza_v2: Some(ExchangeInformation {
                blueprint_id: BlueprintId {
                    package_address: package_address!(
                        "package_rdx1p4dhfl7qwthqqu6p2267m5nedlqnzdvfxdl6q7h8g85dflx8n06p93"
                    ),
                    blueprint_name: "PlazaPair".to_owned(),
                },
                pools: UserResourceIndexedData {
                    bitcoin: PoolHandling::UseExisting {
                        pool_address: component_address!(
                            "component_rdx1cpgyq8809z4mnc5rw2pvru3xdcjftjv45a5cgcwyqdqtg2xs35r58r"
                        ),
                    },
                    ethereum: PoolHandling::UseExisting {
                        pool_address: component_address!(
                            "component_rdx1cpzf8pygechpgat29phu72nzn4gn6shu7x0fdjydjky6g683sl0azk"
                        ),
                    },
                    usdc: PoolHandling::UseExisting {
                        pool_address: component_address!(
                            "component_rdx1cq32fjfp8gu3hh8jau9m6syfaargxagmvcakwwx966ejy6cwczghw4"
                        ),
                    },
                    usdt: PoolHandling::UseExisting {
                        pool_address: component_address!(
                            "component_rdx1crx4h8dljzufy9m3g5ez49d5ge2q0vwysfc77vxrp8x480rqq3qpre"
                        ),
                    },
                },
                liquidity_receipt: LiquidityReceiptHandling::CreateNew {
                    non_fungible_schema:
                        NonFungibleDataSchema::new_local_without_self_package_replacement::<
                            LiquidityReceipt<AnyValue>,
                        >(),
                    metadata: metadata_init! {
                        "name" => "Ignition LP: DefiPlaza", updatable;
                        "description" => "Represents a particular contribution of liquidity to DefiPlaza through the Ignition liquidity incentives program. See the redeem_url metadata for where to redeem these NFTs.", updatable;
                        "tags" => vec!["lp token"], updatable;
                        "icon_url" => UncheckedUrl::of("https://assets.radixdlt.com/icons/icon-Ignition-LP.png"), updatable;
                        "DEX" => "DefiPlaza", updatable;
                        // TODO: Must get this from the DEX
                        "redeem_url" => UncheckedUrl::of("https://www.google.com"), updatable;
                    },
                },
            }),
            caviarnine_v1: Some(ExchangeInformation {
                blueprint_id: BlueprintId {
                    package_address: package_address!(
                        "package_rdx1p4r9rkp0cq67wmlve544zgy0l45mswn6h798qdqm47x4762h383wa3"
                    ),
                    blueprint_name: "QuantaSwap".to_owned(),
                },
                pools: UserResourceIndexedData {
                    bitcoin: PoolHandling::UseExisting {
                        pool_address: component_address!(
                            "component_rdx1crzl2c39m83lpe6fv62epgp3phqunxhc264ys23qz8xeemjcu8lln3"
                        ),
                    },
                    ethereum: PoolHandling::UseExisting {
                        pool_address: component_address!(
                            "component_rdx1cqk2ufmdq6pkcu7ed7r6u9hmdsht9gyd8y8wwtd7w5znefz9k54a7d"
                        ),
                    },
                    usdc: PoolHandling::UseExisting {
                        pool_address: component_address!(
                            "component_rdx1cq9q8umlpmngff6y4e534htz0n37te4m7vsj50u9zc58ys65zl6jv9"
                        ),
                    },
                    usdt: PoolHandling::UseExisting {
                        pool_address: component_address!(
                            "component_rdx1cpl0v3lndt9d7g7uuepztxs9m7m24ly0yfhvcum2y7tm0vlzst0l5y"
                        ),
                    },
                },
                liquidity_receipt: LiquidityReceiptHandling::CreateNew {
                    non_fungible_schema:
                        NonFungibleDataSchema::new_local_without_self_package_replacement::<
                            LiquidityReceipt<AnyValue>,
                        >(),
                    metadata: metadata_init! {
                        "name" => "Ignition LP: Caviarnine", updatable;
                        "description" => "Represents a particular contribution of liquidity to Caviarnine through the Ignition liquidity incentives program. See the redeem_url metadata for where to redeem these NFTs.", updatable;
                        "tags" => vec!["lp token"], updatable;
                        "icon_url" => UncheckedUrl::of("https://assets.radixdlt.com/icons/icon-Ignition-LP.png"), updatable;
                        "DEX" => "Caviarnine", updatable;
                        // TODO: Must get this from the DEX
                        "redeem_url" => UncheckedUrl::of("https://www.google.com"), updatable;
                    },
                },
            }),
        },
        additional_information: AdditionalInformation {
            ociswap_v2_registry_component_and_dapp_definition: None,
        },
        additional_operation_flags: AdditionalOperationFlags::empty(), // cSpell:enable
    }
}
