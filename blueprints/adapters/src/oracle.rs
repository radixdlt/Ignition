use crate::define_adapter_stubs;

define_adapter_stubs! {
    name: OracleAdapter,
    functions: [
        /// Gets the price of the base resource in terms of the quote resource
        /// and an instant of when it was last updated.
        fn get_price(
            &self,
            base: scrypto::prelude::ResourceAddress,
            quote: scrypto::prelude::ResourceAddress,
        ) -> (scrypto::prelude::Decimal, scrypto::prelude::Instant);
    ]
}
