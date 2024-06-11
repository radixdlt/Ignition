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

#![allow(clippy::arithmetic_side_effects, clippy::too_many_arguments)]

use ignition::{InitializationParametersManifest, PoolBlueprintInformation};
use itertools::*;
use package_loader::*;
use radix_common::prelude::*;
use radix_engine::blueprints::package::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::prelude::*;
use radix_transactions::prelude::*;
use rand::prelude::*;

use super::*;
use crate::network_connection_provider::*;

pub fn publish<N: NetworkConnectionProvider>(
    configuration: &PublishingConfiguration,
    network_provider: &mut N,
) -> Result<
    PublishingReceipt,
    PublishingError<<N as NetworkConnectionProvider>::Error>,
> {
    // A cryptographically secure random number generator.
    let mut std_rng = rand::rngs::StdRng::from_entropy();

    // Creating an ephemeral private key to use for the publishing process. This
    // key will be mapped to an account that may store things during the process
    // but will ultimately be discarded in the end.
    let ephemeral_key_u64 = std_rng.next_u64();
    let ephemeral_private_key = PrivateKey::Ed25519(
        Ed25519PrivateKey::from_u64(ephemeral_key_u64).unwrap(),
    );
    let ephemeral_account = ComponentAddress::virtual_account_from_public_key(
        &ephemeral_private_key.public_key(),
    );
    log::info!("Ephemeral private key selected: {}", ephemeral_key_u64);

    // Finding the set of private keys to use for the signatures. This will be
    // the notary, the fee payer, and all of the private keys that control the
    // accounts with the badges.
    let mut signer_private_keys = vec![
        &configuration.transaction_configuration.notary,
        &ephemeral_private_key,
        &configuration
            .transaction_configuration
            .fee_payer_information
            .controlling_key,
    ];

    for badge_handling in configuration.badges.iter() {
        if let BadgeHandling::UseExisting {
            controlling_private_key,
            ..
        } = badge_handling
        {
            signer_private_keys.push(controlling_private_key)
        }
    }

    // Creating an execution service from the passed executor
    let mut execution_service = ExecutionService::new(
        network_provider,
        configuration
            .transaction_configuration
            .fee_payer_information
            .account_address,
        &configuration.transaction_configuration.notary,
        &signer_private_keys,
    );

    let dapp_definition_account = match configuration.dapp_definition {
        DappDefinitionHandling::UseExistingOneWayLink { component_address } => {
            component_address
        }

        // Creating the dApp definition account. The owner role will be set to
        // the ephemeral private key and then switched to the protocol owner and
        // manager at the end.
        DappDefinitionHandling::CreateNew { ref metadata } => {
            let manifest = ManifestBuilder::new()
                .allocate_global_address(
                    ACCOUNT_PACKAGE,
                    ACCOUNT_BLUEPRINT,
                    "reservation",
                    "named_address",
                )
                .then(|builder| {
                    let reservation =
                        builder.address_reservation("reservation");
                    let named_address = builder.named_address("named_address");

                    let mut builder = builder
                        .call_function(
                            ACCOUNT_PACKAGE,
                            ACCOUNT_BLUEPRINT,
                            ACCOUNT_CREATE_ADVANCED_IDENT,
                            AccountCreateAdvancedManifestInput {
                                address_reservation: Some(reservation),
                                owner_role: OwnerRole::Updatable(rule!(
                                    require(
                                        NonFungibleGlobalId::from_public_key(
                                            &ephemeral_private_key.public_key()
                                        )
                                    )
                                )),
                            },
                        )
                        .call_metadata_method(
                            named_address,
                            METADATA_SET_IDENT,
                            MetadataSetInput {
                                key: "account_type".to_owned(),
                                value: MetadataValue::String(
                                    "dapp definition".to_owned(),
                                ),
                            },
                        )
                        .call_metadata_method(
                            named_address,
                            METADATA_SET_IDENT,
                            MetadataSetInput {
                                key: "claimed_websites".to_owned(),
                                value: MetadataValue::OriginArray(vec![]),
                            },
                        )
                        .call_metadata_method(
                            named_address,
                            METADATA_SET_IDENT,
                            MetadataSetInput {
                                key: "dapp_definitions".to_owned(),
                                value: MetadataValue::GlobalAddressArray(
                                    vec![],
                                ),
                            },
                        );

                    for (key, value) in metadata.iter() {
                        builder = builder.call_metadata_method(
                            named_address,
                            METADATA_SET_IDENT,
                            MetadataSetInput {
                                key: key.to_owned(),
                                value: value.clone(),
                            },
                        )
                    }

                    builder
                })
                .build();

            execution_service
                .execute_manifest(manifest.clone())?
                .new_entities
                .new_component_addresses
                .first()
                .copied()
                .expect("Must succeed!")
        }
    };

    // Handling the creation of the user resources if they need to be created.
    let resolved_user_resources = {
        let user_resources_map = configuration.user_resources.into_map();

        let user_resources_already_created =
            user_resources_map.iter().flat_map(|(key, handling)| {
                if let UserResourceHandling::UseExisting { resource_address } =
                    handling
                {
                    Some((*key, resource_address))
                } else {
                    None
                }
            });
        let user_resources_requiring_creation = user_resources_map
            .iter()
            .flat_map(|(key, handling)| {
                if let UserResourceHandling::CreateFreelyMintableAndBurnable {
                    divisibility,
                    metadata,
                } = handling
                {
                    Some((*key, (divisibility, metadata)))
                } else {
                    None
                }
            })
            .collect::<IndexMap<&str, (&u8, &MetadataInit)>>();

        // Construct a manifest that creates the user resources.
        let manifest = TransactionManifestV1 {
            instructions: user_resources_requiring_creation
                .values()
                .map(|(divisibility, metadata)| InstructionV1::CallFunction {
                    package_address: RESOURCE_PACKAGE.into(),
                    blueprint_name: FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT
                        .to_string(),
                    function_name: FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT
                        .to_owned(),
                    args: to_manifest_value(
                        &FungibleResourceManagerCreateManifestInput {
                            owner_role: OwnerRole::None,
                            track_total_supply: true,
                            divisibility: **divisibility,
                            resource_roles: FungibleResourceRoles {
                                mint_roles: mint_roles! {
                                    minter => rule!(allow_all);
                                    minter_updater => rule!(deny_all);
                                },
                                burn_roles: burn_roles! {
                                    burner => rule!(allow_all);
                                    burner_updater => rule!(deny_all);
                                },
                                ..Default::default()
                            },
                            metadata: ModuleConfig {
                                init: (*metadata).clone(),
                                roles: Default::default(),
                            },
                            address_reservation: None,
                        },
                    )
                    .expect("Can't fail!"),
                })
                .collect::<Vec<_>>(),
            blobs: Default::default(),
        };
        let resource_addresses = execution_service
            .execute_manifest(manifest)?
            .new_entities
            .new_resource_addresses;

        UserResourceIndexedData::from_map(
            user_resources_already_created
                .map(|(key, address)| (key, *address))
                .chain(
                    user_resources_requiring_creation
                        .iter()
                        .map(|value| *value.0)
                        .zip(resource_addresses),
                ),
        )
        .expect("Can't fail!")
    };

    // Handling the badge creation that is needed.
    let resolved_badges = {
        let already_existing_badges =
            configuration.badges.into_map().into_iter().filter_map(
                |(key, value)| {
                    if let BadgeHandling::UseExisting {
                        holder_account_address,
                        badge_resource_address,
                        ..
                    } = value
                    {
                        Some((
                            key,
                            (*holder_account_address, *badge_resource_address),
                        ))
                    } else {
                        None
                    }
                },
            );

        let badges_requiring_creation = configuration
            .badges
            .into_map()
            .into_iter()
            .filter_map(|(key, value)| {
                if let BadgeHandling::CreateAndSend { metadata_init, .. } =
                    value
                {
                    Some((key, metadata_init))
                } else {
                    None
                }
            });

        let mut manifest_builder = ManifestBuilder::new();
        let mut keys = vec![];
        for (key, metadata_init) in badges_requiring_creation {
            let mut metadata_init = metadata_init.clone();
            metadata_init.data.insert(
                "dapp_definitions".to_owned(),
                KeyValueStoreInitEntry {
                    value: Some(MetadataValue::GlobalAddressArray(vec![
                        dapp_definition_account.into(),
                    ])),
                    lock: false,
                },
            );

            keys.push(key);
            manifest_builder = manifest_builder.create_fungible_resource(
                OwnerRole::Updatable(rule!(require(
                    NonFungibleGlobalId::from_public_key(
                        &ephemeral_private_key.public_key()
                    )
                ))),
                true,
                0,
                FungibleResourceRoles {
                    mint_roles: mint_roles! {
                        minter => rule!(deny_all);
                        minter_updater => rule!(deny_all);
                    },
                    burn_roles: burn_roles! {
                        burner => rule!(deny_all);
                        burner_updater => rule!(deny_all);
                    },
                    freeze_roles: freeze_roles! {
                        freezer => rule!(deny_all);
                        freezer_updater => rule!(deny_all);
                    },
                    recall_roles: recall_roles! {
                        recaller => rule!(deny_all);
                        recaller_updater => rule!(deny_all);
                    },
                    withdraw_roles: withdraw_roles! {
                        withdrawer => rule!(allow_all);
                        withdrawer_updater => rule!(deny_all);
                    },
                    deposit_roles: deposit_roles! {
                        depositor => rule!(allow_all);
                        depositor_updater => rule!(deny_all);
                    },
                },
                ModuleConfig {
                    roles: Default::default(),
                    init: metadata_init,
                },
                Some(dec!(1)),
            )
        }
        let manifest = manifest_builder
            .try_deposit_entire_worktop_or_abort(ephemeral_account, None)
            .build();
        let badges = keys
            .into_iter()
            .zip(
                execution_service
                    .execute_manifest(manifest.clone())?
                    .new_entities
                    .new_resource_addresses,
            )
            .map(|(key, resource_address)| {
                (key, (ephemeral_account, resource_address))
            });

        BadgeIndexedData::from_map(already_existing_badges.chain(badges))
            .expect("Can't fail")
    };

    let resolved_rules = resolved_badges
        .map(|(_, resource_address)| rule!(require(*resource_address)));

    // The resources created in the previous transaction have an updatable owner
    // role that is set to the ephemeral private key. In this transaction the
    // owner role is modified to be the protocol owner.
    {
        let mut manifest_builder = ManifestBuilder::new();
        for ((_, address), handling) in
            resolved_badges.zip_borrowed(&configuration.badges).iter()
        {
            if let BadgeHandling::CreateAndSend { .. } = handling {
                manifest_builder = manifest_builder
                    .create_proof_from_account_of_amount(
                        resolved_badges.protocol_owner_badge.0,
                        resolved_badges.protocol_owner_badge.1,
                        dec!(1),
                    )
                    .set_owner_role(
                        *address,
                        resolved_rules.protocol_owner_badge.clone(),
                    )
                    .lock_owner_role(*address);
            }
        }
        let manifest = manifest_builder.build();

        execution_service.execute_manifest(manifest.clone())?;
    }

    // Publishing the packages that need to be published
    let resolved_blueprint_ids = {
        let mut map = configuration.packages.protocol_entities.into_map();
        map.extend(configuration.packages.exchange_adapter_entities.into_map());

        let iterator = map
            .into_iter()
            .filter_map(|(key, package_handling)| {
                if let PackageHandling::LoadAndPublish {
                    crate_package_name,
                    metadata,
                    blueprint_name,
                } = package_handling
                {
                    Some((key, (crate_package_name, metadata, blueprint_name)))
                } else {
                    None
                }
            })
            .map(|(key, (crate_package_name, metadata, blueprint_name))| {
                let (code, definition) = PackageLoader::get(crate_package_name);

                let mut metadata = metadata.clone();
                metadata.data.insert(
                    "dapp_definition".to_owned(),
                    KeyValueStoreInitEntry {
                        value: Some(MetadataValue::GlobalAddress(
                            dapp_definition_account.into(),
                        )),
                        lock: false,
                    },
                );

                (key, (code, definition, metadata, blueprint_name.clone()))
            })
            .sorted_by(|x, y| x.1 .0.len().cmp(&y.1 .0.len()));

        // We want to get as many packages into one transaction. Goal is to
        // have each transaction be 980 kbs or less in size. If the addition
        // of a package increases the size beyond that then it goes in the
        // next batch.
        let mut batches = vec![Vec::<(
            String,
            (Vec<u8>, PackageDefinition, MetadataInit, String),
        )>::new()];
        for (key, (code, definition, metadata_init, blueprint_name)) in iterator
        {
            let latest_batch = batches.last_mut().expect("Impossible!");
            let total_code_size = latest_batch
                .iter()
                .map(|entry| entry.1 .0.len())
                .sum::<usize>();

            let size_if_code_is_added_to_batch = total_code_size + code.len();
            // Add to next batch
            if size_if_code_is_added_to_batch > 980 * 1024 {
                batches.push(vec![(
                    key.to_owned(),
                    (code, definition, metadata_init, blueprint_name),
                )])
            }
            // Add to this batch
            else {
                latest_batch.push((
                    key.to_owned(),
                    (code, definition, metadata_init, blueprint_name),
                ));
            }
        }

        // Creating transactions of the batches
        let mut blueprint_id_map = IndexMap::<String, BlueprintId>::new();
        for batch in batches {
            let mut manifest_builder = ManifestBuilder::new();
            for (_, (code, definition, metadata, _)) in batch.iter() {
                manifest_builder = manifest_builder.publish_package_advanced(
                    None,
                    code.clone(),
                    definition.clone(),
                    metadata.clone(),
                    OwnerRole::Fixed(
                        resolved_rules.protocol_owner_badge.clone(),
                    ),
                );
            }
            let manifest = manifest_builder.build();

            blueprint_id_map.extend(
                execution_service
                    .execute_manifest(manifest.clone())?
                    .new_entities
                    .new_package_addresses
                    .into_iter()
                    .zip(batch.into_iter())
                    .map(
                        |(
                            package_address,
                            (key, (_, _, _, blueprint_name)),
                        )| {
                            (
                                key,
                                BlueprintId {
                                    package_address,
                                    blueprint_name,
                                },
                            )
                        },
                    ),
            );
        }

        let blueprint_id_map = configuration
            .packages
            .protocol_entities
            .into_map()
            .into_iter()
            .chain(configuration.packages.exchange_adapter_entities.into_map())
            .filter_map(|(key, value)| {
                if let PackageHandling::UseExisting { blueprint_id } = value {
                    Some((key.to_owned(), blueprint_id.clone()))
                } else {
                    None
                }
            })
            .chain(blueprint_id_map)
            .collect::<IndexMap<_, _>>();

        Entities {
            protocol_entities: ProtocolIndexedData::from_map(
                blueprint_id_map.clone(),
            )
            .expect("Can't fail!"),
            exchange_adapter_entities: ExchangeIndexedData::from_map(
                blueprint_id_map,
            )
            .expect("Can't fail!"),
        }
    };

    // Computing the package global caller
    let resolved_package_global_caller_rules = Entities {
        protocol_entities: resolved_blueprint_ids.protocol_entities.map(
            |blueprint_id| {
                rule!(require(package_of_direct_caller(
                    blueprint_id.package_address
                )))
            },
        ),
        exchange_adapter_entities: resolved_blueprint_ids
            .exchange_adapter_entities
            .map(|blueprint_id| {
                rule!(require(package_of_direct_caller(
                    blueprint_id.package_address
                )))
            }),
    };

    let resolved_exchange_data = ExchangeIndexedData {
        caviarnine_v1: handle_caviarnine_v1_exchange_information(
            &mut execution_service,
            configuration.exchange_information.caviarnine_v1.as_ref(),
            dapp_definition_account,
            &resolved_rules,
            &resolved_package_global_caller_rules,
            &resolved_user_resources,
            configuration.protocol_configuration.protocol_resource,
        )?,
    };

    // Creating the adapter components of the various exchange packages that we
    // published.
    let resolved_adapter_component_addresses = {
        let adapter_instantiation_instructions = resolved_blueprint_ids
            .exchange_adapter_entities
            .clone()
            .zip(
                configuration
                    .protocol_configuration
                    .entities_metadata
                    .exchange_adapter_entities
                    .clone(),
            )
            .map(|(adapter_package, metadata_init)| {
                let mut metadata_init = metadata_init.clone();
                metadata_init.data.insert(
                    "dapp_definition".to_owned(),
                    KeyValueStoreInitEntry {
                        value: Some(MetadataValue::GlobalAddress(
                            dapp_definition_account.into(),
                        )),
                        lock: false,
                    },
                );

                InstructionV1::CallFunction {
                    package_address: adapter_package.package_address.into(),
                    blueprint_name: adapter_package.blueprint_name.clone(),
                    function_name: "instantiate".to_owned(),
                    args: to_manifest_value(&(
                        resolved_rules.protocol_manager_badge.clone(),
                        resolved_rules.protocol_owner_badge.clone(),
                        metadata_init,
                        OwnerRole::Fixed(
                            resolved_rules.protocol_owner_badge.clone(),
                        ),
                        None::<ManifestAddressReservation>,
                    ))
                    .expect("Impossible!"),
                }
            });

        let manifest = TransactionManifestV1 {
            instructions: adapter_instantiation_instructions
                .iter()
                .cloned()
                .collect(),
            blobs: Default::default(),
        };

        ExchangeIndexedData::from_map(
            adapter_instantiation_instructions
                .into_map()
                .into_iter()
                .zip(
                    execution_service
                        .execute_manifest(manifest)?
                        .new_entities
                        .new_component_addresses,
                )
                .map(|((key, _), component_address)| (key, component_address)),
        )
        .expect("Cant fail!")
    };

    // Instantiating the oracle component
    let oracle_component_address = match configuration.oracle_handling {
        OracleHandling::UseExisting { component_address } => component_address,
        OracleHandling::CreateNew => {
            let mut metadata_init = configuration
                .protocol_configuration
                .entities_metadata
                .protocol_entities
                .simple_oracle
                .clone();

            metadata_init.data.insert(
                "dapp_definition".to_owned(),
                KeyValueStoreInitEntry {
                    value: Some(MetadataValue::GlobalAddress(
                        dapp_definition_account.into(),
                    )),
                    lock: false,
                },
            );

            let manifest = ManifestBuilder::new()
                .call_function(
                    resolved_blueprint_ids
                        .protocol_entities
                        .simple_oracle
                        .package_address,
                    resolved_blueprint_ids
                        .protocol_entities
                        .simple_oracle
                        .blueprint_name
                        .clone(),
                    "instantiate",
                    (
                        resolved_rules.oracle_manager_badge.clone(),
                        metadata_init,
                        OwnerRole::Fixed(
                            resolved_rules.protocol_owner_badge.clone(),
                        ),
                        None::<ManifestAddressReservation>,
                    ),
                )
                .build();

            execution_service
                .execute_manifest(manifest)?
                .new_entities
                .new_component_addresses
                .first()
                .copied()
                .unwrap()
        }
    };

    // Instantiating the Ignition component
    let ignition_component_address = {
        let mut metadata_init = configuration
            .protocol_configuration
            .entities_metadata
            .protocol_entities
            .ignition
            .clone();

        metadata_init.data.insert(
            "dapp_definition".to_owned(),
            KeyValueStoreInitEntry {
                value: Some(MetadataValue::GlobalAddress(
                    dapp_definition_account.into(),
                )),
                lock: false,
            },
        );

        let mut ignition_initialization_parameters =
            InitializationParametersManifest {
                initial_pool_information: Some(
                    resolved_exchange_data
                        .clone()
                        .zip(resolved_adapter_component_addresses)
                        .iter()
                        .filter_map(
                            |(exchange_information, adapter_component)| {
                                exchange_information.as_ref().map(
                                    |exchange_information| {
                                        (
                                            exchange_information
                                                .blueprint_id
                                                .clone(),
                                            PoolBlueprintInformation {
                                                adapter: *adapter_component,
                                                allowed_pools:
                                                    exchange_information
                                                        .pools
                                                        .iter()
                                                        .copied()
                                                        .collect(),
                                                liquidity_receipt:
                                                    exchange_information
                                                        .liquidity_receipt,
                                            },
                                        )
                                    },
                                )
                            },
                        )
                        .collect(),
                ),
                initial_user_resource_volatility: Some(
                    resolved_user_resources
                        .zip(
                            configuration
                                .protocol_configuration
                                .user_resource_volatility,
                        )
                        .iter()
                        .map(|(address, volatility)| (*address, *volatility))
                        .collect(),
                ),
                initial_reward_rates: Some(
                    configuration.protocol_configuration.reward_rates.clone(),
                ),
                initial_volatile_protocol_resources: None,
                initial_non_volatile_protocol_resources: None,
                initial_is_open_position_enabled: Some(
                    configuration
                        .protocol_configuration
                        .allow_opening_liquidity_positions,
                ),
                initial_is_close_position_enabled: Some(
                    configuration
                        .protocol_configuration
                        .allow_closing_liquidity_positions,
                ),
            };

        let manifest = ManifestBuilder::new()
            /* Handle the funding */
            .then(|mut builder| {
                if let Some((funding_account, amount)) =
                    configuration.additional_settings.fund_ignition_volatile
                {
                    builder = builder
                        .withdraw_from_account(funding_account, XRD, amount)
                        .take_from_worktop(XRD, amount, "volatile");
                    ignition_initialization_parameters
                        .initial_volatile_protocol_resources =
                        Some(builder.bucket("volatile"));
                }
                if let Some((funding_account, amount)) =
                    configuration.additional_settings.fund_ignition_non_volatile
                {
                    builder = builder
                        .withdraw_from_account(funding_account, XRD, amount)
                        .take_from_worktop(XRD, amount, "non_volatile");
                    ignition_initialization_parameters
                        .initial_non_volatile_protocol_resources =
                        Some(builder.bucket("non_volatile"));
                }
                builder
            })
            /* Instantiating */
            .call_function(
                resolved_blueprint_ids
                    .protocol_entities
                    .ignition
                    .package_address,
                resolved_blueprint_ids
                    .protocol_entities
                    .ignition
                    .blueprint_name
                    .clone(),
                "instantiate",
                (
                    metadata_init,
                    OwnerRole::Fixed(
                        resolved_rules.protocol_owner_badge.clone(),
                    ),
                    resolved_rules.protocol_owner_badge.clone(),
                    resolved_rules.protocol_manager_badge.clone(),
                    configuration.protocol_configuration.protocol_resource,
                    oracle_component_address,
                    configuration
                        .protocol_configuration
                        .maximum_allowed_price_staleness_in_seconds,
                    configuration
                        .protocol_configuration
                        .maximum_allowed_price_difference_percentage,
                    ignition_initialization_parameters,
                    None::<ManifestAddressReservation>,
                ),
            )
            .build();
        execution_service
            .execute_manifest(manifest)?
            .new_entities
            .new_component_addresses
            .first()
            .copied()
            .unwrap()
    };

    let resolved_entity_component_addresses = Entities {
        protocol_entities: ProtocolIndexedData {
            ignition: ignition_component_address,
            simple_oracle: oracle_component_address,
        },
        exchange_adapter_entities: resolved_adapter_component_addresses,
    };

    // Caching the information of the Caviarnine pools
    {
        if let Some(ExchangeInformation { pools, .. }) =
            resolved_exchange_data.caviarnine_v1
        {
            let manifest = ManifestBuilder::new()
                .create_proof_from_account_of_amount(
                    resolved_badges.protocol_manager_badge.0,
                    resolved_badges.protocol_manager_badge.1,
                    dec!(1),
                )
                .then(|builder| {
                    pools.iter().fold(builder, |builder, pool| {
                        builder.call_method(
                            resolved_adapter_component_addresses.caviarnine_v1,
                            "preload_pool_information",
                            (pool,),
                        )
                    })
                })
                .then(|builder| {
                    if let Some(config) = configuration
                        .additional_settings
                        .configure_caviarnine_adapter_pool_configuration
                    {
                        pools.zip(config).iter().fold(builder, |builder, args| {
                            builder.call_method(
                                resolved_adapter_component_addresses.caviarnine_v1,
                                "upsert_pool_contribution_bin_configuration",
                                args,
                            )
                        })
                    } else {
                        builder
                    }
                })
                .build();
            execution_service.execute_manifest(manifest)?;
        }
    }

    // Depositing the created badges into their accounts.
    {
        let mut manifest_builder = ManifestBuilder::new();
        for ((current_holder_address, resource_address), handling) in
            resolved_badges.zip_borrowed(&configuration.badges).iter()
        {
            if let BadgeHandling::CreateAndSend {
                account_address: destination_account_address,
                ..
            } = handling
            {
                manifest_builder = manifest_builder
                    .withdraw_from_account(
                        *current_holder_address,
                        *resource_address,
                        dec!(1),
                    )
                    .try_deposit_entire_worktop_or_abort(
                        *destination_account_address,
                        None,
                    )
            }
        }
        let manifest = manifest_builder.build();
        execution_service.execute_manifest(manifest)?;
    }

    Ok(PublishingReceipt {
        dapp_definition_account,
        packages: Entities {
            protocol_entities: resolved_blueprint_ids
                .protocol_entities
                .map(|blueprint_id| blueprint_id.package_address),
            exchange_adapter_entities: resolved_blueprint_ids
                .exchange_adapter_entities
                .map(|blueprint_id| blueprint_id.package_address),
        },
        components: resolved_entity_component_addresses,
        exchange_information: resolved_exchange_data.clone(),
        protocol_configuration: ProtocolConfigurationReceipt {
            protocol_resource: configuration
                .protocol_configuration
                .protocol_resource,
            user_resource_volatility: configuration
                .protocol_configuration
                .user_resource_volatility,
            reward_rates: configuration
                .protocol_configuration
                .reward_rates
                .clone(),
            allow_opening_liquidity_positions: configuration
                .protocol_configuration
                .allow_opening_liquidity_positions,
            allow_closing_liquidity_positions: configuration
                .protocol_configuration
                .allow_closing_liquidity_positions,
            maximum_allowed_price_staleness_in_seconds: configuration
                .protocol_configuration
                .maximum_allowed_price_staleness_in_seconds,
            maximum_allowed_price_difference_percentage: configuration
                .protocol_configuration
                .maximum_allowed_price_difference_percentage,
            user_resources: resolved_user_resources,
            registered_pools: resolved_exchange_data.map(|information| {
                information.as_ref().map(|information| information.pools)
            }),
        },
        user_resources: resolved_user_resources,
        badges: resolved_badges.map(|(_, address)| *address),
    })
}

