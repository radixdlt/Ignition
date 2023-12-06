use scrypto_test::prelude::*;

#[allow(dead_code)]
pub fn is_wasm_panic<T>(result: &Result<T, RuntimeError>) -> bool {
    matches!(
        result,
        Err(RuntimeError::ApplicationError(
            ApplicationError::PanicMessage(..)
        ))
    )
}
