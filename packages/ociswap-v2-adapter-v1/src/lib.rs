#![warn(clippy::arithmetic_side_effects)]

mod blueprint_interface;
pub use blueprint_interface::*;

use ports_interface::prelude::*;
use scrypto::prelude::*;
use scrypto_interface::*;

macro_rules! define_error {
    (
        $(
            $name: ident => $item: expr;
        )*
    ) => {
        $(
            pub const $name: &'static str = concat!("[Ociswap v2 Adapter v1]", " ", $item);
        )*
    };
}

define_error! {}

#[blueprint_with_traits]
pub mod adapter {
    struct OciswapV2Adapter;

    impl OciswapV2Adapter {
        pub fn instantiate(
            metadata_init: MetadataInit,
            owner_role: OwnerRole,
            address_reservation: Option<GlobalAddressReservation>,
        ) -> Global<OciswapV2Adapter> {
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
                .metadata(ModuleConfig {
                    init: metadata_init,
                    roles: Default::default(),
                })
                .with_address(address_reservation)
                .globalize()
        }

        fn pool(
            component_address: ComponentAddress,
        ) -> OciswapV2PoolInterfaceScryptoStub {
            OciswapV2PoolInterfaceScryptoStub::from(component_address)
        }
    }

    impl PoolAdapterInterfaceTrait for OciswapV2Adapter {
        fn open_liquidity_position(
            &mut self,
            _pool_address: ComponentAddress,
            _buckets: (Bucket, Bucket),
        ) -> OpenLiquidityPositionOutput {
            todo!()
        }

        fn close_liquidity_position(
            &mut self,
            _pool_address: ComponentAddress,
            _pool_units: Bucket,
            _adapter_specific_information: AnyValue,
        ) -> CloseLiquidityPositionOutput {
            todo!()
        }

        fn price(&mut self, _pool_address: ComponentAddress) -> Price {
            todo!()
        }

        fn resource_addresses(
            &mut self,
            pool_address: ComponentAddress,
        ) -> (ResourceAddress, ResourceAddress) {
            let pool = Self::pool(pool_address);
            (pool.x_address(), pool.y_address())
        }
    }
}

#[derive(ScryptoSbor, Debug, Clone)]
pub struct OciswapV2AdapterSpecificInformation {}

impl From<OciswapV2AdapterSpecificInformation> for AnyValue {
    fn from(value: OciswapV2AdapterSpecificInformation) -> Self {
        AnyValue::from_typed(&value).unwrap()
    }
}