fn handle_caviarnine_v1_exchange_information<N: NetworkConnectionProvider>(
    execution_service: &mut ExecutionService<N>,
    exchange_information: Option<
        &ExchangeInformation<PoolHandling, LiquidityReceiptHandling>,
    >,
    dapp_definition: ComponentAddress,
    badge_rules: &BadgeIndexedData<AccessRule>,
    entity_package_caller_rules: &Entities<AccessRule>,
    user_resources: &UserResourceIndexedData<ResourceAddress>,
    protocol_resource: ResourceAddress,
) -> Result<
    Option<ExchangeInformation<ComponentAddress, ResourceAddress>>,
    ExecutionServiceError<<N as NetworkConnectionProvider>::Error>,
> {
    match exchange_information {
        Some(exchange_information) => {
            // Create the liquidity receipt if it needs to be created.
            let liquidity_receipt = match exchange_information.liquidity_receipt
            {
                LiquidityReceiptHandling::CreateNew {
                    ref non_fungible_schema,
                    ref metadata,
                } => handle_liquidity_receipt_creation(
                    execution_service,
                    non_fungible_schema,
                    metadata,
                    dapp_definition,
                    badge_rules,
                    entity_package_caller_rules,
                )?,
                LiquidityReceiptHandling::UseExisting { resource_address } => {
                    resource_address
                }
            };

            // Creating the liquidity pools that need to be created
            let pools =
                exchange_information.pools.zip(*user_resources).try_map(
                    |(pool_handling, user_resource_address)| -> Result<
                        ComponentAddress,
                        ExecutionServiceError<
                            <N as NetworkConnectionProvider>::Error,
                        >,
                    > {
                        match pool_handling {
                            PoolHandling::Create => {
                                let manifest = ManifestBuilder::new()
                                    .call_function(
                                        exchange_information
                                            .blueprint_id
                                            .package_address,
                                        exchange_information
                                            .blueprint_id
                                            .blueprint_name
                                            .clone(),
                                        "new",
                                        (
                                            rule!(allow_all),
                                            rule!(allow_all),
                                            user_resource_address,
                                            protocol_resource,
                                            100u32,
                                            None::<ManifestAddressReservation>,
                                        ),
                                    )
                                    .build();

                                Ok(execution_service
                                    .execute_manifest(manifest)?
                                    .new_entities
                                    .new_component_addresses
                                    .first()
                                    .copied()
                                    .unwrap())
                            }
                            PoolHandling::UseExisting { pool_address } => {
                                Ok(*pool_address)
                            }
                        }
                    },
                )?;

            Ok(Some(ExchangeInformation {
                blueprint_id: exchange_information.blueprint_id.clone(),
                pools,
                liquidity_receipt,
            }))
        }
        None => Ok(None),
    }
}

