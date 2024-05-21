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

#![allow(clippy::arithmetic_side_effects)]

use address_macros::component_address;
use address_macros::resource_address;
use common::prelude::*;
use macro_rules_attribute::apply;
use publishing_tool::publishing::*;
use radix_common::prelude::*;
use radix_engine::blueprints::consensus_manager::*;
use radix_engine::blueprints::models::*;
use radix_engine::system::system_db_reader::*;
use radix_engine::system::system_modules::EnabledModules;
use radix_engine_interface::blueprints::consensus_manager::*;
use radix_engine_interface::prelude::*;
use radix_transactions::prelude::*;
use stateful_tests::*;

#[apply(mainnet_test)]
#[ignore = "Test fails because the testing tool creates a new C9 adapter."]
fn all_ignition_entities_are_linked_to_the_dapp_definition_in_accordance_with_the_metadata_standard(
    _: AccountAndControllingKey,
    receipt: &PublishingReceipt,
    ledger: &mut StatefulLedgerSimulator<'_>,
) {
    // Collecting all of the entities into an array
    let ignition_entities = receipt
        .badges
        .iter()
        .copied()
        .map(GlobalAddress::from)
        .chain(
            receipt
                .packages
                .protocol_entities
                .iter()
                .copied()
                .map(GlobalAddress::from),
        )
        .chain(
            receipt
                .packages
                .exchange_adapter_entities
                .iter()
                .copied()
                .map(GlobalAddress::from),
        )
        .chain(
            receipt
                .components
                .protocol_entities
                .iter()
                .copied()
                .map(GlobalAddress::from),
        )
        .chain(
            receipt
                .components
                .exchange_adapter_entities
                .iter()
                .copied()
                .map(GlobalAddress::from),
        )
        .chain(
            receipt
                .exchange_information
                .iter()
                .filter_map(|information| {
                    information
                        .as_ref()
                        .map(|information| information.liquidity_receipt)
                })
                .map(GlobalAddress::from),
        )
        .collect::<Vec<_>>();

    // Validating that the dapp definition account has the correct fields and
    // metadata values.
    let dapp_definition_account = receipt.dapp_definition_account;
    let Some(MetadataValue::String(dapp_definition_account_type)) =
        ledger.get_metadata(dapp_definition_account.into(), "account_type")
    else {
        panic!("Dapp definition account type either does not exist or isn't a string")
    };
    let Some(MetadataValue::GlobalAddressArray(
        dapp_definition_claimed_entities,
    )) =
        ledger.get_metadata(dapp_definition_account.into(), "claimed_entities")
    else {
        panic!("Dapp definition claimed entities either does not exist or is not an array")
    };
    assert_eq!(dapp_definition_account_type, "dapp definition");
    assert_eq!(
        dapp_definition_claimed_entities
            .into_iter()
            .collect::<HashSet<_>>(),
        ignition_entities.iter().copied().collect::<HashSet<_>>()
    );

    // Validating the dapp definition of components and packages. They have the
    // metadata field "dapp_definition" (not plural) and its just an address and
    // not an array.
    for entity_address in ignition_entities.iter().filter(|address| {
        PackageAddress::try_from(**address).is_ok()
            || ComponentAddress::try_from(**address).is_ok()
    }) {
        let Some(MetadataValue::GlobalAddress(linked_dapp_definition_account)) =
            ledger.get_metadata(*entity_address, "dapp_definition")
        else {
            panic!("Dapp definition key does not exist on package or component")
        };
        assert_eq!(
            linked_dapp_definition_account,
            dapp_definition_account.into()
        )
    }

    // Validating the dapp definition of resources. They have the metadata field
    // "dapp_definitions" (plural) and its an array of dapp definitions.
    for entity_address in ignition_entities
        .iter()
        .filter(|address| ResourceAddress::try_from(**address).is_ok())
    {
        let Some(MetadataValue::GlobalAddressArray(
            linked_dapp_definition_account,
        )) = ledger.get_metadata(*entity_address, "dapp_definitions")
        else {
            panic!(
                "Dapp definition key does not exist on resource: {}",
                entity_address.to_hex()
            )
        };
        assert_eq!(
            linked_dapp_definition_account.first().copied().unwrap(),
            dapp_definition_account.into()
        )
    }
}

