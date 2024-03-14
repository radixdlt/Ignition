#![allow(clippy::arithmetic_side_effects)]

use common::prelude::*;
use macro_rules_attribute::apply;
use publishing_tool::publishing::*;
use radix_engine::blueprints::consensus_manager::*;
use radix_engine::blueprints::models::*;
use radix_engine::system::system_db_reader::*;
use radix_engine::system::system_modules::EnabledModules;
use radix_engine::types::*;
use radix_engine_interface::blueprints::consensus_manager::*;
use stateful_tests::*;
use transaction::prelude::*;

#[apply(mainnet_test)]
fn all_ignition_entities_are_linked_to_the_dapp_definition_in_accordance_with_the_metadata_standard(
    _: AccountAndControllingKey,
    receipt: &PublishingReceipt,
    test_runner: &mut StatefulTestRunner<'_>,
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
    let Some(MetadataValue::String(dapp_definition_account_type)) = test_runner
        .get_metadata(dapp_definition_account.into(), "account_type")
    else {
        panic!("Dapp definition account type either does not exist or isn't a string")
    };
    let Some(MetadataValue::GlobalAddressArray(
        dapp_definition_claimed_entities,
    )) = test_runner
        .get_metadata(dapp_definition_account.into(), "claimed_entities")
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
            test_runner.get_metadata(*entity_address, "dapp_definition")
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
        )) = test_runner.get_metadata(*entity_address, "dapp_definitions")
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
                test_runner: &mut StatefulTestRunner<'_>,
            ) {
                // Arrange
                    let Some(ExchangeInformation { pools, .. }) =
                    receipt.exchange_information.$exchange_ident
                else {
                    panic!("No {} pools", stringify!($exchange_ident));
                };
                let pool = pools.$resource_ident;
                let user_resource = receipt.user_resources.$resource_ident;

                let current_epoch = test_runner.get_current_epoch();

                // Act
                let transaction = TransactionBuilder::new()
                    .header(TransactionHeaderV1 {
                        network_id: 1,
                        start_epoch_inclusive: current_epoch,
                        end_epoch_exclusive: current_epoch.after(10).unwrap(),
                        nonce: test_runner.next_transaction_nonce(),
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
                let receipt = test_runner.execute_raw_transaction(
                    &NetworkDefinition::mainnet(),
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
                test_runner: &mut StatefulTestRunner<'_>,
            ) {
                // Arrange
                    let Some(ExchangeInformation { pools, liquidity_receipt, .. }) =
                    receipt.exchange_information.$exchange_ident
                else {
                    panic!("No {} pools", stringify!($exchange_ident));
                };
                let pool = pools.$resource_ident;
                let user_resource = receipt.user_resources.$resource_ident;

                let current_epoch = test_runner.get_current_epoch();

                let transaction = TransactionBuilder::new()
                    .header(TransactionHeaderV1 {
                        network_id: 1,
                        start_epoch_inclusive: current_epoch,
                        end_epoch_exclusive: current_epoch.after(10).unwrap(),
                        nonce: test_runner.next_transaction_nonce(),
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
                let transaction_receipt = test_runner.execute_raw_transaction(
                    &NetworkDefinition::mainnet(),
                    &transaction.to_raw().unwrap(),
                );

                transaction_receipt.expect_commit_success();

                // Set the current time to be 6 months from now.
                {
                    let current_time =
                        test_runner.get_current_time(TimePrecisionV2::Minute);
                    let maturity_instant = current_time
                        .add_seconds(
                            *LockupPeriod::from_months($lockup_period).unwrap().seconds() as i64
                        )
                        .unwrap();
                    let db = test_runner.substate_db_mut();
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
                    let (price, _) = test_runner
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
                    test_runner
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

                let current_epoch = test_runner.get_current_epoch();

                // Act
                let transaction = TransactionBuilder::new()
                    .header(TransactionHeaderV1 {
                        network_id: 1,
                        start_epoch_inclusive: current_epoch,
                        end_epoch_exclusive: current_epoch.after(10).unwrap(),
                        nonce: test_runner.next_transaction_nonce(),
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
                let receipt = test_runner.execute_raw_transaction(
                    &NetworkDefinition::mainnet(),
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
    test_runner: &mut StatefulTestRunner<'_>,
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
    let receipt = test_runner.preview_manifest(
        manifest_builder.build(),
        vec![],
        0,
        PreviewFlags {
            use_free_credit: true,
            assume_all_signature_proofs: true,
            skip_epoch_check: true,
        },
    );
    receipt.expect_commit_success();
    for i in (0..4) {
        let price = receipt.expect_commit_success().output::<Price>(i);
        println!("{price:#?}");
    }
}
