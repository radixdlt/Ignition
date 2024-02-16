//! This test module's purpose is to test Ignition's ability to open liquidity
//! positions in Caviarnine's pools that are already on mainnet and especially
//! if we can do so within the fee limit. Therefore, this test module creates
//! pools in the local test environment that are identical or close as possible
//! to the mainnet pools with the same amount of liquidity on both sides and
//! attempts to open positions against them. The information on how much is
//! current in the pool is obtained from the gateway. Therefore, this module
//! relies on the current state and can very much not be deterministic in some
//! cases on how much fees are required. However, it is the best way we have
//! found to test the C9 pools in a "real environment" and ensuring that what
//! we have works with C9.

use gateway_client::apis::configuration::Configuration as GatewayConfig;
use gateway_client::apis::transaction_api::*;
use gateway_client::models::*;
use tests::prelude::*;

#[test]
fn can_open_and_close_positions_to_all_mainnet_caviarnine_pools() {
    let ScryptoUnitEnv {
        environment: mut test_runner,
        resources,
        protocol,
        caviarnine_v1,
        ..
    } = ScryptoUnitEnv::new_with_configuration(Configuration {
        maximum_allowed_relative_price_difference: dec!(0.03),
        ..Default::default()
    });

    let (public_key, private_key, account) = test_runner.new_account(false);

    let pool_information = mainnet_state::pool_information(&resources);
    let pool_information = ResourceInformation {
        bitcoin: (pool_information.bitcoin, 8),
        ethereum: (pool_information.ethereum, 18),
        usdc: (pool_information.usdc, 6),
        usdt: (pool_information.usdt, 6),
    };

    for (
        mainnet_state::PoolInformation {
            resource_x,
            resource_y,
            active_tick,
            bin_span,
            bins_below,
            bins_above,
            price,
        },
        divisibility,
    ) in pool_information.iter()
    {
        let resource_address = if resource_x == XRD {
            resource_y
        } else {
            resource_x
        };

        let mut amount_in_bins = {
            let mut amount = indexmap! {};

            for (tick, amount_x) in bins_above {
                let (x, _) = amount.entry(tick).or_insert((dec!(0), dec!(0)));
                *x = amount_x + *x;
            }

            for (tick, amount_y) in bins_below {
                let (_, y) = amount.entry(tick).or_insert((dec!(0), dec!(0)));
                *y = amount_y + *y;
            }

            amount
        };

        // Creating a new pool with the same information as this provided pool.
        let pool_address = test_runner
            .execute_manifest(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .caviarnine_v1_pool_new(
                        caviarnine_v1.package,
                        rule!(deny_all),
                        rule!(allow_all),
                        resource_x,
                        resource_y,
                        bin_span,
                        None,
                    )
                    .build(),
                vec![],
            )
            .expect_commit_success()
            .new_component_addresses()
            .first()
            .copied()
            .unwrap();

        // Providing the liquidity to the pools.
        let (divisibility_x, divisibility_y) = if resource_x == XRD {
            (18, divisibility)
        } else {
            (divisibility, 18)
        };

        let amount_x = amount_in_bins
            .values()
            .map(|x| x.0)
            .reduce(|acc, item| acc + item)
            .unwrap_or_default();
        let amount_y = amount_in_bins
            .values()
            .map(|x| x.1)
            .reduce(|acc, item| acc + item)
            .unwrap_or_default();

        let amount_x = amount_x
            .checked_round(divisibility_x, RoundingMode::ToPositiveInfinity)
            .unwrap();
        let amount_y = amount_y
            .checked_round(divisibility_y, RoundingMode::ToPositiveInfinity)
            .unwrap();

        let active_amounts =
            amount_in_bins.remove(&active_tick.unwrap()).unwrap();
        let positions =
            vec![(active_tick.unwrap(), active_amounts.0, active_amounts.1)]
                .into_iter()
                .chain(amount_in_bins.into_iter().map(|(k, v)| {
                    (
                        k,
                        v.0.checked_round(divisibility_x, RoundingMode::ToZero)
                            .unwrap(),
                        v.1.checked_round(divisibility_y, RoundingMode::ToZero)
                            .unwrap(),
                    )
                }))
                .collect::<Vec<_>>();

        let price_in_simulated_pool = test_runner
            .execute_manifest_without_auth(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .mint_fungible(resource_x, amount_x)
                    .mint_fungible(resource_y, amount_y)
                    .take_all_from_worktop(resource_x, "resource_x")
                    .take_all_from_worktop(resource_y, "resource_y")
                    .with_bucket("resource_x", |builder, bucket_x| {
                        builder.with_bucket(
                            "resource_y",
                            |builder, bucket_y| {
                                builder.caviarnine_v1_pool_add_liquidity(
                                    pool_address,
                                    bucket_x,
                                    bucket_y,
                                    positions,
                                )
                            },
                        )
                    })
                    .deposit_batch(account)
                    .caviarnine_v1_pool_get_price(pool_address)
                    .build(),
            )
            .expect_commit_success()
            .output::<Option<Decimal>>(7)
            .unwrap();

        // If this assertion passes, then the pool we've created should be in
        // the same state as the mainnet one.
        assert_eq!(price_in_simulated_pool, price.unwrap());

        // Adding this pool to Ignition.
        test_runner
            .execute_manifest_without_auth(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .call_method(
                        protocol.ignition,
                        "add_allowed_pool",
                        (pool_address,),
                    )
                    .build(),
            )
            .expect_commit_success();

        // Updating the oracle price
        test_runner
            .execute_manifest_without_auth(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .call_method(
                        protocol.oracle,
                        "set_price",
                        (resource_x, resource_y, price_in_simulated_pool),
                    )
                    .call_method(
                        protocol.oracle,
                        "set_price",
                        (resource_y, resource_x, 1 / price_in_simulated_pool),
                    )
                    .build(),
            )
            .expect_commit_success();

        // Minting some of the resource and depositing them into the user's
        // account.
        test_runner
            .execute_manifest_without_auth(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .mint_fungible(resource_address, dec!(1000))
                    .deposit_batch(account)
                    .build(),
            )
            .expect_commit_success();

        // Cache the pool information - Note on this, the Caviarnine pools
        // literally require this and if the information is not cached we can
        // sometimes run out of cost units in the execution.
        test_runner
            .execute_manifest_without_auth(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .call_method(
                        caviarnine_v1.adapter,
                        "cache_pool_information",
                        (pool_address,),
                    )
                    .build(),
            )
            .expect_commit_success();

        // Constructing a transaction that is as close as possible to a real one
        // to open a liquidity position and ensure that we can open one with the
        // fee limit that we currently have.
        let current_epoch = test_runner.get_current_epoch();
        let transaction = TransactionBuilder::new()
            .header(TransactionHeaderV1 {
                network_id: 0xf2,
                start_epoch_inclusive: current_epoch,
                end_epoch_exclusive: current_epoch.after(10).unwrap(),
                nonce: test_runner.next_transaction_nonce(),
                notary_public_key: public_key.into(),
                notary_is_signatory: true,
                tip_percentage: 0,
            })
            .manifest(
                ManifestBuilder::new()
                    .lock_fee(account, dec!(10))
                    .withdraw_from_account(
                        account,
                        resource_address,
                        dec!(1000),
                    )
                    .take_all_from_worktop(resource_address, "bucket")
                    .with_bucket("bucket", |builder, bucket| {
                        builder.call_method(
                            protocol.ignition,
                            "open_liquidity_position",
                            (
                                bucket,
                                pool_address,
                                LockupPeriod::from_months(6).unwrap(),
                            ),
                        )
                    })
                    .deposit_batch(account)
                    .build(),
            )
            .notarize(&private_key)
            .build();
        let receipt = test_runner.execute_raw_transaction(
            &NetworkDefinition::simulator(),
            &transaction.to_raw().unwrap(),
        );
        receipt.expect_commit_success();
        println!(
            "Opening a position costs {} XRD in total with {} XRD in execution",
            receipt.fee_summary.total_cost(),
            receipt.fee_summary.total_execution_cost_in_xrd
        );

        // Set the current time to be 6 months from now.
        {
            let current_time =
                test_runner.get_current_time(TimePrecisionV2::Minute);
            let maturity_instant = current_time
                .add_seconds(
                    *LockupPeriod::from_months(6).unwrap().seconds() as i64
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

        test_runner
            .execute_manifest_without_auth(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .call_method(
                        protocol.oracle,
                        "set_price",
                        (resource_x, resource_y, price_in_simulated_pool),
                    )
                    .call_method(
                        protocol.oracle,
                        "set_price",
                        (resource_y, resource_x, 1 / price_in_simulated_pool),
                    )
                    .build(),
            )
            .expect_commit_success();

        // Close the liquidity position.
        let current_epoch = test_runner.get_current_epoch();
        let transaction = TransactionBuilder::new()
            .header(TransactionHeaderV1 {
                network_id: 0xf2,
                start_epoch_inclusive: current_epoch,
                end_epoch_exclusive: current_epoch.after(10).unwrap(),
                nonce: test_runner.next_transaction_nonce(),
                notary_public_key: public_key.into(),
                notary_is_signatory: true,
                tip_percentage: 0,
            })
            .manifest(
                ManifestBuilder::new()
                    .lock_fee(account, dec!(10))
                    .withdraw_from_account(
                        account,
                        caviarnine_v1.liquidity_receipt,
                        dec!(1),
                    )
                    .take_all_from_worktop(
                        caviarnine_v1.liquidity_receipt,
                        "bucket",
                    )
                    .with_bucket("bucket", |builder, bucket| {
                        builder.call_method(
                            protocol.ignition,
                            "close_liquidity_position",
                            (bucket,),
                        )
                    })
                    .deposit_batch(account)
                    .build(),
            )
            .notarize(&private_key)
            .build();
        let receipt = test_runner.execute_raw_transaction(
            &NetworkDefinition::simulator(),
            &transaction.to_raw().unwrap(),
        );
        receipt.expect_commit_success();
        println!(
            "Closing a position costs {} XRD in total with {} XRD in execution",
            receipt.fee_summary.total_cost(),
            receipt.fee_summary.total_execution_cost_in_xrd
        );
    }
}

mod mainnet_state {
    use super::*;
    use std::sync::*;

    static POOL_INFORMATION: Mutex<
        OnceCell<ResourceInformation<PoolInformation>>,
    > = Mutex::new(OnceCell::new());

    /// The function that users call to get the pool information. This hides the
    /// details of the mutex, once cell, and all of this information.
    pub fn pool_information(
        resources_addresses: &ResourceInformation<ResourceAddress>,
    ) -> ResourceInformation<PoolInformation> {
        POOL_INFORMATION
            .lock()
            .unwrap()
            .get_or_init(|| init_pool_information(resources_addresses))
            .clone()
    }

    /// Calls the gateway getting the information of the pools of interest.
    fn init_pool_information(
        resources_addresses: &ResourceInformation<ResourceAddress>,
    ) -> ResourceInformation<PoolInformation> {
        let gateway_configuration = GatewayConfig {
            base_path: "https://mainnet.radixdlt.com/".to_owned(),
            ..Default::default()
        };

        let network_definition = NetworkDefinition::mainnet();
        let decoder = AddressBech32Decoder::new(&network_definition);

        let pools = ResourceInformation {
            bitcoin: "component_rdx1cr4nrgchhqe9etjmyl6cvefc9mjpyxlu72xt0l0hdfjw3tm4z8esln",
            ethereum: "component_rdx1crennqxtn9axwfj4juccy9le0jw6m0fuyzdfu7vs5834f9nwtk5352",
            usdc: "component_rdx1czg0xynqq0kgfh9n4lpjtw2dtjxczdregez8vtwht6x3h0v9jzxg70",
            usdt: "component_rdx1czaa66y5nal99hsqwj3vkcvdv00q8g8dtrxjy82rfcj9g4pffxc4r4",
        };
        let pools = pools.map(|item| {
            ComponentAddress::try_from_bech32(&decoder, item).unwrap()
        });

        pools.zip(*resources_addresses).map(
            |(component_address, resource_address)| {
                // Doing a preview to get the information we need about the pool
                // like the amounts, the bin span, etc...
                let manifest = ManifestBuilder::new()
                    .caviarnine_v1_pool_get_token_x_address(*component_address)
                    .caviarnine_v1_pool_get_token_y_address(*component_address)
                    .caviarnine_v1_pool_get_active_tick(*component_address)
                    .caviarnine_v1_pool_get_bin_span(*component_address)
                    .caviarnine_v1_pool_get_bins_below(
                        *component_address,
                        None,
                        None,
                        None,
                    )
                    .caviarnine_v1_pool_get_bins_above(
                        *component_address,
                        None,
                        None,
                        None,
                    )
                    .caviarnine_v1_pool_get_price(*component_address)
                    .build();

                let preview_response = transaction_preview(
                    &gateway_configuration,
                    TransactionPreviewRequest {
                        manifest: transaction::manifest::decompile(
                            &manifest.instructions,
                            &network_definition,
                        )
                        .unwrap(),
                        blobs_hex: None,
                        start_epoch_inclusive: 200,
                        end_epoch_exclusive: 300,
                        notary_public_key: None,
                        notary_is_signatory: None,
                        tip_percentage: 0,
                        nonce: 12,
                        signer_public_keys: vec![],
                        flags: Box::new(TransactionPreviewRequestFlags {
                            assume_all_signature_proofs: true,
                            skip_epoch_check: true,
                            use_free_credit: true,
                        }),
                    },
                )
                .unwrap();

                let receipt = scrypto_decode::<VersionedTransactionReceipt>(
                    &preview_response.encoded_receipt,
                )
                .unwrap()
                .into_latest();

                let commit_result = receipt.expect_commit_success();

                let resource_x = commit_result.output::<ResourceAddress>(0);
                let _ = commit_result.output::<ResourceAddress>(1);
                let active_tick = commit_result.output::<Option<u32>>(2);
                let bin_span = commit_result.output::<u32>(3);
                let bins_below = commit_result.output::<Vec<(u32, Decimal)>>(4);
                let bins_above = commit_result.output::<Vec<(u32, Decimal)>>(5);
                let price = commit_result.output::<Option<Decimal>>(6);

                // It is guaranteed that one of the resources is XRD and the
                // other is the other resource. So, we change them here.
                let (resource_x, resource_y) = if resource_x == XRD {
                    (XRD, *resource_address)
                } else {
                    (*resource_address, XRD)
                };

                PoolInformation {
                    resource_x,
                    resource_y,
                    active_tick,
                    bin_span,
                    bins_below,
                    bins_above,
                    price,
                }
            },
        )
    }

    #[derive(Clone, ScryptoSbor, Debug)]
    pub struct PoolInformation {
        pub resource_x: ResourceAddress,
        pub resource_y: ResourceAddress,
        pub active_tick: Option<u32>,
        pub bin_span: u32,
        pub bins_below: Vec<(u32, Decimal)>,
        pub bins_above: Vec<(u32, Decimal)>,
        pub price: Option<Decimal>,
    }
}