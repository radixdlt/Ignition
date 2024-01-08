use adapters_interface::oracle::*;
use scrypto::prelude::*;
use scrypto_interface::*;

#[blueprint_with_traits]
mod test_oracle {
    enable_method_auth! {
        roles {
            oracle_manager => updatable_by: [oracle_manager];
        },
        methods {
            set_price => restrict_to: [oracle_manager];
            get_price => PUBLIC;
        }
    }

    pub struct TestOracle {
        /// Maps the (base, quote) to the (price, updated_at).
        prices: KeyValueStore<
            (ResourceAddress, ResourceAddress),
            (Decimal, Instant),
        >,
    }

    impl TestOracle {
        pub fn instantiate(
            oracle_manager: AccessRule,
            owner_role: OwnerRole,
            address_reservation: Option<GlobalAddressReservation>,
        ) -> Global<TestOracle> {
            let address_reservation = address_reservation.unwrap_or(
                Runtime::allocate_component_address(BlueprintId {
                    package_address: Runtime::package_address(),
                    blueprint_name: Runtime::blueprint_name(),
                })
                .0,
            );

            Self {
                prices: KeyValueStore::new(),
            }
            .instantiate()
            .prepare_to_globalize(owner_role)
            .roles(roles! {
                oracle_manager => oracle_manager;
            })
            .with_address(address_reservation)
            .globalize()
        }

        pub fn set_price(
            &mut self,
            base: ResourceAddress,
            quote: ResourceAddress,
            price: Decimal,
        ) {
            self.prices.insert(
                (base, quote),
                (price, Clock::current_time_rounded_to_minutes()),
            )
        }
    }

    impl OracleAdapterInterfaceTrait for TestOracle {
        fn get_price(
            &self,
            base: ResourceAddress,
            quote: ResourceAddress,
        ) -> Price {
            let (price, last_update) = *self
                .prices
                .get(&(base, quote))
                .expect("Price not found for this resource");
            Price {
                base,
                quote,
                price,
                last_update,
            }
        }
    }
}