macro_rules! define_open_and_close_liquidity_position_tests {
    (
        $(
            $exchange_ident: ident => [
                $(
                    $resource_ident: ident
                ),* $(,)?
            ]
        ),* $(,)?
    ) => {
        $(
            $(
                define_open_and_close_liquidity_position_tests!($exchange_ident, $resource_ident, 9);
                define_open_and_close_liquidity_position_tests!($exchange_ident, $resource_ident, 10);
                define_open_and_close_liquidity_position_tests!($exchange_ident, $resource_ident, 11);
                define_open_and_close_liquidity_position_tests!($exchange_ident, $resource_ident, 12);
            )*
        )*
    };
    (
        $exchange_ident: ident,
        $resource_ident: ident,
        $lockup_period: expr
    ) => {
        paste::paste! {
            #[apply(mainnet_test)]
            fn [< can_open_an_ignition_position_in_ $exchange_ident _ $resource_ident _pool_with_ $lockup_period _months_in_lock_up >](
                AccountAndControllingKey {
                    account_address: test_account,
                    controlling_key: test_account_private_key,
                }: AccountAndControllingKey,
                receipt: &PublishingReceipt,
                ledger: &mut StatefulLedgerSimulator<'_>,
            ) {
                // Arrange
                    let Some(ExchangeInformation { pools, .. }) =
                    receipt.exchange_information.$exchange_ident
                else {
                    panic!("No {} pools", stringify!($exchange_ident));
                };
                let pool = pools.$resource_ident;
                let user_resource = receipt.user_resources.$resource_ident;

                let current_epoch = ledger.get_current_epoch();

                ledger.execute_manifest_without_auth(ManifestBuilder::new()
                    .lock_fee(test_account, dec!(10))
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
                                receipt.components.protocol_entities.ignition,
                                "deposit_protocol_resources",
                                (volatile, Volatility::Volatile),
                            )
                            .call_method(
                                receipt.components.protocol_entities.ignition,
                                "deposit_protocol_resources",
                                (non_volatile, Volatility::NonVolatile),
                            )
                    })
                    .build()
                )
                .expect_commit_success();

                // Act
                let transaction = TransactionBuilder::new()
                    .header(TransactionHeaderV1 {
                        network_id: 0xf2,
                        start_epoch_inclusive: current_epoch,
                        end_epoch_exclusive: current_epoch.after(10).unwrap(),
                        nonce: ledger.next_transaction_nonce(),
                        notary_public_key: test_account_private_key.public_key(),
                        notary_is_signatory: true,
                        tip_percentage: 0,
                    })
                    .manifest(
                        ManifestBuilder::new()
                            .lock_fee(test_account, dec!(10))
                            .withdraw_from_account(test_account, user_resource, dec!(1))
                            .take_all_from_worktop(user_resource, "bucket")
                            .with_bucket("bucket", |builder, bucket| {
                                builder.call_method(
                                    receipt.components.protocol_entities.ignition,
                                    "open_liquidity_position",
                                    (bucket, pool, LockupPeriod::from_months($lockup_period).unwrap()),
                                )
                            })
                            .deposit_batch(test_account)
                            .build(),
                    )
                    .notarize(&test_account_private_key)
                    .build();
                let receipt = ledger.execute_notarized_transaction(
                    &transaction.to_raw().unwrap(),
                );

                // Assert
                receipt.expect_commit_success();
                println!(
                    "Opening a position in {} {} pool costs {} XRD in total with {} XRD in execution",
                    stringify!($exchange_ident),
                    stringify!($resource_ident),
                    receipt.fee_summary.total_cost(),
                    receipt.fee_summary.total_execution_cost_in_xrd
                );

            }

            #[apply(mainnet_test)]
            fn [< can_open_and_close_an_ignition_position_in_ $exchange_ident _ $resource_ident _pool_with_ $lockup_period _months_in_lock_up >](
                AccountAndControllingKey {
                    account_address: test_account,
                    controlling_key: test_account_private_key,
                }: AccountAndControllingKey,
                receipt: &PublishingReceipt,
                ledger: &mut StatefulLedgerSimulator<'_>,
            ) {
                // Arrange
                let Some(ExchangeInformation { pools, liquidity_receipt, .. }) =
                    receipt.exchange_information.$exchange_ident
                else {
                    panic!("No {} pools", stringify!($exchange_ident));
                };
                let pool = pools.$resource_ident;
                let user_resource = receipt.user_resources.$resource_ident;

                let current_epoch = ledger.get_current_epoch();

                ledger.execute_manifest_without_auth(ManifestBuilder::new()
                    .lock_fee(test_account, dec!(10))
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
                                receipt.components.protocol_entities.ignition,
                                "deposit_protocol_resources",
                                (volatile, Volatility::Volatile),
                            )
                            .call_method(
                                receipt.components.protocol_entities.ignition,
                                "deposit_protocol_resources",
                                (non_volatile, Volatility::NonVolatile),
                            )
                    })
                    .build()
                )
                .expect_commit_success();

                let transaction = TransactionBuilder::new()
                    .header(TransactionHeaderV1 {
                        network_id: 0xf2,
                        start_epoch_inclusive: current_epoch,
                        end_epoch_exclusive: current_epoch.after(10).unwrap(),
                        nonce: ledger.next_transaction_nonce(),
                        notary_public_key: test_account_private_key.public_key(),
                        notary_is_signatory: true,
                        tip_percentage: 0,
                    })
                    .manifest(
                        ManifestBuilder::new()
                            .lock_fee(test_account, dec!(10))
                            .withdraw_from_account(test_account, user_resource, dec!(1))
                            .take_all_from_worktop(user_resource, "bucket")
                            .with_bucket("bucket", |builder, bucket| {
                                builder.call_method(
                                    receipt.components.protocol_entities.ignition,
                                    "open_liquidity_position",
                                    (bucket, pool, LockupPeriod::from_months($lockup_period).unwrap()),
                                )
                            })
                            .deposit_batch(test_account)
                            .build(),
                    )
                    .notarize(&test_account_private_key)
                    .build();
                let transaction_receipt = ledger.execute_notarized_transaction(
                    &transaction.to_raw().unwrap(),
                );

                transaction_receipt.expect_commit_success();

                // Set the current time to be X months from now.
                {
                    let current_time =
                        ledger.get_current_time(TimePrecisionV2::Minute);
                    let maturity_instant = current_time
                        .add_seconds(
                            *LockupPeriod::from_months($lockup_period).unwrap().seconds() as i64
                        )
                        .unwrap();
                    let db = ledger.substate_db_mut();
                    let mut writer = SystemDatabaseWriter::new(db);

                    writer
                        .write_typed_object_field(
                            CONSENSUS_MANAGER.as_node_id(),
                            ModuleId::Main,
                            ConsensusManagerField::ProposerMilliTimestamp.field_index(),
                            ConsensusManagerProposerMilliTimestampFieldPayload::from_content_source(
                                ProposerMilliTimestampSubstate {
                                    epoch_milli: maturity_instant.seconds_since_unix_epoch * 1000,
                                },
                            ),
                        )
                        .unwrap();

                    writer
                        .write_typed_object_field(
                            CONSENSUS_MANAGER.as_node_id(),
                            ModuleId::Main,
                            ConsensusManagerField::ProposerMinuteTimestamp.field_index(),
                            ConsensusManagerProposerMinuteTimestampFieldPayload::from_content_source(
                                ProposerMinuteTimestampSubstate {
                                    epoch_minute: i32::try_from(
                                        maturity_instant.seconds_since_unix_epoch / 60,
                                    )
                                    .unwrap(),
                                },
                            ),
                        )
                        .unwrap();
                }

                {
                    let oracle = receipt.components.protocol_entities.simple_oracle;
                    let (price, _) = ledger
                        .execute_manifest_with_enabled_modules(
                            ManifestBuilder::new()
                                .call_method(
                                    oracle,
                                    "get_price",
                                    (user_resource, XRD),
                                )
                                .build(),
                            EnabledModules::for_notarized_transaction()
                                & !EnabledModules::AUTH
                                & !EnabledModules::COSTING,
                        )
                        .expect_commit_success()
                        .output::<(Decimal, Instant)>(0);
                    ledger
                        .execute_manifest_with_enabled_modules(
                            ManifestBuilder::new()
                                .call_method(
                                    oracle,
                                    "set_price",
                                    (user_resource, XRD, price),
                                )
                                .build(),
                            EnabledModules::for_notarized_transaction()
                                & !EnabledModules::AUTH
                                & !EnabledModules::COSTING,
                        )
                        .expect_commit_success();
                }

                let current_epoch = ledger.get_current_epoch();

                // Act
                let transaction = TransactionBuilder::new()
                    .header(TransactionHeaderV1 {
                        network_id: 0xf2,
                        start_epoch_inclusive: current_epoch,
                        end_epoch_exclusive: current_epoch.after(10).unwrap(),
                        nonce: ledger.next_transaction_nonce(),
                        notary_public_key: test_account_private_key.public_key().into(),
                        notary_is_signatory: true,
                        tip_percentage: 0,
                    })
                    .manifest(
                        ManifestBuilder::new()
                            .lock_fee(test_account, dec!(10))
                            .withdraw_from_account(
                                test_account,
                                liquidity_receipt,
                                dec!(1),
                            )
                            .take_all_from_worktop(
                                liquidity_receipt,
                                "bucket",
                            )
                            .with_bucket("bucket", |builder, bucket| {
                                builder.call_method(
                                    receipt.components.protocol_entities.ignition,
                                    "close_liquidity_position",
                                    (bucket,),
                                )
                            })
                            .deposit_batch(test_account)
                            .build(),
                    )
                    .notarize(&test_account_private_key)
                    .build();
                let receipt = ledger.execute_notarized_transaction(
                    &transaction.to_raw().unwrap(),
                );

                // Assert
                receipt.expect_commit_success();
                println!(
                    "Closing a position in {} {} pool costs {} XRD in total with {} XRD in execution",
                    stringify!($exchange_ident),
                    stringify!($resource_ident),
                    receipt.fee_summary.total_cost(),
                    receipt.fee_summary.total_execution_cost_in_xrd
                );
            }
        }
    };
}

