#![allow(unused_macros)]

#[macro_export]
macro_rules! package_address {
    ($address: expr) => {
        ::radix_engine_interface::prelude::PackageAddress::try_from(
            $crate::decode_to_node_id!($address),
        )
        .unwrap()
    };
}

#[macro_export]
macro_rules! component_address {
    ($address: expr) => {
        ::radix_engine_interface::prelude::ComponentAddress::try_from(
            $crate::decode_to_node_id!($address),
        )
        .unwrap()
    };
}

#[macro_export]
macro_rules! resource_address {
    ($address: expr) => {
        ::radix_engine_interface::prelude::ResourceAddress::try_from(
            $crate::decode_to_node_id!($address),
        )
        .unwrap()
    };
}

#[macro_export]
macro_rules! internal_address {
    ($address: expr) => {
        ::radix_engine_interface::prelude::InternalAddress::try_from(
            $crate::decode_to_node_id!($address),
        )
        .unwrap()
    };
}

#[macro_export]
macro_rules! global_address {
    ($address: expr) => {
        ::radix_engine_interface::prelude::GlobalAddress::try_from(
            $crate::decode_to_node_id!($address),
        )
        .unwrap()
    };
}

#[macro_export]
macro_rules! decode_to_node_id {
    ($address: expr) => {
        ::radix_engine_interface::prelude::AddressBech32Decoder::validate_and_decode_ignore_hrp(
            $address,
        )
        .ok()
        .and_then(|(_, _, value)| value.try_into().map(NodeId).ok())
        .unwrap()
    };
}
