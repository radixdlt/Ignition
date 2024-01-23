mod blueprint_interface;
pub use blueprint_interface::*;

use adapters_interface::prelude::*;
use scrypto::prelude::*;
use scrypto_interface::*;

macro_rules! define_error {
    (
        $(
            $name: ident => $item: expr;
        )*
    ) => {
        $(
            pub const $name: &'static str = concat!("[Ociswap Adapter]", " ", $item);
        )*
    };
}

define_error! {
    FAILED_TO_GET_RESOURCE_ADDRESSES_ERROR
        => "Failed to get resource addresses - unexpected error.";
    FAILED_TO_GET_VAULT_ERROR
        => "Failed to get vault - unexpected error.";
    PRICE_IS_UNDEFINED
        => "Price is undefined.";
    FAILED_TO_CALCULATE_K_VALUE_OF_POOL_ERROR
        => "Failed to calculate the K value of the pool.";
}

#[blueprint_with_traits]
pub mod adapter {
    struct OciswapAdapter;

    impl OciswapAdapter {
        pub fn instantiate(
            owner_role: OwnerRole,
            address_reservation: Option<GlobalAddressReservation>,
        ) -> Global<OciswapAdapter> {
            let address_reservation = address_reservation.unwrap_or(
                Runtime::allocate_component_address(BlueprintId {
                    package_address: Runtime::package_address(),
                    blueprint_name: Runtime::blueprint_name(),
                })
                .0,
            );

            Self {}
                .instantiate()
                .prepare_to_globalize(owner_role)
                .with_address(address_reservation)
                .globalize()
        }

        fn pool(
            component_address: ComponentAddress,
        ) -> OciswapPoolInterfaceScryptoStub {
            OciswapPoolInterfaceScryptoStub::from(component_address)
        }
    }

    impl PoolAdapterInterfaceTrait for OciswapAdapter {
        fn open_liquidity_position(
            &mut self,
            pool_address: ComponentAddress,
            buckets: (Bucket, Bucket),
        ) -> OpenLiquidityPositionOutput {
            let mut pool = Self::pool(pool_address);

            // TODO: Is this actually pool units and change?
            let (pool_units, change) = pool.add_liquidity(buckets.0, buckets.1);

            let user_share = pool_units.amount()
                / pool_units.resource_manager().total_supply().unwrap();

            let pool_k = Global::<TwoResourcePool>::from(pool.liquidity_pool())
                .get_vault_amounts()
                .values()
                .map(|item| PreciseDecimal::from(*item))
                .reduce(|acc, item| acc * item)
                .expect(FAILED_TO_CALCULATE_K_VALUE_OF_POOL_ERROR);

            OpenLiquidityPositionOutput {
                pool_units,
                change: change
                    .map(|bucket| {
                        indexmap! {
                            bucket.resource_address() => bucket
                        }
                    })
                    .unwrap_or_default(),
                others: Default::default(),
                adapter_specific_information:
                    OciswapAdapterSpecificInformation {
                        user_share_in_pool_when_position_opened: user_share,
                        pool_k_when_position_opened: pool_k,
                    }
                    .into(),
            }
        }

        fn close_liquidity_position(
            &mut self,
            pool_address: ComponentAddress,
            pool_units: Bucket,
            _adapter_specific_information: AnyValue,
        ) -> CloseLiquidityPositionOutput {
            let mut pool = Self::pool(pool_address);

            let (bucket1, bucket2) = pool.remove_liquidity(pool_units);

            CloseLiquidityPositionOutput {
                resources: indexmap! {
                    bucket1.resource_address() => bucket1,
                    bucket2.resource_address() => bucket2,
                },
                others: Default::default(),
                // TODO: Determine how we wish to go about this calculation.
                fees: Default::default(),
            }
        }

        fn price(&mut self, pool_address: ComponentAddress) -> Price {
            let pool = Self::pool(pool_address);
            let pool = Global::<TwoResourcePool>::from(pool.liquidity_pool());
            let vault_amounts = pool.get_vault_amounts();

            let (resource_address1, resource_address2) =
                self.resource_addresses(pool_address);
            let amount1 = *vault_amounts
                .get(&resource_address1)
                .expect(FAILED_TO_GET_VAULT_ERROR);
            let amount2 = *vault_amounts
                .get(&resource_address2)
                .expect(FAILED_TO_GET_VAULT_ERROR);

            Price {
                base: resource_address1,
                quote: resource_address2,
                price: amount2.checked_div(amount1).expect(PRICE_IS_UNDEFINED),
            }
        }

        fn resource_addresses(
            &mut self,
            pool_address: ComponentAddress,
        ) -> (ResourceAddress, ResourceAddress) {
            let pool = Self::pool(pool_address);
            let pool = Global::<TwoResourcePool>::from(pool.liquidity_pool());
            let mut keys = pool.get_vault_amounts().into_keys();

            let resource_address1 =
                keys.next().expect(FAILED_TO_GET_RESOURCE_ADDRESSES_ERROR);
            let resource_address2 =
                keys.next().expect(FAILED_TO_GET_RESOURCE_ADDRESSES_ERROR);

            (resource_address1, resource_address2)
        }

        // TODO: Final check prior to launch.
        fn exchange_specific_liquidity_receipt_data(
            &mut self,
        ) -> LiquidityReceiptExchangeSpecificData {
            LiquidityReceiptExchangeSpecificData {
                name: "Ociswap Liquidity Receipt".to_owned(),
                description: "A receipt of liquidity provided to a Ociswap pool through the Ignition protocol".to_owned(),
                key_image_url: Url::of("https://ociswap.com/icons/oci.png"),
                redemption_url: Url::of("https://ociswap.com/"),
            }
        }
    }
}

#[derive(ScryptoSbor, Debug, Clone)]
pub struct OciswapAdapterSpecificInformation {
    /// The share of the user in the pool when the position was opened.
    pub user_share_in_pool_when_position_opened: Decimal,

    /// The value of the K of the pool when the position was opened.
    pub pool_k_when_position_opened: PreciseDecimal,
}

impl From<OciswapAdapterSpecificInformation> for AnyValue {
    fn from(value: OciswapAdapterSpecificInformation) -> Self {
        AnyValue::from_typed(&value).unwrap()
    }
}
