//! Defines macros used in the definition of the interfaces of adapters.

#[macro_export]
macro_rules! define_adapter_stubs {
    (
        name: $adapter_name: ident,
        functions: [
            $($functions_tokens: tt)*
        ]
    ) => {
        /* Scrypto stub */
        #[cfg(feature = "scrypto")]
        pub use scrypto::$adapter_name;

        #[cfg(feature = "scrypto")]
        mod scrypto {
            #[derive(::scrypto::prelude::ScryptoSbor)]
            #[sbor(transparent)]
            pub struct $adapter_name<T>(pub T);

            impl<T> From<T> for $adapter_name<::scrypto::prelude::BlueprintId>
            where
                T: Into<::scrypto::prelude::BlueprintId>,
            {
                fn from(value: T) -> $adapter_name<::scrypto::prelude::BlueprintId> {
                    Self(value.into())
                }
            }

            impl<T> From<T> for $adapter_name<::scrypto::prelude::Reference>
            where
                T: Into<::scrypto::prelude::NodeId>,
            {
                fn from(value: T) -> $adapter_name<::scrypto::prelude::Reference> {
                    Self(::scrypto::prelude::Reference(value.into()))
                }
            }

            impl $adapter_name<::scrypto::prelude::BlueprintId> {
                $crate::define_functions!($($functions_tokens)*);
            }

            impl $adapter_name<::scrypto::prelude::Reference> {
                $crate::define_methods!($($functions_tokens)*);
            }
        }


    };
}

#[macro_export]
macro_rules! define_functions {
    (
        $(#[$meta: meta])*
        fn $fn_name: ident (
            $(
                $arg_name: ident: $arg_type: ty
            ),* $(,)?
        ) $(-> $rtn_type: ty)? ; $($tokens: tt)*
    ) => {
        $(#[$meta])*
        pub fn $fn_name(
            $( $arg_name: $arg_type, )*
        ) -> resolve_return_type!($(-> $rtn_type)?) {
            ::scrypto::prelude::scrypto_decode(&::scrypto::prelude::ScryptoVmV1Api::blueprint_call(
                self.0.package_address,
                &self.0.blueprint_name,
                stringify!($fn_name),
                ::scrypto::prelude::scrypto_encode(&(
                    $( $arg_name ),* ,
                )).unwrap()
            )).unwrap()
        }
    };
    (
        $(#[$meta:meta])*
        fn $fn_name: ident (
            &self,
            $(
                $arg_name: ident: $arg_type: ty
            ),* $(,)?
        ) $(-> $rtn_type: ty)? ; $($tokens: tt)*
    ) => {

    };
    (
        $(#[$meta:meta])*
        fn $fn_name: ident (
            &mut self,
            $(
                $arg_name: ident: $arg_type: ty
            ),* $(,)?
        ) $(-> $rtn_type: ty)? ; $($tokens: tt)*
    ) => {

    };
    () => {};
}

#[macro_export]
macro_rules! define_methods {
    (
        $(#[$meta: meta])*
        fn $fn_name: ident (
            $(
                $arg_name: ident: $arg_type: ty
            ),* $(,)?
        ) $(-> $rtn_type: ty)? ; $($tokens: tt)*
    ) => {

    };
    (
        $(#[$meta:meta])*
        fn $fn_name: ident (
            &self,
            $(
                $arg_name: ident: $arg_type: ty
            ),* $(,)?
        ) $(-> $rtn_type: ty)? ; $($tokens: tt)*
    ) => {
        $(#[$meta])*
        pub fn $fn_name( &self, $( $arg_name: $arg_type, )* ) -> $crate::resolve_return_type!($(-> $rtn_type)?) {
            ::scrypto::prelude::scrypto_decode(
                &::scrypto::prelude::ScryptoVmV1Api::object_call(
                    &self.0.as_node_id(),
                    stringify!($fn_name),
                    ::scrypto::prelude::scrypto_encode(&(
                        $( $arg_name ),* ,
                    )).unwrap()
                )
            ).unwrap()
        }
    };
    (
        $(#[$meta:meta])*
        fn $fn_name: ident (
            &mut self,
            $(
                $arg_name: ident: $arg_type: ty
            ),* $(,)?
        ) $(-> $rtn_type: ty)? ; $($tokens: tt)*
    ) => {
        $(#[$meta])*
        pub fn $fn_name( &mut self, $( $arg_name: $arg_type, )* ) -> resolve_return_type!($(-> $rtn_type)?) {
            ::scrypto::prelude::scrypto_decode(
                &::scrypto::prelude::ScryptoVmV1Api::object_call(
                    &self.0.as_node_id(),
                    stringify!($fn_name),
                    ::scrypto::prelude::scrypto_encode(&(
                        $( $arg_name ),* ,
                    )).unwrap()
                )
            ).unwrap()
        }
    };
    () => {};
}

#[macro_export]
macro_rules! resolve_return_type {
    () => {
        ()
    };
    (-> $type: ty) => {
        $type
    };
}