define_open_and_close_liquidity_position_tests! {
    caviarnine_v1 => [
        bitcoin,
        ethereum,
        usdc,
        usdt
    ],
    defiplaza_v2 => [
        bitcoin,
        ethereum,
        usdc,
        usdt
    ],
    ociswap_v2 => [
        bitcoin,
        ethereum,
        usdc,
        usdt
    ]
}

#[apply(mainnet_test)]
fn log_reported_price_from_defiplaza_pool(
    _: AccountAndControllingKey,
    receipt: &PublishingReceipt,
    ledger: &mut StatefulLedgerSimulator<'_>,
) {
    let mut manifest_builder = ManifestBuilder::new();
    for pool in receipt
        .exchange_information
        .defiplaza_v2
        .as_ref()
        .unwrap()
        .pools
        .iter()
    {
        manifest_builder = manifest_builder.call_method(
            receipt.components.exchange_adapter_entities.defiplaza_v2,
            "price",
            (*pool,),
        )
    }
    let receipt = ledger.preview_manifest(
        manifest_builder.build(),
        vec![],
        0,
        PreviewFlags {
            use_free_credit: true,
            assume_all_signature_proofs: true,
            skip_epoch_check: true,
            disable_auth: true,
        },
    );
    receipt.expect_commit_success();
    for i in 0..4 {
        let price = receipt.expect_commit_success().output::<Price>(i);
        println!("{price:#?}");
    }
}

