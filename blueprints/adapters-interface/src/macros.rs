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

            #[derive(
                ::scrypto::prelude::ScryptoSbor,
                Clone,
                Copy,
                Debug,
                PartialEq,
                Eq,
                PartialOrd,
                Ord,
                Hash
            )]
            #[sbor(transparent)]
            pub struct $adapter_name(pub ::scrypto::prelude::Reference);

            impl<T> From<T> for $adapter_name
            where
                T: Into<::scrypto::prelude::NodeId>,
            {
                fn from(value: T) -> Self {
                    Self(::scrypto::prelude::Reference(value.into()))
                }
            }

            impl $adapter_name {
                $crate::define_functions!($($functions_tokens)*);
            }
        }

        #[cfg(feature = "scrypto-test")]
        pub use scrypto_test::$adapter_name;

        /* Scrypto stubs */
        #[cfg(feature = "scrypto-test")]
        mod scrypto_test {
            use super::*;

            #[derive(Debug,Clone,Copy)]
            pub struct $adapter_name (pub ::scrypto_test::prelude::NodeId);

            impl<D: ::sbor::Decoder<::scrypto_test::prelude::ScryptoCustomValueKind>>
                ::scrypto_test::prelude::Decode<::scrypto_test::prelude::ScryptoCustomValueKind, D>
                for $adapter_name
            {
                #[inline]
                fn decode_body_with_value_kind(
                    decoder: &mut D,
                    value_kind: ::scrypto_test::prelude::ValueKind<
                        ::scrypto_test::prelude::ScryptoCustomValueKind,
                    >,
                ) -> std::result::Result<Self, ::scrypto_test::prelude::DecodeError> {
                    let node_id = match value_kind {
                        ValueKind::Custom(
                            ::scrypto_test::prelude::ScryptoCustomValueKind::Reference,
                        ) => {
                            <::scrypto_test::prelude::Reference as ::scrypto_test::prelude::Decode<
                                ::scrypto_test::prelude::ScryptoCustomValueKind,
                                D,
                            >>::decode_body_with_value_kind(
                                decoder, value_kind
                            )
                            .map(|reference| reference.0)
                        }
                        ValueKind::Custom(
                            ::scrypto_test::prelude::ScryptoCustomValueKind::Own,
                        ) => <::scrypto_test::prelude::Own as ::scrypto_test::prelude::Decode<
                            ::scrypto_test::prelude::ScryptoCustomValueKind,
                            D,
                        >>::decode_body_with_value_kind(
                            decoder, value_kind
                        )
                        .map(|own| own.0),
                        _ => Err(::scrypto_test::prelude::DecodeError::InvalidCustomValue),
                    }?;
                    Ok(Self(node_id))
                }
            }

            impl ::core::convert::TryFrom<$adapter_name>
                for ::scrypto_test::prelude::ComponentAddress
            {
                type Error = ::scrypto_test::prelude::ParseComponentAddressError;
                fn try_from(value: $adapter_name) -> ::std::result::Result<Self, Self::Error> {
                    ::scrypto_test::prelude::ComponentAddress::try_from(value.0)
                }
            }
            impl ::core::convert::TryFrom<$adapter_name> for ::scrypto_test::prelude::ResourceAddress {
                type Error = ::scrypto_test::prelude::ParseResourceAddressError;
                fn try_from(value: $adapter_name) -> ::std::result::Result<Self, Self::Error> {
                    ::scrypto_test::prelude::ResourceAddress::try_from(value.0)
                }
            }
            impl ::core::convert::TryFrom<$adapter_name> for ::scrypto_test::prelude::PackageAddress {
                type Error = ::scrypto_test::prelude::ParsePackageAddressError;
                fn try_from(value: $adapter_name) -> ::std::result::Result<Self, Self::Error> {
                    ::scrypto_test::prelude::PackageAddress::try_from(value.0)
                }
            }
            impl ::core::convert::TryFrom<$adapter_name> for ::scrypto_test::prelude::GlobalAddress {
                type Error = ::scrypto_test::prelude::ParseGlobalAddressError;
                fn try_from(value: $adapter_name) -> ::std::result::Result<Self, Self::Error> {
                    ::scrypto_test::prelude::GlobalAddress::try_from(value.0)
                }
            }
            impl ::core::convert::TryFrom<$adapter_name> for ::scrypto_test::prelude::InternalAddress {
                type Error = ::scrypto_test::prelude::ParseInternalAddressError;
                fn try_from(value: $adapter_name) -> ::std::result::Result<Self, Self::Error> {
                    ::scrypto_test::prelude::InternalAddress::try_from(value.0)
                }
            }
            impl ::core::convert::From<$adapter_name> for ::scrypto_test::prelude::Own {
                fn from(value: $adapter_name) -> Self {
                    Self(value.0)
                }
            }
            impl ::core::convert::From<$adapter_name> for ::scrypto_test::prelude::Reference {
                fn from(value: $adapter_name) -> Self {
                    Self(value.0)
                }
            }
            impl ::core::convert::From<$adapter_name> for ::scrypto_test::prelude::NodeId {
                fn from(value: $adapter_name) -> ::scrypto_test::prelude::NodeId {
                    value.0
                }
            }

            impl $adapter_name {
                $crate::define_functions!($($functions_tokens)*);
            }
        }
    };
}

#[cfg(feature = "scrypto")]
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

#[cfg(feature = "scrypto-test")]
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
        pub fn $fn_name(
            &self
            $( , $arg_name: $arg_type )* ,
            env: &mut ::scrypto_test::prelude::TestEnvironment
        ) -> Result<
            $crate::resolve_return_type!($(-> $rtn_type)?),
            ::scrypto_test::prelude::RuntimeError
        > {
            env.call_method_typed(
                self.0,
                stringify!($fn_name),
                &($($arg_name,)*)
            )
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
        pub fn $fn_name(
            &mut self
            $( , $arg_name: $arg_type )* ,
            env: &mut ::scrypto_test::prelude::TestEnvironment
        ) -> Result<
            $crate::resolve_return_type!($(-> $rtn_type)?),
            ::scrypto_test::prelude::RuntimeError
        > {
            env.call_method_typed(
                self.0,
                stringify!($fn_name),
                &($($arg_name,)*)
            )
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
