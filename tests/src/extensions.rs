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
}
