use crate::define_adapter_stubs;
use ::scrypto::prelude::*;

define_adapter_stubs! {
    name: OracleAdapter,
    functions: [
        /// Gets the price of the base resource in terms of the quote resource
        /// and an instant of when it was last updated.
        fn get_price(
            &self,
            base: ResourceAddress,
            quote: ResourceAddress,
        ) -> (Decimal, Instant);
    ]
}