#[apply(mainnet_test)]
fn a_position_can_be_opened_and_closed_in_the_lsu_lp_pool(
    AccountAndControllingKey {
        account_address: test_account,
        controlling_key: test_account_private_key,
    }: AccountAndControllingKey,
    PublishingReceipt {
        components:
            Entities {
                protocol_entities: ProtocolIndexedData { simple_oracle, .. },
                ..
            },
        ..
    }: &PublishingReceipt,
    ledger: &mut StatefulLedgerSimulator<'_>,
) {
    // Arrange
    let pool_address = component_address!(
        "component_rdx1crdhl7gel57erzgpdz3l3vr64scslq4z7vd0xgna6vh5fq5fnn9xas"
    );
    let lsu_lp_resource = resource_address!(
        "resource_rdx1thksg5ng70g9mmy9ne7wz0sc7auzrrwy7fmgcxzel2gvp8pj0xxfmf"
    );
    let ignition_component = component_address!(
        "component_rdx1cqtpf3tah2u9h4tdj35fx7u3wku0j2y7xzsaxcc7nhvdenkaqhfg56"
    );
    let liquidity_receipt = resource_address!(
        "resource_rdx1nf0s0v9m8e2mcck3gyp6n4zudqx2yfdyhh6mj3xf3fkfrqtt0auw0p"
    );

    ledger
        .execute_manifest_without_auth(
            ManifestBuilder::new()
                .lock_fee(test_account, dec!(10))
                /* Submitting a price point to the oracle for the LSU LP */
                .call_method(*simple_oracle, "set_price", (lsu_lp_resource, XRD, dec!(1)))
                /* Fund Ignition */
                .mint_fungible(XRD, dec!(200_000_000_000_000))
                .take_from_worktop(XRD, dec!(100_000_000_000_000), "volatile")
                .take_from_worktop(XRD, dec!(100_000_000_000_000), "non_volatile")
                .with_name_lookup(|builder, _| {
                    let volatile = builder.bucket("volatile");
                    let non_volatile = builder.bucket("non_volatile");

                    builder
                        .call_method(
                            ignition_component,
                            "deposit_protocol_resources",
                            (volatile, Volatility::Volatile),
                        )
                        .call_method(
                            ignition_component,
                            "deposit_protocol_resources",
                            (non_volatile, Volatility::NonVolatile),
                        )
                })
                /* Start Ignition */
                .call_method(ignition_component, "set_is_open_position_enabled", (true,))
                .call_method(ignition_component, "set_is_close_position_enabled", (true,))
                /* Mint the LSU/LP resource. This is because we dont have LSUs */
                .mint_fungible(lsu_lp_resource, 100_000)
                /* Deposit the resources into the test account */
                .deposit_batch(test_account)
                .build(),
        )
        .expect_commit_success();

    let current_epoch = ledger.get_current_epoch();
    let transaction = TransactionBuilder::new()
        .header(TransactionHeaderV1 {
            network_id: 0xf2,
            start_epoch_inclusive: current_epoch,
            end_epoch_exclusive: current_epoch.after(10).unwrap(),
            nonce: ledger.next_transaction_nonce(),
            notary_public_key: test_account_private_key.public_key(),
            notary_is_signatory: true,
            tip_percentage: 0,
        })
        .manifest(
            ManifestBuilder::new()
                .lock_fee(test_account, dec!(10))
                .withdraw_from_account(test_account, lsu_lp_resource, 100_000)
                .take_all_from_worktop(lsu_lp_resource, "bucket")
                .with_bucket("bucket", |builder, bucket| {
                    builder.call_method(
                        ignition_component,
                        "open_liquidity_position",
                        (
                            bucket,
                            pool_address,
                            LockupPeriod::from_months(9).unwrap(),
                        ),
                    )
                })
                .deposit_batch(test_account)
                .build(),
        )
        .notarize(&test_account_private_key)
        .build();
    let tx_receipt =
        ledger.execute_notarized_transaction(&transaction.to_raw().unwrap());
    println!(
        "Opening a position in LSULP/XRD pool costs {} XRD in total with {} XRD in execution",
        tx_receipt.fee_summary.total_cost(),
        tx_receipt.fee_summary.total_execution_cost_in_xrd
    );

    ledger.push_time_forward(
        *LockupPeriod::from_months(9).unwrap().seconds() as _
    );

    ledger
        .execute_manifest_without_auth(
            ManifestBuilder::new()
                .lock_fee(test_account, dec!(10))
                .call_method(
                    *simple_oracle,
                    "set_price",
                    (lsu_lp_resource, XRD, dec!(1)),
                )
                .build(),
        )
        .expect_commit_success();

    let transaction = TransactionBuilder::new()
        .header(TransactionHeaderV1 {
            network_id: 0xf2,
            start_epoch_inclusive: current_epoch,
            end_epoch_exclusive: current_epoch.after(10).unwrap(),
            nonce: ledger.next_transaction_nonce(),
            notary_public_key: test_account_private_key.public_key(),
            notary_is_signatory: true,
            tip_percentage: 0,
        })
        .manifest(
            ManifestBuilder::new()
                .lock_fee(test_account, dec!(10))
                .withdraw_from_account(test_account, liquidity_receipt, dec!(1))
                .take_all_from_worktop(liquidity_receipt, "bucket")
                .with_bucket("bucket", |builder, bucket| {
                    builder.call_method(
                        ignition_component,
                        "close_liquidity_position",
                        (bucket,),
                    )
                })
                .deposit_batch(test_account)
                .build(),
        )
        .notarize(&test_account_private_key)
        .build();
    let receipt =
        ledger.execute_notarized_transaction(&transaction.to_raw().unwrap());

    // Assert
    receipt.expect_commit_success();
    println!(
        "Closing a position in LSULP/XRD pool costs {} XRD in total with {} XRD in execution",
        receipt.fee_summary.total_cost(),
        receipt.fee_summary.total_execution_cost_in_xrd
    );
}
