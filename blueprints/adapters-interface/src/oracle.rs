use radix_engine_common::prelude::*;
use scrypto_interface::define_interface;

define_interface! {
    OracleAdapter {
        /// Gets the price of the base resource in terms of the quote resource
        /// and an instant of when it was last updated.
        fn get_price(
            &self,
            base: ResourceAddress,
            quote: ResourceAddress,
        ) -> (Decimal, Instant);
    }
}
