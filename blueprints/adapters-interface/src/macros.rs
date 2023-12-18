//! Defines macros used in the definition of the interfaces of
//! adapters_interface.

#[macro_export]
macro_rules! define_adapter_stubs {
    (
        name: $adapter_name: ident,
        functions: [
            $($functions_tokens: tt)*
        ]
    ) => {
        #[cfg(feature = "scrypto")]
        pub use scrypto::$adapter_name;

        /* Scrypto stubs */
        #[cfg(feature = "scrypto")]
        mod scrypto {
            use super::*;

            #[derive(::scrypto::prelude::ScryptoSbor)]
            #[sbor(transparent)]
            pub struct $adapter_name(pub ::scrypto::prelude::Reference);

            #[cfg(feature = "scrypto")]
            impl<T> From<T> for $adapter_name
            where
                T: Into<::scrypto::prelude::NodeId>,
            {
                fn from(value: T) -> Self {
                    Self(::scrypto::prelude::Reference(value.into()))
                }
            }

            #[cfg(feature = "scrypto")]
            impl $adapter_name {
                $crate::define_functions!($($functions_tokens)*);
            }
        }
    };
}

#[macro_export]
macro_rules! define_functions {
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
        pub fn $fn_name( &mut self, $( $arg_name: $arg_type, )* ) -> $crate::resolve_return_type!($(-> $rtn_type)?) {
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
