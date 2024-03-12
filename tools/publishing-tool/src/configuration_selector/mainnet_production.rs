use crate::publishing::*;
use crate::utils::*;
use crate::*;
use common::prelude::*;
use radix_engine_interface::prelude::*;
use transaction::prelude::*;

pub fn mainnet_production(
    notary_private_key: &PrivateKey,
) -> PublishingConfiguration {
    let notary_account_address =
        ComponentAddress::virtual_account_from_public_key(
            &notary_private_key.public_key(),
        );

    PublishingConfiguration {
        protocol_configuration: ProtocolConfiguration {
            // The protocol resource to use is XRD.
            protocol_resource: XRD,
            // This is the volatility classification of the various resources
            // that we will be supporting. This signals to Ignition which XRD
            // vault it should use for contributions of this resource.
            user_resource_volatility: UserResourceIndexedData {
                bitcoin: Volatility::Volatile,
                ethereum: Volatility::Volatile,
                usdc: Volatility::NonVolatile,
                usdt: Volatility::NonVolatile,
            },
            // This is a mapping of the reward rate in months to the upfront 
            // reward percentage.
            reward_rates: indexmap! {
                LockupPeriod::from_months(9).unwrap() => dec!(0.125),  // 12.5%
                LockupPeriod::from_months(10).unwrap() => dec!(0.145), // 14.5%
                LockupPeriod::from_months(11).unwrap() => dec!(0.17),  // 17.0%
                LockupPeriod::from_months(12).unwrap() => dec!(0.2),  // 20.0%
            },
            // When Ignition is first deployed nobody is allowed to open or
            // close positions.
            allow_opening_liquidity_positions: false,
            allow_closing_liquidity_positions: false,
            // The maximum allowed price staleness is 60 seconds
            maximum_allowed_price_staleness_in_seconds: 60,
            // The maximum allowed price difference percentage is 5% from the 
            // oracle price.
            maximum_allowed_price_difference_percentage: dec!(0.05),
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
                        "description" => "An adapter used by the Ignition protocol for Ociswap v2 interactions.", updatable;
                        // Dapp definition will be automatically added by the
                        // publisher accordingly.
                    },
                    defiplaza_v2: metadata_init! {
                        "name" => "Ignition DefiPlaza v2 Adapter", updatable;
                        "description" => "An adapter used by the Ignition protocol for DefiPlaza v2 interactions.", updatable;
                        // Dapp definition will be automatically added by the
                        // publisher accordingly.
                    },
                    caviarnine_v1: metadata_init! {
                        "name" => "Ignition Caviarnine v1 Adapter", updatable;
                        "description" => "An adapter used by the Ignition protocol for Caviarnine v1 interactions.", updatable;
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
            // Whoever notarizes this transaction will also be handling the 
            // payment of fees for it.
            notary: clone_private_key(notary_private_key),
            fee_payer_information: AccountAndControllingKey::new_virtual_account(
                clone_private_key(notary_private_key),
            ),
        },
        badges: BadgeIndexedData {
            oracle_manager_badge: BadgeHandling::CreateAndSend {
                // This is the account of devops that runs the oracle software
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
                // When we initially deploy we should send the protocol manager
                // badge to the notary's account address. 
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
                // When we initially deploy we should send the protocol manager
                // badge to the notary's account address. 
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
        // The address of the user resources. These have been obtained from the
        // dapp definition of the instabridge application found here:
        // https://dashboard.radixdlt.com/account/account_rdx1cxamqz2f03s8g6smfz32q2gr3prhwh3gqdkdk93d8q8srp8d38cs7e/metadata
        // and have been verified against the addresses we have seen on the 
        // exchanges.
        user_resources: UserResourceIndexedData {
            bitcoin: UserResourceHandling::UseExisting {
                resource_address: resource_address!(
                    "resource_rdx1t580qxc7upat7lww4l2c4jckacafjeudxj5wpjrrct0p3e82sq4y75"
                ),
            },
            ethereum: UserResourceHandling::UseExisting {
                resource_address: resource_address!(
                    "resource_rdx1th88qcj5syl9ghka2g9l7tw497vy5x6zaatyvgfkwcfe8n9jt2npww"
                ),
            },
            usdc: UserResourceHandling::UseExisting {
                resource_address: resource_address!(
                    "resource_rdx1t4upr78guuapv5ept7d7ptekk9mqhy605zgms33mcszen8l9fac8vf"
                ),
            },
            usdt: UserResourceHandling::UseExisting {
                resource_address: resource_address!(
                    "resource_rdx1thrvr3xfs2tarm2dl9emvs26vjqxu6mqvfgvqjne940jv0lnrrg7rw"
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
            // No ociswap v2 currently on mainnet. So, we set this to be None. 
            // When they are live the protocol manager can add support for them
            // in Ignition manually and outside of the publishing process.
            ociswap_v2: None,
            // The package and pools found here have been given to us by the
            // Defiplaza team here:
            // https://rdxworks.slack.com/archives/C06908324TX/p1709652626987359
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
                            "component_rdx1cpv5g5a86qezw0g46w2ph8ydlu2m7jnzxw9p4lx6593qn9fmnwerta"
                        ),
                    },
                    ethereum: PoolHandling::UseExisting {
                        pool_address: component_address!(
                            "component_rdx1crwdzvlv7djtkug9gmvp9ejun0gm0w6cvkpfqycw8fcp4gg82eftjc"
                        ),
                    },
                    usdc: PoolHandling::UseExisting {
                        pool_address: component_address!(
                            "component_rdx1cpw85pmjl8ujjq7kp50lgh3ej5hz3ky9x65q2cjqvg4efnhcmfpz27"
                        ),
                    },
                    usdt: PoolHandling::UseExisting {
                        pool_address: component_address!(
                            "component_rdx1czr2hzfv2xnxdsts4a02dglkn05clv3a2t9uk04709utehau8gjv8h"
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
                        // I have confirmed this with DefiPlaza to be the 
                        // correct link.
                        "redeem_url" => UncheckedUrl::of("https://radix.defiplaza.net/ignition"), updatable;
                    },
                },
            }),
            // The package and pools found here have been given to us by the
            // Caviarnine team here:
            // https://rdxworks.slack.com/archives/C066A9AD4SE/p1709569482550539
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
                            "component_rdx1cp9w8443uyz2jtlaxnkcq84q5a5ndqpg05wgckzrnd3lgggpa080ed"
                        ),
                    },
                    ethereum: PoolHandling::UseExisting {
                        pool_address: component_address!(
                            "component_rdx1cpsvw207842gafeyvf6tc0gdnq47u3mn74kvzszqlhc03lrns52v82"
                        ),
                    },
                    usdc: PoolHandling::UseExisting {
                        pool_address: component_address!(
                            "component_rdx1cr6lxkr83gzhmyg4uxg49wkug5s4wwc3c7cgmhxuczxraa09a97wcu"
                        ),
                    },
                    usdt: PoolHandling::UseExisting {
                        pool_address: component_address!(
                            "component_rdx1cqs338cyje65rk44zgmjvvy42qcszrhk9ewznedtkqd8l3crtgnmh5"
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
                        // I have confirmed this with Caviarnine to be the 
                        // correct link.
                        "redeem_url" => UncheckedUrl::of("https://www.caviarnine.com/ignition"), updatable;
                    },
                },
            }),
        },
        additional_information: AdditionalInformation {
            ociswap_v2_registry_component_and_dapp_definition: None,
        },
        additional_operation_flags: AdditionalOperationFlags::empty(),
    }
}
