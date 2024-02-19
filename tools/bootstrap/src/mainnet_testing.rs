use crate::error::*;
use crate::transaction_service::*;
use crate::types::*;
use crate::*;
use clap::Parser;
use common::prelude::*;
use ignition::{
    InitializationParametersManifest, PoolBlueprintInformationManifest,
};
use package_loader::PackageLoader;
use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::node_modules::*;
use radix_engine_interface::blueprints::account::*;
use radix_engine_interface::prelude::*;
use transaction::prelude::*;

const PRIVATE_KEY_ENVIRONMENT_VARIABLE: &str = "PRIVATE_KEY";

#[derive(Parser, Debug)]
pub struct MainnetTesting {}

impl MainnetTesting {
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
        const GATEWAY_API_BASE_URL: &str = "https://mainnet.radixdlt.com/";
        let network_definition = NetworkDefinition::mainnet();
        let bech32m_coders =
            Bech32mCoders::from_network_definition(&network_definition);

        // TODO: What do we want these values to be?
        const MAXIMUM_ALLOWED_PRICE_STALENESS_IN_SECONDS: i64 = 60; // 60 seconds
        const MAXIMUM_ALLOWED_PRICE_DIFFERENCE_PERCENTAGE: Decimal =
            Decimal::MAX; // TODO: No oracle is deployed on mainnet for testing yet.

        let protocol_resource = resource_address!("resource_rdx1t4dekrf58h0r28s3c93z92w3jt5ngx87jzd63mgc597zmf3534rxfv");
        let resources = NameIndexedResourceInformation {
            bitcoin: resource_address!("resource_rdx1t58dla7ykxzxe5es89wlhgzatqla0gceukg0eeduzvtj4cxd55etn8"),
            ethereum: resource_address!("resource_rdx1tkscrlztcyn82ej5z3n232f0qqp0qur69arjf279ppmg5usa3xhnsm"),
            usdc: resource_address!("resource_rdx1th7nx2hy0cf6aea6mz7zhkdmy4p45s488xutltnp7296zxj8hwchpf"),
            usdt: resource_address!("resource_rdx1tkafx32lu72mcxr85gjx0rh3rx9q89zqffg4phmv5rxdqg5fnd0w7s"),
        };
        let exchanges = NameIndexedDexInformation {
            caviarnine_v1: DexInformation {
                package: package_address!("package_rdx1p4r9rkp0cq67wmlve544zgy0l45mswn6h798qdqm47x4762h383wa3"),
                pools: NameIndexedResourceInformation {
                    bitcoin: component_address!("component_rdx1crzl2c39m83lpe6fv62epgp3phqunxhc264ys23qz8xeemjcu8lln3"),
                    ethereum: component_address!("component_rdx1cqk2ufmdq6pkcu7ed7r6u9hmdsht9gyd8y8wwtd7w5znefz9k54a7d"),
                    usdc: component_address!("component_rdx1cq9q8umlpmngff6y4e534htz0n37te4m7vsj50u9zc58ys65zl6jv9"),
                    usdt: component_address!("component_rdx1cpl0v3lndt9d7g7uuepztxs9m7m24ly0yfhvcum2y7tm0vlzst0l5y")
                }
            },
            // TODO: Ths following is INCORRECT INFORMATION! There is no Ociswap
            // package on mainnet.
            ociswap_v1: DexInformation {
                package: package_address!("package_rdx1p5l6dp3slnh9ycd7gk700czwlck9tujn0zpdnd0efw09n2zdnn0lzx"),
                pools: NameIndexedResourceInformation {
                    bitcoin: component_address!("component_rdx1cr5uxxjq4a0r3gfn6yd62lk96fqca34tnmyqdxkwefhckcjea4t3am"),
                    ethereum: component_address!("component_rdx1cqylpcl8p45l2h5ew0qrkwyz23dky3e6ucs7kkhrtm90k9z3kzeztn"),
                    usdc: component_address!("component_rdx1cq96chge0q6kkk962heg0mgfl82gjw7x25dp9jv80gkx90mc3hk2ua"),
                    usdt: component_address!("component_rdx1cz3fa8qtfgfwjt3fzrtm544a89p5laerww7590g2tfcradqwdv3laq")
                }
            },
        };
        // TODO: Numbers here are not real and I have added from just to get
        // things going. MUST modify before launch.
        let reward_information = indexmap! {
            LockupPeriod::from_minutes(0).unwrap() => dec!(0.125),  // 12.5%
            LockupPeriod::from_minutes(1).unwrap() => dec!(0.15),  // 15.0%
        };

        // TODO: MUST determine what those accounts are prior to launch!
        // For now, for the TEST deployments these are accounts that I CONTROL!
        let protocol_manager_account = component_address!("account_rdx12xvk6x3usuzu7hdc5clc7lpu8e4czze6xa7vrw7vlek0h84j9299na");
        let protocol_owner_account = component_address!("account_rdx12xvk6x3usuzu7hdc5clc7lpu8e4czze6xa7vrw7vlek0h84j9299na");

        /* cSpell:enable */

        // An ephemeral private key that we will use the bootstrapping process.
        // This key will initially control the dApp definition to allow us to
        // easily update the metadata and will later on change the owner role
        // of the dApp definition to the protocol owner.

        let ephemeral_private_key =
            Ed25519PrivateKey::from_u64(rand::random()).unwrap();
        println!(
            "Ephemeral Private Key: {:?}",
            ephemeral_private_key.to_bytes()
        );

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
                                // TODO: Confirm with the exchanges what they
                                // want their name to be.
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
                                } => PoolBlueprintInformationManifest {
                                    adapter: caviarnine_v1_adapter_v1_component,
                                    allowed_pools: exchanges.caviarnine_v1.pools.into_iter().collect(),
                                    liquidity_receipt: caviarnine_v1_liquidity_receipt_resource
                                },
                                BlueprintId {
                                    package_address: exchanges.ociswap_v1.package,
                                    blueprint_name: "BasicPool".to_owned()
                                } => PoolBlueprintInformationManifest {
                                    adapter: ociswap_v1_adapter_v1_component,
                                    allowed_pools: exchanges.ociswap_v1.pools.into_iter().collect(),
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
