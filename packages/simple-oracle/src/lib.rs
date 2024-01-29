use adapters_interface::prelude::*;
use scrypto::prelude::*;
use scrypto_interface::*;

#[blueprint_with_traits]
mod simple_oracle {
    enable_method_auth! {
        roles {
            oracle_manager => updatable_by: [oracle_manager];
        },
        methods {
            set_price => restrict_to: [oracle_manager];
            set_price_batch => restrict_to: [oracle_manager];
            get_price => PUBLIC;
        }
    }

    pub struct SimpleOracle {
        /// Maps the (base, quote) to the (price, updated_at).
        prices: KeyValueStore<
            (ResourceAddress, ResourceAddress),
            (Decimal, Instant),
        >,
    }

    impl SimpleOracle {
        pub fn instantiate(
            oracle_manager: AccessRule,
            metadata_init: MetadataInit,
            owner_role: OwnerRole,
            address_reservation: Option<GlobalAddressReservation>,
        ) -> Global<SimpleOracle> {
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
            .metadata(ModuleConfig {
                init: metadata_init,
                roles: Default::default(),
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

        pub fn set_price_batch(
            &mut self,
            prices: IndexMap<(ResourceAddress, ResourceAddress), Decimal>,
        ) {
            let time = Clock::current_time_rounded_to_minutes();
            for (addresses, price) in prices.into_iter() {
                self.prices.insert(addresses, (price, time))
            }
        }
    }

    impl OracleAdapterInterfaceTrait for SimpleOracle {
        fn get_price(
            &self,
            base: ResourceAddress,
            quote: ResourceAddress,
        ) -> (Price, Instant) {
            let (price, last_update) = *self
                .prices
                .get(&(base, quote))
                .expect("Price not found for this resource");
            (Price { base, quote, price }, last_update)
        }
    }
}
