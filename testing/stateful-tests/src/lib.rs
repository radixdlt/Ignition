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

use address_macros::*;
use common::prelude::Volatility::*;
use common::prelude::*;
use extend::*;
use publishing_tool::database_overlay::*;
use publishing_tool::publishing::*;
use radix_common::prelude::*;
use radix_engine::system::system_db_reader::SystemDatabaseReader;
use radix_engine::system::system_modules::*;
use radix_engine::transaction::*;
use radix_engine::vm::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::prelude::*;
use radix_transactions::prelude::*;
use scrypto_test::prelude::*;
use state_manager::ActualStateManagerDatabase;
use std::sync::*;

pub type StatefulLedgerSimulator<'a> = LedgerSimulator<
    NoExtension,
    UnmergeableSubstateDatabaseOverlay<'a, ActualStateManagerDatabase>,
>;

fn get_database() -> &'static ActualStateManagerDatabase {
    static DATABASE: OnceLock<ActualStateManagerDatabase> = OnceLock::new();
    DATABASE.get_or_init(|| {
        const STATE_MANAGER_DATABASE_PATH_ENVIRONMENT_VARIABLE: &str =
            "STATE_MANAGER_DATABASE_PATH";
        let Ok(state_manager_database_path) =
            std::env::var(STATE_MANAGER_DATABASE_PATH_ENVIRONMENT_VARIABLE)
                .map(std::path::PathBuf::from)
        else {
            panic!(
                "The `{}` environment variable is not set",
                STATE_MANAGER_DATABASE_PATH_ENVIRONMENT_VARIABLE
            );
        };
        ActualStateManagerDatabase::new(
            state_manager_database_path,
            Default::default(),
            &NetworkDefinition::mainnet(),
        )
        .unwrap()
    })
}

pub fn execute_test_within_environment<F, O>(test_function: F) -> O
where
    F: Fn(
        AccountAndControllingKey,
        &PublishingReceipt,
        &mut StatefulLedgerSimulator<'_>,
    ) -> O,
{
    let publishing_receipt = publishing_receipt();

    // Creating the database and the necessary overlays to run the tests.
    let overlayed_state_manager_database =
        UnmergeableSubstateDatabaseOverlay::new_unmergeable(get_database());

    // Creating a test runner from the overlayed state manager database
    let (mut ledger, _) = LedgerSimulatorBuilder::new()
        .with_custom_database(overlayed_state_manager_database)
        .without_kernel_trace()
        .build_without_bootstrapping();

    // Creating a new account which we will be using as the notary and funding
    // it with XRD. Since there is no faucet on mainnet, the only way we can
    // fund this account is by disabling the auth module and minting XRD.
    let notary_private_key =
        PrivateKey::Ed25519(Ed25519PrivateKey::from_u64(1).unwrap());
    let notary_account_address =
        ComponentAddress::virtual_account_from_public_key(
            &notary_private_key.public_key(),
        );
    ledger
        .execute_manifest_with_enabled_modules(
            ManifestBuilder::new()
                .mint_fungible(XRD, dec!(100_000_000_000))
                .deposit_batch(notary_account_address)
                .build(),
            EnabledModules::for_notarized_transaction()
                & !EnabledModules::COSTING
                & !EnabledModules::AUTH,
        )
        .expect_commit_success();

    // Enabling contributions through Ignition
    ledger
        .execute_manifest_with_enabled_modules(
            ManifestBuilder::new()
                .call_method(
                    publishing_receipt.components.protocol_entities.ignition,
                    "set_is_open_position_enabled",
                    (true,),
                )
                .call_method(
                    publishing_receipt.components.protocol_entities.ignition,
                    "set_is_close_position_enabled",
                    (true,),
                )
                .call_method(
                    publishing_receipt.components.protocol_entities.ignition,
                    "set_maximum_allowed_price_difference_percentage",
                    (dec!(0.05),),
                )
                .build(),
            EnabledModules::for_notarized_transaction()
                & !EnabledModules::COSTING
                & !EnabledModules::AUTH,
        )
        .expect_commit_success();

    // Creating an account to use for the testing that has some of each of the
    // resources
    let test_account_private_key =
        PrivateKey::Ed25519(Ed25519PrivateKey::from_u64(2).unwrap());
    let test_account_address =
        ComponentAddress::virtual_account_from_public_key(
            &test_account_private_key.public_key(),
        );
    ledger
        .execute_manifest_with_enabled_modules(
            TransactionManifestV1 {
                instructions: std::iter::once(XRD)
                    .chain(publishing_receipt.user_resources.iter().copied())
                    .map(|resource_address| InstructionV1::CallMethod {
                        address: resource_address.into(),
                        method_name: FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT
                            .to_owned(),
                        args: to_manifest_value(
                            &FungibleResourceManagerMintInput {
                                amount: dec!(100_000_000_000),
                            },
                        )
                        .unwrap(),
                    })
                    .chain(std::iter::once(InstructionV1::CallMethod {
                        address: test_account_address.into(),
                        method_name: ACCOUNT_DEPOSIT_BATCH_IDENT.into(),
                        args: to_manifest_value(&(
                            ManifestExpression::EntireWorktop,
                        ))
                        .unwrap(),
                    }))
                    .collect(),
                blobs: Default::default(),
            },
            EnabledModules::for_notarized_transaction()
                & !EnabledModules::COSTING
                & !EnabledModules::AUTH,
        )
        .expect_commit_success();
    let test_account =
        AccountAndControllingKey::new_virtual_account(test_account_private_key);

    // We are now ready to execute the function callback and return its output
    // back
    test_function(test_account, publishing_receipt, &mut ledger)
}