fn handle_liquidity_receipt_creation<N: NetworkConnectionProvider>(
    execution_service: &mut ExecutionService<N>,
    non_fungible_schema: &NonFungibleDataSchema,
    metadata_init: &MetadataInit,
    dapp_definition_account: ComponentAddress,
    badge_rules: &BadgeIndexedData<AccessRule>,
    entity_package_caller_rules: &Entities<AccessRule>,
) -> Result<
    ResourceAddress,
    ExecutionServiceError<<N as NetworkConnectionProvider>::Error>,
> {
    // Adding the dapp definition to the metadata
    let mut metadata_init = metadata_init.clone();
    metadata_init.data.insert(
        "dapp_definitions".to_owned(),
        KeyValueStoreInitEntry {
            value: Some(MetadataValue::GlobalAddressArray(vec![
                dapp_definition_account.into(),
            ])),
            lock: false,
        },
    );

    let manifest = ManifestBuilder::new()
        .call_function(
            RESOURCE_PACKAGE,
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT.to_owned(),
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_RUID_WITH_INITIAL_SUPPLY_IDENT.to_owned(),
            NonFungibleResourceManagerCreateRuidWithInitialSupplyManifestInput {
                owner_role: OwnerRole::Fixed(badge_rules.protocol_owner_badge.clone()),
                track_total_supply: true,
                non_fungible_schema: non_fungible_schema.clone(),
                entries: Default::default(),
                resource_roles: NonFungibleResourceRoles {
                    // Mintable and burnable by the Ignition package and the
                    // protocol owner can update who can do that.
                    mint_roles: mint_roles! {
                        minter => entity_package_caller_rules.protocol_entities.ignition.clone();
                        minter_updater => badge_rules.protocol_owner_badge.clone();
                    },
                    burn_roles: burn_roles! {
                        burner => entity_package_caller_rules.protocol_entities.ignition.clone();
                        burner_updater => badge_rules.protocol_owner_badge.clone();
                    },
                    // The protocol owner reserves the rights to update the data
                    // of the non-fungibles as they see fit.
                    non_fungible_data_update_roles: non_fungible_data_update_roles! {
                        non_fungible_data_updater => rule!(deny_all);
                        non_fungible_data_updater_updater => badge_rules.protocol_owner_badge.clone();
                    },
                    // Everything else is deny all and can't be changed.
                    recall_roles: recall_roles! {
                        recaller => rule!(deny_all);
                        recaller_updater => rule!(deny_all);
                    },
                    freeze_roles: freeze_roles! {
                        freezer => rule!(deny_all);
                        freezer_updater => rule!(deny_all);
                    },
                    deposit_roles: deposit_roles! {
                        depositor => rule!(allow_all);
                        depositor_updater => rule!(deny_all);
                    },
                    withdraw_roles: withdraw_roles! {
                        withdrawer => rule!(allow_all);
                        withdrawer_updater => rule!(deny_all);
                    },
                },
                metadata: ModuleConfig {
                    init: metadata_init,
                    roles: metadata_roles! {
                        metadata_setter => badge_rules.protocol_owner_badge.clone();
                        metadata_setter_updater => badge_rules.protocol_owner_badge.clone();
                        metadata_locker => badge_rules.protocol_owner_badge.clone();
                        metadata_locker_updater => badge_rules.protocol_owner_badge.clone();
                    },
                },
                address_reservation: None,
            },
        )
        .build();

    execution_service
        .execute_manifest(manifest)
        .map(|new_entities| {
            new_entities
                .new_entities
                .new_resource_addresses
                .first()
                .copied()
                .unwrap()
        })
}

#[derive(Debug)]
pub enum PublishingError<E> {
    NetworkConnectionProviderError(E),
    ExecutionServiceError(ExecutionServiceError<E>),
}

impl<E> From<E> for PublishingError<E> {
    fn from(value: E) -> Self {
        Self::NetworkConnectionProviderError(value)
    }
}

impl<E> From<ExecutionServiceError<E>> for PublishingError<E> {
    fn from(value: ExecutionServiceError<E>) -> Self {
        Self::ExecutionServiceError(value)
    }
}
