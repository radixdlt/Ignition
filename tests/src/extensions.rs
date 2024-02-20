use crate::prelude::*;
use extend::ext;

#[ext]
pub impl DefaultTestRunner {
    fn execute_manifest_without_auth(
        &mut self,
        manifest: TransactionManifestV1,
    ) -> TransactionReceiptV1 {
        self.execute_manifest_with_enabled_modules(
            manifest,
            EnabledModules::for_test_transaction() & !EnabledModules::AUTH,
        )
    }

    fn execute_manifest_with_enabled_modules(
        &mut self,
        manifest: TransactionManifestV1,
        enabled_modules: EnabledModules,
    ) -> TransactionReceiptV1 {
        let mut execution_config = ExecutionConfig::for_test_transaction();
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