/// This macro can be applied to any function to turn it into a test function
/// that takes arguments. The arguments given is the mainnet state after the
/// publishing of Ignition to the network. The following is an example:
///
/// ```norun
/// use macro_rules_attribute::apply;
///
/// #[apply(mainnet_test)]
/// fn example(
///     _test_account: AccountAndControllingKey,
///     _configuration: PublishingConfiguration,
///     _receipt: PublishingReceipt,
///     _ledger: &mut StatefulLedgerSimulator<'_>,
/// ) -> Result<(), RuntimeError> {
///     assert!(false);
///     Ok(())
/// }
/// ```
///
/// The above function will be treated as a test and will be discoverable by
/// the testing harness.
#[macro_export]
macro_rules! mainnet_test {
    (
        $(#[$meta: meta])*
        $fn_vis: vis fn $fn_ident: ident (
            $($tokens: tt)*
        ) $(-> $return_type: ty)? $block: block
    ) => {
        #[test]
        $(#[$meta])*
        $fn_vis fn $fn_ident () $(-> $return_type)? {
            $crate::execute_test_within_environment(|
                $($tokens)*
            | -> $crate::resolve_return_type!($(-> $return_type)?) {
                $block
            })
        }
    };
}

#[macro_export]
macro_rules! resolve_return_type {
    () => {
        ()
    };
    (-> $type: ty) => {
        $type
    };
}

#[ext]
pub impl<'a> StatefulLedgerSimulator<'a> {
    fn execute_manifest_without_auth(
        &mut self,
        manifest: TransactionManifestV1,
    ) -> TransactionReceiptV1 {
        self.execute_manifest_with_enabled_modules(
            manifest,
            EnabledModules::for_notarized_transaction() & !EnabledModules::AUTH,
        )
    }

    fn execute_manifest_with_enabled_modules(
        &mut self,
        manifest: TransactionManifestV1,
        enabled_modules: EnabledModules,
    ) -> TransactionReceiptV1 {
        let mut execution_config = ExecutionConfig::for_test_transaction();
        execution_config.system_overrides = Some(SystemOverrides {
            disable_costing: !enabled_modules.contains(EnabledModules::COSTING),
            disable_limits: !enabled_modules.contains(EnabledModules::LIMITS),
            disable_auth: !enabled_modules.contains(EnabledModules::AUTH),
            network_definition: Default::default(),
            costing_parameters: Default::default(),
            limit_parameters: Default::default(),
        });
        execution_config.enable_kernel_trace =
            enabled_modules.contains(EnabledModules::KERNEL_TRACE);
        execution_config.enable_cost_breakdown =
            enabled_modules.contains(EnabledModules::KERNEL_TRACE);
        execution_config.execution_trace =
            if enabled_modules.contains(EnabledModules::EXECUTION_TRACE) {
                Some(1)
            } else {
                None
            };

        let nonce = self.next_transaction_nonce();
        let test_transaction = TestTransaction::new_from_nonce(manifest, nonce);
        let prepared_transaction = test_transaction.prepare().unwrap();
        let executable =
            prepared_transaction.get_executable(Default::default());
        self.execute_transaction(executable, execution_config)
    }

    /// Constructs a notarized transaction and executes it. This is primarily
    /// used in the testing of fees to make sure that they're approximated in
    /// the best way.
    fn construct_and_execute_notarized_transaction(
        &mut self,
        manifest: TransactionManifestV1,
        notary_private_key: &PrivateKey,
    ) -> TransactionReceiptV1 {
        let network_definition = NetworkDefinition::simulator();
        let current_epoch = self.get_current_epoch();
        let transaction = TransactionBuilder::new()
            .header(TransactionHeaderV1 {
                network_id: network_definition.id,
                start_epoch_inclusive: current_epoch,
                end_epoch_exclusive: current_epoch.after(10).unwrap(),
                nonce: self.next_transaction_nonce(),
                notary_public_key: notary_private_key.public_key(),
                notary_is_signatory: true,
                tip_percentage: 0,
            })
            .manifest(manifest)
            .notarize(notary_private_key)
            .build();
        self.execute_notarized_transaction(&transaction.to_raw().unwrap())
    }

    fn blueprint_id(&self, node_id: impl Into<NodeId>) -> BlueprintId {
        SystemDatabaseReader::new(self.substate_db())
            .get_blueprint_id(&node_id.into(), ModuleId::Main)
            .unwrap()
    }
}

fn publishing_receipt() -> &'static PublishingReceipt {
    static PUBLISHING_RECEIPT: OnceLock<PublishingReceipt> = OnceLock::new();
    PUBLISHING_RECEIPT.get_or_init(|| PublishingReceipt {
        dapp_definition_account: component_address!(
            "account_rdx1cxh9jq27n5vllmsexah8jj3txzue8yu236uekcnfr4hq5ptw8nn7f0"
        ),
        packages: Entities {
            protocol_entities: ProtocolIndexedData {
                ignition: package_address!(
                    "package_rdx1pksyy7cyun85mgnuqdv4z3wm68d3pkfwzfkqrchhsu358zpjjuv426"
                ),
                simple_oracle: package_address!(
                    "package_rdx1phx0yptt32290n0uym4aqh3zyyup4ykth4enph8a68ggp7c38dqaxw"
                ),
            },
            exchange_adapter_entities: ExchangeIndexedData {
                ociswap_v2: package_address!(
                    "package_rdx1pknh02tzgdjk7fs9nxyckpdkjkz5jhcu87m78vajexurh99dk9yt22"
                ),
                defiplaza_v2: package_address!(
                    "package_rdx1p5q7uhr7kkrtr2ta6xl938txrk8r2cra02cpvf2le548jjrcsfvzkc"
                ),
                caviarnine_v1: package_address!(
                    "package_rdx1p5c0rcv7kwnjlyfpam5qfp0xnknz9rpdy0de7fhxj689mvfxdzj558"
                ),
            },
        },
        components: Entities {
            protocol_entities: ProtocolIndexedData {
                ignition: component_address!(
                    "component_rdx1cqplswlzpvw9yx687mcnvjuguy24veqk4c55rscjxl3pll7rxfs2dz"
                ),
                simple_oracle: component_address!(
                    "component_rdx1cr3psyfptwkktqusfg8ngtupr4wwfg32kz2xvh9tqh4c7pwkvlk2kn"
                ),
            },
            exchange_adapter_entities: ExchangeIndexedData {
                ociswap_v2: component_address!(
                    "component_rdx1cqrsdg6ag5urfe3av7d6z9q04emgjv726f48uhmzpex54jpwcxasq3"
                ),
                defiplaza_v2: component_address!(
                    "component_rdx1cr2asvvh7s02l4pzez8szp6kck4f230h8rkxmf56347hwje5gg7vtc"
                ),
                caviarnine_v1: component_address!(
                    "component_rdx1cpjs0phmgzwmhxel74l256zqdp39d2rfvj6m54e5k758k2vma8grp9"
                ),
            },
        },
        exchange_information: ExchangeIndexedData {
            ociswap_v2: Some(ExchangeInformation {
                blueprint_id: BlueprintId {
                    package_address: package_address!(
                        "package_rdx1pkrgvskdkglfd2ar4jkpw5r2tsptk85gap4hzr9h3qxw6ca40ts8dt"
                    ),
                    blueprint_name: "PrecisionPool".into(),
                },
                pools: UserResourceIndexedData {
                    bitcoin: component_address!(
                        "component_rdx1cpgmgrskahkxe4lnpp9s2f5ga0z8jkl7ne8gjmw3fc2224lxq505mr"
                    ),
                    ethereum: component_address!(
                        "component_rdx1crahf8qdh8fgm8mvzmq5w832h97q5099svufnqn26ue44fyezn7gnm"
                    ),
                    usdc: component_address!(
                        "component_rdx1cz8daq5nwmtdju4hj5rxud0ta26wf90sdk5r4nj9fqjcde5eht8p0f"
                    ),
                    usdt: component_address!(
                        "component_rdx1cz79xc57dpuhzd3wylnc88m3pyvfk7c5e03me2qv7x8wh9t6c3aw4g"
                    ),
                },
                liquidity_receipt: resource_address!(
                    "resource_rdx1ngeqqquzmjrd6q6atyawlh7p29jrpshdayw7rklyjw4n5k7ks6plm8"
                ),
            }),
            defiplaza_v2: Some(ExchangeInformation {
                blueprint_id: BlueprintId {
                    package_address: package_address!(
                        "package_rdx1p4dhfl7qwthqqu6p2267m5nedlqnzdvfxdl6q7h8g85dflx8n06p93"
                    ),
                    blueprint_name: "PlazaPair".into(),
                },
                pools: UserResourceIndexedData {
                    bitcoin: component_address!(
                        "component_rdx1czzqr5m40x3sklwntcmx8uw3ld5nj7marq66nm6erp3prw7rv8zu29"
                    ),
                    ethereum: component_address!(
                        "component_rdx1cr0nw5ppvryyqcv6thkslcltkw5cm3c2lvm2yr8jhh9rqe76stmars"
                    ),
                    usdc: component_address!(
                        "component_rdx1czmha58h7vw0e4qpxz8ga68cq6h5fjm27w2z43r0n6k9x65nvrjp4g"
                    ),
                    usdt: component_address!(
                        "component_rdx1crhrzxe6x35hwx3wmnnw0g8qs84p2hle6ud7n2q4ffzp0udluqm8hj"
                    ),
                },
                liquidity_receipt: resource_address!(
                    "resource_rdx1ntmgj3amlsrj0qxzqwzlk99d7g0xkzv6mg8vd5egawvgd8nt5ypwa7"
                ),
            }),
            caviarnine_v1: Some(ExchangeInformation {
                blueprint_id: BlueprintId {
                    package_address: package_address!(
                        "package_rdx1p4r9rkp0cq67wmlve544zgy0l45mswn6h798qdqm47x4762h383wa3"
                    ),
                    blueprint_name: "QuantaSwap".into(),
                },
                pools: UserResourceIndexedData {
                    bitcoin: component_address!(
                        "component_rdx1cp9w8443uyz2jtlaxnkcq84q5a5ndqpg05wgckzrnd3lgggpa080ed"
                    ),
                    ethereum: component_address!(
                        "component_rdx1cpsvw207842gafeyvf6tc0gdnq47u3mn74kvzszqlhc03lrns52v82"
                    ),
                    usdc: component_address!(
                        "component_rdx1cr6lxkr83gzhmyg4uxg49wkug5s4wwc3c7cgmhxuczxraa09a97wcu"
                    ),
                    usdt: component_address!(
                        "component_rdx1cqs338cyje65rk44zgmjvvy42qcszrhk9ewznedtkqd8l3crtgnmh5"
                    ),
                },
                liquidity_receipt: resource_address!(
                    "resource_rdx1n2uzpxdlg90ajqy9r597xkffeefhacl8hqd6kpvmfmt56wlda0dzk9"
                ),
            }),
        },
        protocol_configuration: ProtocolConfigurationReceipt {
            protocol_resource: XRD,
            user_resource_volatility: UserResourceIndexedData {
                bitcoin: Volatile,
                ethereum: Volatile,
                usdc: NonVolatile,
                usdt: NonVolatile,
            },
            reward_rates: indexmap! {
                LockupPeriod::from_months(9).unwrap() => dec!(0.125),
                LockupPeriod::from_months(10).unwrap() => dec!(0.145),
                LockupPeriod::from_months(11).unwrap() => dec!(0.17),
                LockupPeriod::from_months(12).unwrap() => dec!(0.2),
            },
            allow_opening_liquidity_positions: false,
            allow_closing_liquidity_positions: false,
            maximum_allowed_price_staleness_in_seconds: 60,
            maximum_allowed_price_difference_percentage: dec!(0.05),
            user_resources: UserResourceIndexedData {
                bitcoin: resource_address!(
                    "resource_rdx1t580qxc7upat7lww4l2c4jckacafjeudxj5wpjrrct0p3e82sq4y75"
                ),
                ethereum: resource_address!(
                    "resource_rdx1th88qcj5syl9ghka2g9l7tw497vy5x6zaatyvgfkwcfe8n9jt2npww"
                ),
                usdc: resource_address!(
                    "resource_rdx1t4upr78guuapv5ept7d7ptekk9mqhy605zgms33mcszen8l9fac8vf"
                ),
                usdt: resource_address!(
                    "resource_rdx1thrvr3xfs2tarm2dl9emvs26vjqxu6mqvfgvqjne940jv0lnrrg7rw"
                ),
            },
            registered_pools: ExchangeIndexedData {
                ociswap_v2: Some(UserResourceIndexedData {
                    bitcoin: component_address!(
                        "component_rdx1cpgmgrskahkxe4lnpp9s2f5ga0z8jkl7ne8gjmw3fc2224lxq505mr"
                    ),
                    ethereum: component_address!(
                        "component_rdx1crahf8qdh8fgm8mvzmq5w832h97q5099svufnqn26ue44fyezn7gnm"
                    ),
                    usdc: component_address!(
                        "component_rdx1cz8daq5nwmtdju4hj5rxud0ta26wf90sdk5r4nj9fqjcde5eht8p0f"
                    ),
                    usdt: component_address!(
                        "component_rdx1cz79xc57dpuhzd3wylnc88m3pyvfk7c5e03me2qv7x8wh9t6c3aw4g"
                    ),
                }),
                defiplaza_v2: Some(UserResourceIndexedData {
                    bitcoin: component_address!(
                        "component_rdx1czzqr5m40x3sklwntcmx8uw3ld5nj7marq66nm6erp3prw7rv8zu29"
                    ),
                    ethereum: component_address!(
                        "component_rdx1cr0nw5ppvryyqcv6thkslcltkw5cm3c2lvm2yr8jhh9rqe76stmars"
                    ),
                    usdc: component_address!(
                        "component_rdx1czmha58h7vw0e4qpxz8ga68cq6h5fjm27w2z43r0n6k9x65nvrjp4g"
                    ),
                    usdt: component_address!(
                        "component_rdx1crhrzxe6x35hwx3wmnnw0g8qs84p2hle6ud7n2q4ffzp0udluqm8hj"
                    ),
                }),
                caviarnine_v1: Some(UserResourceIndexedData {
                    bitcoin: component_address!(
                        "component_rdx1cp9w8443uyz2jtlaxnkcq84q5a5ndqpg05wgckzrnd3lgggpa080ed"
                    ),
                    ethereum: component_address!(
                        "component_rdx1cpsvw207842gafeyvf6tc0gdnq47u3mn74kvzszqlhc03lrns52v82"
                    ),
                    usdc: component_address!(
                        "component_rdx1cr6lxkr83gzhmyg4uxg49wkug5s4wwc3c7cgmhxuczxraa09a97wcu"
                    ),
                    usdt: component_address!(
                        "component_rdx1cqs338cyje65rk44zgmjvvy42qcszrhk9ewznedtkqd8l3crtgnmh5"
                    ),
                }),
            },
        },
        user_resources: UserResourceIndexedData {
            bitcoin: resource_address!(
                "resource_rdx1t580qxc7upat7lww4l2c4jckacafjeudxj5wpjrrct0p3e82sq4y75"
            ),
            ethereum: resource_address!(
                "resource_rdx1th88qcj5syl9ghka2g9l7tw497vy5x6zaatyvgfkwcfe8n9jt2npww"
            ),
            usdc: resource_address!(
                "resource_rdx1t4upr78guuapv5ept7d7ptekk9mqhy605zgms33mcszen8l9fac8vf"
            ),
            usdt: resource_address!(
                "resource_rdx1thrvr3xfs2tarm2dl9emvs26vjqxu6mqvfgvqjne940jv0lnrrg7rw"
            ),
        },
        badges: BadgeIndexedData {
            oracle_manager_badge: resource_address!(
                "resource_rdx1th3yr5dlydnhw0lfp6r22x5l2fj9lv3t8f0enkp7j5ttnx3e09rhna"
            ),
            protocol_owner_badge: resource_address!(
                "resource_rdx1t5ezhhs9cnua2thfnknmpj2rysz0rtwpexvjhvylww2ng5h3makwma"
            ),
            protocol_manager_badge: resource_address!(
                "resource_rdx1t5w3cekqxjcphrvtp8x5rqz55s4qk97ralrtldnlvf3t6nfhq9a4en"
            ),
        },
    })
}
