use common::prelude::*;

use self::utils::*;
use crate::*;

pub fn stokenet_testing(
    notary_private_key: &PrivateKey,
) -> PublishingConfiguration {
    let notary_account_address =
        ComponentAddress::virtual_account_from_public_key(
            &notary_private_key.public_key(),
        );

    // cSpell:disable
    PublishingConfiguration {
        protocol_configuration: ProtocolConfiguration {
            protocol_resource: XRD,
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
                account_address: notary_account_address,
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
                    "resource_tdx_2_1thltk578jr4v7axqpu5ceznhlha6ca2qtzcflqdmytgtf37xncu7l9"
                ),
            },
            ethereum: UserResourceHandling::UseExisting {
                resource_address: resource_address!(
                    "resource_tdx_2_1t59gx963vzd6u6fz63h5de2zh9nmgwxc8y832edmr6pxvz98wg6zu3"
                ),
            },
            usdc: UserResourceHandling::UseExisting {
                resource_address: resource_address!(
                    "resource_tdx_2_1thfv477eqwlh8x4wt6xsc62myt4z0zxmdpr4ea74fa8jnxh243y60r"
                ),
            },
            usdt: UserResourceHandling::UseExisting {
                resource_address: resource_address!(
                    "resource_tdx_2_1t4p3ytx933n576pdps4ua7jkjh36zrh36a543u0tfcsu2vthavlqg8"
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
            ociswap_v2: Some(ExchangeInformation {
                blueprint_id: BlueprintId {
                    package_address: package_address!(
                        "package_tdx_2_1phgf5er6zx60wu4jjhtps97akqjpv787f6k7rjqkxgdpacng89a4uz"
                    ),
                    blueprint_name: "LiquidityPool".to_owned(),
                },
                pools: UserResourceIndexedData {
                    bitcoin: PoolHandling::Create,
                    ethereum: PoolHandling::Create,
                    usdc: PoolHandling::Create,
                    usdt: PoolHandling::Create,
                },
                liquidity_receipt: LiquidityReceiptHandling::CreateNew {
                    non_fungible_schema:
                        NonFungibleDataSchema::new_local_without_self_package_replacement::<
                            LiquidityReceipt<AnyValue>,
                        >(),
                    metadata: metadata_init! {
                        "name" => "Ignition LP: Ociswap", updatable;
                        "description" => "Represents a particular contribution of liquidity to Ociswap through the Ignition liquidity incentives program. See the redeem_url metadata for where to redeem these NFTs.", updatable;
                        "tags" => vec!["lp token"], updatable;
                        "icon_url" => UncheckedUrl::of("https://assets.radixdlt.com/icons/icon-Ignition-LP.png"), updatable;
                        "DEX" => "Ociswap", updatable;
                        // TODO: Must get this from the DEX
                        "redeem_url" => UncheckedUrl::of("https://www.google.com"), updatable;
                    },
                },
            }),
            caviarnine_v1: None,
            defiplaza_v2: None,
        },
        additional_information: AdditionalInformation {
            ociswap_v2_registry_component: Some(component_address!(
                "component_tdx_2_1cpwm3sjxr48gmsnh7lgmh5de3eqqzthqkazztc4qv6n3fvedgjepwk"
            )),
        },
        additional_operation_flags: AdditionalOperationFlags::SUBMIT_ORACLE_PRICES_OF_ONE
        // cSpell:enable
    }
}
