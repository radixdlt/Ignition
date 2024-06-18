// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use crate::publishing::*;
use crate::utils::*;
use crate::*;
use caviarnine_v1_adapter_v2::ContributionBinConfiguration;
use common::prelude::*;
use radix_common::prelude::*;
use radix_engine_interface::prelude::*;
use radix_transactions::prelude::*;

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
                lsu_lp_resource: Volatility::NonVolatile,
            },
            // This is a mapping of the reward rate in months to the upfront
            // reward percentage.
            reward_rates: indexmap! {
                LockupPeriod::from_months(6).unwrap() => dec!(0.04),
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
                    caviarnine_v1: metadata_init! {
                        "name" => "Ignition Caviarnine v1 Adapter", updatable;
                        "description" => "An adapter used by the Ignition protocol for Caviarnine v1 interactions.", updatable;
                        // Dapp definition will be automatically added by the
                        // publisher accordingly.
                    },
                },
            },
            matching_factors: UserResourceIndexedData {
                lsu_lp_resource: dec!(0.4),
            },
        },
        dapp_definition: DappDefinitionHandling::UseExistingOneWayLink {
            component_address: component_address!(
                "account_rdx1cxh9jq27n5vllmsexah8jj3txzue8yu236uekcnfr4hq5ptw8nn7f0"
            ),
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
            oracle_manager_badge: BadgeHandling::UseExisting {
                controlling_private_key: clone_private_key(notary_private_key),
                holder_account_address: notary_account_address,
                badge_resource_address: resource_address!(
                    "resource_rdx1th3yr5dlydnhw0lfp6r22x5l2fj9lv3t8f0enkp7j5ttnx3e09rhna"
                ),
            },
            protocol_manager_badge: BadgeHandling::UseExisting {
                controlling_private_key: clone_private_key(notary_private_key),
                holder_account_address: notary_account_address,
                badge_resource_address: resource_address!(
                    "resource_rdx1t5w3cekqxjcphrvtp8x5rqz55s4qk97ralrtldnlvf3t6nfhq9a4en"
                ),
            },
            protocol_owner_badge: BadgeHandling::UseExisting {
                controlling_private_key: clone_private_key(notary_private_key),
                holder_account_address: notary_account_address,
                badge_resource_address: resource_address!(
                    "resource_rdx1t5ezhhs9cnua2thfnknmpj2rysz0rtwpexvjhvylww2ng5h3makwma"
                ),
            },
        },
        user_resources: UserResourceIndexedData {
            lsu_lp_resource: UserResourceHandling::UseExisting {
                resource_address: resource_address!(
                    "resource_rdx1thksg5ng70g9mmy9ne7wz0sc7auzrrwy7fmgcxzel2gvp8pj0xxfmf"
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
                simple_oracle: PackageHandling::UseExisting {
                    blueprint_id: BlueprintId {
                        package_address: package_address!(
                            "package_rdx1phx0yptt32290n0uym4aqh3zyyup4ykth4enph8a68ggp7c38dqaxw"
                        ),
                        blueprint_name: "SimpleOracle".to_owned(),
                    },
                },
            },
            exchange_adapter_entities: ExchangeIndexedData {
                caviarnine_v1: PackageHandling::LoadAndPublish {
                    crate_package_name: "caviarnine-v1-adapter-v2".to_owned(),
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
            caviarnine_v1: Some(ExchangeInformation {
                blueprint_id: BlueprintId {
                    package_address: package_address!(
                        "package_rdx1p4r9rkp0cq67wmlve544zgy0l45mswn6h798qdqm47x4762h383wa3"
                    ),
                    blueprint_name: "QuantaSwap".to_owned(),
                },
                pools: UserResourceIndexedData {
                    lsu_lp_resource: PoolHandling::UseExisting {
                        pool_address: component_address!(
                            "component_rdx1crdhl7gel57erzgpdz3l3vr64scslq4z7vd0xgna6vh5fq5fnn9xas"
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
        oracle_handling: OracleHandling::UseExisting {
            component_address: component_address!(
                "component_rdx1cr3psyfptwkktqusfg8ngtupr4wwfg32kz2xvh9tqh4c7pwkvlk2kn"
            ),
        },
        additional_settings: AdditionalSettings {
            fund_ignition_volatile: None,
            fund_ignition_non_volatile: Some((
                component_address!(
                    "account_rdx12xvk6x3usuzu7hdc5clc7lpu8e4czze6xa7vrw7vlek0h84j9299na"
                ),
                dec!(1),
            )),
            configure_caviarnine_adapter_pool_configuration: Some(UserResourceIndexedData {
                lsu_lp_resource: indexmap! {
                    LockupPeriod::from_months(6).unwrap()
                        => ContributionBinConfiguration {
                            start_tick: 27043,
                            end_tick: 27101
                        },
                },
            }),
        },
    }
}
