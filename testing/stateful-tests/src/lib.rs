use common::prelude::*;
use extend::*;
use publishing_tool::component_address;
use publishing_tool::configuration_selector::*;
use publishing_tool::database_overlay::*;
use publishing_tool::network_connection_provider::*;
use publishing_tool::publishing::*;
use radix_engine::system::system_modules::*;
use radix_engine::transaction::*;
use radix_engine::vm::*;
use radix_engine_interface::blueprints::account::*;
use scrypto_unit::*;
use state_manager::RocksDBStore;
use std::ops::*;
use transaction::prelude::*;

lazy_static::lazy_static! {
    /// The substate manager database is a lazy-static since it takes a lot of
    /// time to be opened for read-only and this had a very negative impact on
    /// tests. Keep in mind that this now means that we should keep all of the
    /// tests to one module and that we should use `cargo test` and not nextest.
    static ref SUBSTATE_MANAGER_DATABASE: RocksDBStore = {
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
        RocksDBStore::new_read_only(state_manager_database_path).expect(
            "Failed to create a new instance of the state manager database",
        )
    };
}

pub type StatefulTestRunner<'a> = TestRunner<
    NoExtension,
    UnmergeableSubstateDatabaseOverlay<'a, RocksDBStore>,
>;

pub fn execute_test_within_environment<F, O>(test_function: F) -> O
where
    F: Fn(
        AccountAndControllingKey,
        PublishingConfiguration,
        PublishingReceipt,
        ComponentAddress,
        &mut StatefulTestRunner<'_>,
    ) -> O,
{
    // Creating the database and the necessary overlays to run the tests.

    let overlayed_state_manager_database =
        UnmergeableSubstateDatabaseOverlay::new_unmergeable(
            SUBSTATE_MANAGER_DATABASE.deref(),
        );

    // Creating a test runner from the overlayed state manager database
    let mut test_runner = TestRunnerBuilder::new()
        .with_custom_database(overlayed_state_manager_database)
        .without_trace()
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
    test_runner
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

    // The notary account address now has the fees required to be able to pay
    // for the deployment of Ignition. We now get the configuration and run the
    // deployment.
    let mut simulator_network_connection_provider =
        SimulatorNetworkConnector::new_with_test_runner(
            test_runner,
            NetworkDefinition::mainnet(),
        );
    let publishing_configuration = ConfigurationSelector::MainnetProduction
        .configuration(&notary_private_key);
    let publishing_receipt = publish(
        &publishing_configuration,
        &mut simulator_network_connection_provider,
    )
    .expect("Publishing of Ignition must succeed!");
    let mut test_runner =
        simulator_network_connection_provider.into_test_runner();

    // Modifying the Ignition component state so that we can use it in tests.
    // What we will modify is the Oracle to use where we will be using an actual
    // oracle that is live on mainnet that prices are being submitted to. We
    // also need to allow for opening and closing of liquidity positions.
    // Additionally, we fund Ignition with XRD.
    let oracle = component_address!(
        "component_rdx1crty68w9d6ud4ecreewvpsvgyq0u9ta8syqrmuzelem593putyu79e"
    );
    test_runner
        .execute_manifest_with_enabled_modules(
            ManifestBuilder::new()
                .call_method(
                    publishing_receipt.components.protocol_entities.ignition,
                    "set_oracle_adapter",
                    (oracle,),
                )
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
                            publishing_receipt
                                .components
                                .protocol_entities
                                .ignition,
                            "deposit_protocol_resources",
                            (volatile, Volatility::Volatile),
                        )
                        .call_method(
                            publishing_receipt
                                .components
                                .protocol_entities
                                .ignition,
                            "deposit_protocol_resources",
                            (non_volatile, Volatility::NonVolatile),
                        )
                })
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
    test_runner
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
    test_function(
        test_account,
        publishing_configuration,
        publishing_receipt,
        oracle,
        &mut test_runner,
    )
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
///     _test_runner: &mut StatefulTestRunner<'_>,
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
pub impl<'a> StatefulTestRunner<'a> {
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
        let mut execution_config = ExecutionConfig::for_notarized_transaction(
            NetworkDefinition::mainnet(),
        );
        execution_config.enabled_modules = enabled_modules;

        let nonce = self.next_transaction_nonce();
        let test_transaction = TestTransaction::new_from_nonce(manifest, nonce);
        let prepared_transaction = test_transaction.prepare().unwrap();
        let executable =
            prepared_transaction.get_executable(Default::default());
        self.execute_transaction(
            executable,
            Default::default(),
            execution_config,
        )
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
        self.execute_raw_transaction(
            &network_definition,
            &transaction.to_raw().unwrap(),
        )
    }
}
