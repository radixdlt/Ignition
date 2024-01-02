#[cfg(not(any(feature = "scrypto", feature = "scrypto-test")))]
compile_error!(
    "one of features \"scrypto\" or \"scrypto-test\" must be enabled"
);

#[cfg(all(feature = "scrypto", feature = "scrypto-test"))]
compile_error!("features \"scrypto\" and \"scrypto-test\" cannot be enabled at the same time");

pub mod macros;
pub mod oracle;
pub mod pool;
