#[macro_export]
macro_rules! global_address {
    (
        $address: expr
    ) => {
        $crate::address!(
            ::radix_engine_interface::prelude::GlobalAddress,
            $address
        )
    };
}

#[macro_export]
macro_rules! component_address {
    (
        $address: expr
    ) => {
        $crate::address!(
            ::radix_engine_interface::prelude::ComponentAddress,
            $address
        )
    };
}

#[macro_export]
macro_rules! package_address {
    (
        $address: expr
    ) => {
        $crate::address!(
            ::radix_engine_interface::prelude::PackageAddress,
            $address
        )
    };
}

#[macro_export]
macro_rules! resource_address {
    (
        $address: expr
    ) => {
        $crate::address!(
            ::radix_engine_interface::prelude::ResourceAddress,
            $address
        )
    };
}

#[macro_export]
macro_rules! address {
    (
        $ty: ty,
        $address: expr
    ) => {
        AddressBech32Decoder::validate_and_decode_ignore_hrp($address.as_ref())
            .ok()
            .and_then(|(_, _, address)| address.try_into().ok())
            .map(<$ty>::new_or_panic)
            .unwrap()
    };
}
