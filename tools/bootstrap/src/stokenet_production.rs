use crate::error::*;
use crate::transaction_service::*;
use crate::types::*;
use crate::*;
use clap::Parser;
use common::prelude::*;
use ignition::{InitializationParametersManifest, PoolBlueprintInformation};
use package_loader::PackageLoader;
use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::node_modules::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::prelude::*;
use transaction::prelude::*;

const PRIVATE_KEY_ENVIRONMENT_VARIABLE: &str = "PRIVATE_KEY";

#[derive(Parser, Debug)]
pub struct StokenetProduction {}

impl StokenetProduction {
    pub fn run<O: std::io::Write>(self, _: &mut O) -> Result<(), Error> {
        // Loading the private key that will notarize and pay the fees of the
        // transaction.
        let notary_private_key = {
            std::env::var(PRIVATE_KEY_ENVIRONMENT_VARIABLE)
                .map_err(|_| Error::FailedToLoadPrivateKey)
                .and_then(|hex| {
                    hex::decode(hex).map_err(|_| Error::FailedToLoadPrivateKey)
                })
                .and_then(|bytes| {
                    Ed25519PrivateKey::from_bytes(&bytes)
                        .map_err(|_| Error::FailedToLoadPrivateKey)
                })
                .map(PrivateKey::Ed25519)
        }?;
        let notary_account = ComponentAddress::virtual_account_from_public_key(
            &notary_private_key.public_key(),
        );
        let fee_handling = FeeHandling::EstimateAndLock {
            fee_payer_account: notary_account,
            fee_payer_private_key: &notary_private_key,
        };

        // Initializing all of the data that this command will use. These are
        // pretty much constants but we can't make them constants because most
        // of the functions are not `const`. There is also not really a point
        // in making them a lazy static, let's keep things simple.

        /* cSpell:disable - Sorry for this, I dislike it too. */
        const GATEWAY_API_BASE_URL: &str = "https://stokenet.radixdlt.com/";
        let network_definition = NetworkDefinition::stokenet();
        let bech32m_coders =
            Bech32mCoders::from_network_definition(&network_definition);

        // TODO: What do we want these values to be?
        const MAXIMUM_ALLOWED_PRICE_STALENESS_IN_SECONDS: i64 = 60; // 60 seconds
        const MAXIMUM_ALLOWED_PRICE_DIFFERENCE_PERCENTAGE: Decimal = dec!(0.05); // 5 %

        let protocol_resource = resource_address!("resource_tdx_2_1thwmtk9qet08y3wpujd8nddmvjuqyptg5nt0mw0zcdgcrahu5k36qx");
        let resources = NameIndexedResourceInformation {
            bitcoin: resource_address!("resource_tdx_2_1thltk578jr4v7axqpu5ceznhlha6ca2qtzcflqdmytgtf37xncu7l9"),
            ethereum: resource_address!("resource_tdx_2_1t59gx963vzd6u6fz63h5de2zh9nmgwxc8y832edmr6pxvz98wg6zu3"),
            usdc: resource_address!("resource_tdx_2_1thfv477eqwlh8x4wt6xsc62myt4z0zxmdpr4ea74fa8jnxh243y60r"),
            usdt: resource_address!("resource_tdx_2_1t4p3ytx933n576pdps4ua7jkjh36zrh36a543u0tfcsu2vthavlqg8"),
        };
        let exchanges = NameIndexedDexInformation {
            caviarnine_v1: DexInformation {
                package: package_address!("package_tdx_2_1p57g523zj736u370z6g4ynrytn7t6r2hledvzkhl6tzpg3urn0707e"),
                pools: NameIndexedResourceInformation {
                    bitcoin: component_address!("component_tdx_2_1czt59vxdqg7q4l0gzphmt5ev6lagl2cu6sm2hsaz9y8ypcf0aukf8r"),
                    ethereum: component_address!("component_tdx_2_1crqpgnpf3smh7kg8d4sz4h3502l65s4tslwhg46ru07ra6l30pcsj4"),
                    usdc: component_address!("component_tdx_2_1cpwkf9uhel3ut4ydm58g0uyaw7sxckmp2pz7sdv79vzt9y3p7ad4fu"),
                    usdt: component_address!("component_tdx_2_1czmdhtq0u8f40khky4c6j74msskuz60yq3y0zewu85phrdj0ryz2hl")
                }
            },
            // TODO: Ths following is INCORRECT INFORMATION! There is no Ociswap
            // package on Stokenet.
            ociswap_v1: DexInformation {
                package: package_address!("package_tdx_2_1p40dekel26tp2a2srma4sc3lj2ukr6y8k4amr7x8yav86lyyeg7ta7"),
                pools: NameIndexedResourceInformation {
                    bitcoin: component_address!("component_tdx_2_1czt59vxdqg7q4l0gzphmt5ev6lagl2cu6sm2hsaz9y8ypcf0aukf8r"),
                    ethereum: component_address!("component_tdx_2_1crqpgnpf3smh7kg8d4sz4h3502l65s4tslwhg46ru07ra6l30pcsj4"),
                    usdc: component_address!("component_tdx_2_1cpwkf9uhel3ut4ydm58g0uyaw7sxckmp2pz7sdv79vzt9y3p7ad4fu"),
                    usdt: component_address!("component_tdx_2_1czmdhtq0u8f40khky4c6j74msskuz60yq3y0zewu85phrdj0ryz2hl")
                }
            },
        };
        // TODO: Numbers here are not real and I have added from just to get
        // things going. MUST modify before launch.
        let reward_information = indexmap! {
            LockupPeriod::from_months(9).unwrap() => dec!(0.125),  // 12.5%
            LockupPeriod::from_months(10).unwrap() => dec!(0.15),  // 15.0%
            LockupPeriod::from_months(11).unwrap() => dec!(0.175), // 17.5%
            LockupPeriod::from_months(12).unwrap() => dec!(0.20),  // 20.0%
        };

        // TODO: MUST determine what those accounts are prior to launch!
        // For now they are MY stokenet accounts!
        let protocol_manager_account = component_address!("account_tdx_2_12xxuglkrdgcphpqk34fv59ewq3gu5uwlzs42hpy0grsrefvgwgxrev");
        let protocol_owner_account = component_address!("account_tdx_2_12xxuglkrdgcphpqk34fv59ewq3gu5uwlzs42hpy0grsrefvgwgxrev");

        /* cSpell:enable */

        // An ephemeral private key that we will use the bootstrapping process.
        // This key will initially control the dApp definition to allow us to
        // easily update the metadata and will later on change the owner role
        // of the dApp definition to the protocol owner.
        let ephemeral_private_key = PrivateKey::Ed25519(
            Ed25519PrivateKey::from_u64(rand::random()).unwrap(),
        );
        let ephemeral_virtual_signature_badge =
            NonFungibleGlobalId::from_public_key(
                &ephemeral_private_key.public_key(),
            );

        // This is the transaction service that the submission will happen
        // through. It does most of the heavy lifting associated with the
        // transaction submission.
        let transaction_service =
            TransactionService::new(&bech32m_coders, GATEWAY_API_BASE_URL);

        // Creating the dApp definition account. When this account starts it
        // its owner will be a virtual signature badge which will change once
        // add all of the metadata fields that we want to add. The the manifest
        // that involves the dApp definition will set the metadata on it and
        // will also change its owner to be the protocol Owner badge.
        let dapp_definition_account = {
            let manifest = ManifestBuilder::new()
                .call_function(
                    ACCOUNT_PACKAGE,
                    ACCOUNT_BLUEPRINT,
                    ACCOUNT_CREATE_ADVANCED_IDENT,
                    AccountCreateAdvancedManifestInput {
                        owner_role: OwnerRole::Updatable(rule!(require(
                            ephemeral_virtual_signature_badge
                        ))),
                        address_reservation: None,
                    },
                )
                .build();
            std::thread::sleep(std::time::Duration::from_secs(5));
            *transaction_service
                .submit_manifest(manifest, &notary_private_key, &fee_handling)?
                .new_component_addresses
                .first()
                .expect("Must succeed!")
        };

        // Creating the protocol owner and the protocol manager badges and
        // sending them off to the accounts specified up above.
        let (protocol_manager_badge, protocol_owner_badge) = {
            let manifest = ManifestBuilder::new()
                // The protocol manager badge
                .create_fungible_resource(
                    OwnerRole::None,
                    true,
                    0,
                    Default::default(),
                    // TODO: What do we want those to be? Any preference? 
                    metadata! {
                        init {
                            "name" => "Ignition Protocol Manager", updatable;
                            "symbol" => "IGNPM", updatable;
                            "description" => "A badge that gives the authority to manage the Ignition protocol.", updatable;
                            "badge" => vec!["badge"], updatable;
                            "dapp_definitions" => vec![dapp_definition_account], updatable;
                        }
                    },
                    Some(dec!(1)),
                )
                .try_deposit_entire_worktop_or_abort(protocol_manager_account, None)
                // The protocol owner badge
                .create_fungible_resource(
                    OwnerRole::None,
                    true,
                    0,
                    Default::default(),
                    metadata! {
                        init {
                            "name" => "Ignition Protocol Owner", updatable;
                            "symbol" => "IGNPO", updatable;
                            "description" => "A badge that of the owner of the ignition protocol.", updatable;
                            "badge" => vec!["badge"], updatable;
                            "dapp_definitions" => vec![dapp_definition_account], updatable;
                        }
                    },
                    Some(dec!(1)),
                )
                .try_deposit_entire_worktop_or_abort(protocol_owner_account, None)
                .build();

            std::thread::sleep(std::time::Duration::from_secs(5));
            let resource_addresses = transaction_service
                .submit_manifest(manifest, &notary_private_key, &fee_handling)?
                .new_resource_addresses;
            (
                *resource_addresses.first().unwrap(),
                *resource_addresses.get(1).unwrap(),
            )
        };

        let protocol_manager_rule = rule!(require(protocol_manager_badge));
        let protocol_owner_rule = rule!(require(protocol_owner_badge));
        let owner_role = OwnerRole::Fixed(protocol_owner_rule.clone());

        // Publishing the packages.
        let (
            ignition_package_address,
            simple_oracle_package_address,
            ociswap_v1_adapter_v1_package_address,
            caviarnine_v1_adapter_v1_package_address,
        ) = {
            let (ignition_code, ignition_package_definition) =
                PackageLoader::get("ignition");
            let (simple_oracle_code, simple_oracle_package_definition) =
                PackageLoader::get("simple-oracle");
            let (
                ociswap_v1_adapter_v1_code,
                ociswap_v1_adapter_v1_package_definition,
            ) = PackageLoader::get("ociswap-v1-adapter-v1");
            let (
                caviarnine_v1_adapter_v1_code,
                caviarnine_v1_adapter_v1_package_definition,
            ) = PackageLoader::get("caviarnine-v1-adapter-v1");

            // We can publish the simple oracle, ociswap adapter v1, and
            // caviarnine adapter v1 all in a single transaction since they
            // are below the size limit.
            let manifest = ManifestBuilder::new()
                .publish_package_advanced(
                    None,
                    simple_oracle_code,
                    simple_oracle_package_definition,
                    metadata_init! {
                        "name" => "Simple Oracle Package", updatable;
                        "description" => "The implementation of the Oracle used by the Ignition protocol.", updatable;
                        "tags" => vec!["oracle"], updatable;
                        "dapp_definition" => dapp_definition_account, updatable;
                    },
                    owner_role.clone(),
                )
                .publish_package_advanced(
                    None,
                    ociswap_v1_adapter_v1_code,
                    ociswap_v1_adapter_v1_package_definition,
                    metadata_init! {
                        "name" => "Ociswap Adapter v1 Package", updatable;
                        "description" => "The implementation of an adapter for Ociswap for the Ignition protocol.", updatable;
                        "tags" => vec!["adapter"], updatable;
                        "dapp_definition" => dapp_definition_account, updatable;
                    },
                    owner_role.clone(),
                )
                .publish_package_advanced(
                    None,
                    caviarnine_v1_adapter_v1_code,
                    caviarnine_v1_adapter_v1_package_definition,
                    metadata_init! {
                        "name" => "Caviarnine Adapter v1 Package", updatable;
                        "description" => "The implementation of an adapter for Caviarnine for the Ignition protocol.", updatable;
                        "tags" => vec!["adapter"], updatable;
                        "dapp_definition" => dapp_definition_account, updatable;
                    },
                    owner_role.clone(),
                ).build();

            std::thread::sleep(std::time::Duration::from_secs(5));
            let package_addresses = transaction_service
                .submit_manifest(manifest, &notary_private_key, &fee_handling)?
                .new_package_addresses;

            let (
                simple_oracle_package_address,
                ociswap_v1_adapter_v1_package_address,
                caviarnine_v1_adapter_v1_package_address,
            ) = (
                *package_addresses.first().unwrap(),
                *package_addresses.get(1).unwrap(),
                *package_addresses.get(2).unwrap(),
            );

            // Publishing the Ignition package
            let manifest = ManifestBuilder::new()
                .publish_package_advanced(
                    None,
                    ignition_code,
                    ignition_package_definition,
                    metadata_init! {
                        "name" => "Ignition Package", updatable;
                        "description" => "The implementation of the Ignition protocol.", updatable;
                        "tags" => Vec::<GlobalAddress>::new(), updatable;
                        "dapp_definition" => dapp_definition_account, updatable;
                    },
                    owner_role.clone(),
                )
                .build();
            std::thread::sleep(std::time::Duration::from_secs(5));
            let ignition_package_address = *transaction_service
                .submit_manifest(manifest, &notary_private_key, &fee_handling)?
                .new_package_addresses
                .first()
                .unwrap();

            (
                ignition_package_address,
                simple_oracle_package_address,
                ociswap_v1_adapter_v1_package_address,
                caviarnine_v1_adapter_v1_package_address,
            )
        };

        // Creating the different liquidity receipt resources that the different
        // exchanges will use. They will be mintable and burnable through the
        // Ignition package caller badge.
        let ignition_package_global_caller_rule =
            rule!(require(package_of_direct_caller(ignition_package_address)));
        let (
            ociswap_v1_liquidity_receipt_resource,
            caviarnine_v1_liquidity_receipt_resource,
        ) = {
            let roles = NonFungibleResourceRoles {
                // Mintable and burnable by the Ignition package and
                // the protocol owner can update who can do that.
                mint_roles: mint_roles! {
                    minter => ignition_package_global_caller_rule.clone();
                    minter_updater => protocol_owner_rule.clone();
                },
                burn_roles: burn_roles! {
                    burner => ignition_package_global_caller_rule.clone();
                    burner_updater => protocol_owner_rule.clone();
                },
                // We reserve the right to change the data of the
                // liquidity receipts when we want.
                non_fungible_data_update_roles: non_fungible_data_update_roles! {
                    non_fungible_data_updater => rule!(deny_all);
                    non_fungible_data_updater_updater => protocol_owner_rule.clone();
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
            };

            let manifest = ManifestBuilder::new()
                // Ociswap liquidity receipt
                .call_function(
                    RESOURCE_PACKAGE,
                    NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                    NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_RUID_WITH_INITIAL_SUPPLY_IDENT,
                    NonFungibleResourceManagerCreateRuidWithInitialSupplyManifestInput {
                        owner_role: owner_role.clone(),
                        track_total_supply: true,
                        non_fungible_schema: NonFungibleDataSchema::new_local_without_self_package_replacement::<LiquidityReceipt<AnyValue>>(),
                        entries: Vec::new(),
                        resource_roles: roles.clone(),
                        metadata: metadata! {
                            roles {
                                metadata_setter => protocol_owner_rule.clone();
                                metadata_setter_updater => protocol_owner_rule.clone();
                                metadata_locker => protocol_owner_rule.clone();
                                metadata_locker_updater => protocol_owner_rule.clone();
                            },
                            init {
                                // TODO: Confirm with the exchanges what they want
                                // their name to be.
                                "name" => "Ignition LP: Ociswap", updatable;
                                "description" => "Represents a particular contribution of liquidity to Ociswap through the Ignition liquidity incentives program. See the redeem_url metadata for where to redeem these NFTs.", updatable;
                                "tags" => vec!["lp token"], updatable;
                                "dapp_definitions" => vec![dapp_definition_account], updatable;
                                // TODO: Must get this from our design team
                                "icon_url" => UncheckedUrl::of("https://www.google.com"), updatable;
                                "DEX" => "Ociswap", updatable;
                                // TODO: Must get this from Ociswap!
                                "redeem_url" => UncheckedUrl::of("https://www.google.com"), updatable;
                            }
                        },
                        address_reservation: None
                    }
                )
                // Caviarnine liquidity receipt
                .call_function(
                    RESOURCE_PACKAGE,
                    NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                    NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_RUID_WITH_INITIAL_SUPPLY_IDENT,
                    NonFungibleResourceManagerCreateRuidWithInitialSupplyManifestInput {
                        owner_role: owner_role.clone(),
                        track_total_supply: true,
                        non_fungible_schema: NonFungibleDataSchema::new_local_without_self_package_replacement::<LiquidityReceipt<AnyValue>>(),
                        entries: Vec::new(),
                        resource_roles: roles.clone(),
                        metadata: metadata! {
                            roles {
                                metadata_setter => protocol_owner_rule.clone();
                                metadata_setter_updater => protocol_owner_rule.clone();
                                metadata_locker => protocol_owner_rule.clone();
                                metadata_locker_updater => protocol_owner_rule.clone();
                            },
                            init {
                                // TODO: Confirm with the exchanges what they want
                                // their name to be.
                                "name" => "Ignition LP: Caviarnine", updatable;
                                "description" => "Represents a particular contribution of liquidity to Caviarnine through the Ignition liquidity incentives program. See the redeem_url metadata for where to redeem these NFTs.", updatable;
                                "tags" => vec!["lp token"], updatable;
                                "dapp_definitions" => vec![dapp_definition_account], updatable;
                                // TODO: Must get this from our design team
                                "icon_url" => UncheckedUrl::of("https://www.google.com"), updatable;
                                "DEX" => "Caviarnine", updatable;
                                // TODO: Must get this from Caviarnine!
                                "redeem_url" => UncheckedUrl::of("https://www.google.com"), updatable;
                            }
                        },
                        address_reservation: None
                    }
                )
                .build();

            std::thread::sleep(std::time::Duration::from_secs(5));
            let resource_addresses = transaction_service
                .submit_manifest(manifest, &notary_private_key, &fee_handling)?
                .new_resource_addresses;
            (
                *resource_addresses.first().unwrap(),
                *resource_addresses.get(1).unwrap(),
            )
        };

        // Creating the oracle and adapters.
        let (
            ignition_component,
            simple_oracle_component,
            ociswap_v1_adapter_v1_component,
            caviarnine_v1_adapter_v1_component,
        ) = {
            let manifest = ManifestBuilder::new()
                // Creating the oracle component
                .call_function(
                    simple_oracle_package_address,
                    "SimpleOracle",
                    "instantiate",
                    (
                        protocol_manager_rule.clone(),
                        metadata_init! {
                            "name" => "Ignition Oracle", updatable;
                            "description" => "The oracle used by the Ignition protocol.", updatable;
                            "tags" => vec!["oracle"], updatable;
                            "dapp_definition" => dapp_definition_account, updatable;
                        },
                        owner_role.clone(),
                        None::<ManifestAddressReservation>,
                    ),
                )
                // Creating the ociswap adapter v1 component
                .call_function(
                    ociswap_v1_adapter_v1_package_address,
                    "OciswapV1Adapter",
                    "instantiate",
                    (
                        metadata_init! {
                            "name" => "Ignition Ociswap Adapter", updatable;
                            "description" => "The adapter used by the Ignition protocol to communicate with Ociswap pools.", updatable;
                            "dapp_definition" => dapp_definition_account, updatable;
                        },
                        owner_role.clone(),
                        None::<ManifestAddressReservation>,
                    ),
                )
                // Creating the ociswap adapter v1 component
                .call_function(
                    caviarnine_v1_adapter_v1_package_address,
                    "CaviarnineV1Adapter",
                    "instantiate",
                    (
                        metadata_init! {
                            "name" => "Ignition Caviarnine Adapter", updatable;
                            "description" => "The adapter used by the Ignition protocol to communicate with Caviarnine pools.", updatable;
                            "dapp_definition" => dapp_definition_account, updatable;
                        },
                        owner_role.clone(),
                        None::<ManifestAddressReservation>,
                    ),
                )
                .build();

            std::thread::sleep(std::time::Duration::from_secs(5));
            let component_addresses = transaction_service
                .submit_manifest(manifest, &notary_private_key, &fee_handling)?
                .new_component_addresses;

            let (
                simple_oracle_component,
                ociswap_v1_adapter_v1_component,
                caviarnine_v1_adapter_v1_component,
            ) = (
                *component_addresses.first().unwrap(),
                *component_addresses.get(1).unwrap(),
                *component_addresses.get(2).unwrap(),
            );

            // Instantiating the Ignition component
            let manifest = ManifestBuilder::new()
                // Instantiate Ignition.
                .call_function(
                    ignition_package_address,
                    "Ignition",
                    "instantiate",
                    manifest_args!(
                        metadata_init! {
                            "name" => "Ignition", updatable;
                            "description" => "The Ignition protocol component", updatable;
                            "dapp_definition" => dapp_definition_account, updatable;
                        },
                        owner_role.clone(),
                        protocol_owner_rule.clone(),
                        protocol_manager_rule.clone(),
                        protocol_resource,
                        simple_oracle_component,
                        MAXIMUM_ALLOWED_PRICE_STALENESS_IN_SECONDS,
                        MAXIMUM_ALLOWED_PRICE_DIFFERENCE_PERCENTAGE,
                        InitializationParametersManifest {
                            initial_pool_information: Some(indexmap! {
                                BlueprintId {
                                    package_address: exchanges.caviarnine_v1.package,
                                    blueprint_name: "QuantaSwap".to_owned()
                                } => PoolBlueprintInformation {
                                    adapter: caviarnine_v1_adapter_v1_component,
                                    allowed_pools: exchanges.caviarnine_v1.pools.into_iter().collect(),
                                    liquidity_receipt: caviarnine_v1_liquidity_receipt_resource
                                },
                                BlueprintId {
                                    package_address: exchanges.ociswap_v1.package,
                                    blueprint_name: "BasicPool".to_owned()
                                } => PoolBlueprintInformation {
                                    adapter: ociswap_v1_adapter_v1_component,
                                    // TODO: Fix this when we have actual 
                                    // ociswap pools.
                                    allowed_pools: Default::default(),
                                    // allowed_pools: exchanges.ociswap.pools.into_iter().collect(),
                                    liquidity_receipt: ociswap_v1_liquidity_receipt_resource
                                }
                            }),
                            initial_user_resource_volatility: Some(
                                indexmap! {
                                    resources.bitcoin => Volatility::Volatile,
                                    resources.ethereum => Volatility::Volatile,
                                    resources.usdc => Volatility::NonVolatile,
                                    resources.usdt => Volatility::NonVolatile,
                                }
                            ),
                            initial_reward_rates: Some(reward_information),
                            initial_volatile_protocol_resources: None,
                            initial_non_volatile_protocol_resources: None,
                            initial_is_open_position_enabled: Some(true),
                            initial_is_close_position_enabled: Some(true),
                        },
                        None::<ManifestAddressReservation>
                    )
                )
                .build();

            std::thread::sleep(std::time::Duration::from_secs(5));
            let component_addresses = transaction_service
                .submit_manifest(
                    manifest,
                    &ephemeral_private_key,
                    &fee_handling,
                )?
                .new_component_addresses;

            let ignition_component_address =
                *component_addresses.first().unwrap();

            (
                ignition_component_address,
                simple_oracle_component,
                ociswap_v1_adapter_v1_component,
                caviarnine_v1_adapter_v1_component,
            )
        };

        // Updating the dapp definition account with the metadata that it
        // should have.
        {
            let manifest = ManifestBuilder::new()
                .set_metadata(
                    dapp_definition_account,
                    "account_type",
                    "dapp definition",
                )
                .set_metadata(
                    dapp_definition_account,
                    "claimed_websites",
                    Vec::<UncheckedOrigin>::new(),
                )
                .set_metadata(
                    dapp_definition_account,
                    "dapp_definitions",
                    Vec::<GlobalAddress>::new(),
                )
                .set_metadata(
                    dapp_definition_account,
                    "claimed_entities",
                    vec![
                        GlobalAddress::from(protocol_manager_badge),
                        GlobalAddress::from(protocol_owner_badge),
                        GlobalAddress::from(ignition_package_address),
                        GlobalAddress::from(simple_oracle_package_address),
                        GlobalAddress::from(
                            ociswap_v1_adapter_v1_package_address,
                        ),
                        GlobalAddress::from(
                            caviarnine_v1_adapter_v1_package_address,
                        ),
                        GlobalAddress::from(
                            ociswap_v1_liquidity_receipt_resource,
                        ),
                        GlobalAddress::from(
                            caviarnine_v1_liquidity_receipt_resource,
                        ),
                        GlobalAddress::from(ignition_component),
                        GlobalAddress::from(simple_oracle_component),
                        GlobalAddress::from(ociswap_v1_adapter_v1_component),
                        GlobalAddress::from(caviarnine_v1_adapter_v1_component),
                    ],
                )
                .call_role_assignment_method(
                    dapp_definition_account,
                    ROLE_ASSIGNMENT_SET_OWNER_IDENT,
                    RoleAssignmentSetOwnerInput {
                        rule: protocol_owner_rule,
                    },
                )
                .build();

            std::thread::sleep(std::time::Duration::from_secs(5));
            transaction_service.submit_manifest(
                manifest,
                &ephemeral_private_key,
                &fee_handling,
            )?;
        }

        Ok(())
    }
}

pub struct DexInformation {
    pub pools: NameIndexedResourceInformation<ComponentAddress>,
    pub package: PackageAddress,
}
