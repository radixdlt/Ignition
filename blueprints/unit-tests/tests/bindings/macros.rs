#[macro_export]
macro_rules! test_bindings {
    (
        $blueprint:ident,
        $struct_name:ident,
        $functions:ident {
            $($function_contents:tt)*
        },
        {
            $($method_contents:tt)*
        } $(,)?
    ) => {
        paste::paste! {
            #[derive(Debug,Clone,Copy)]
            pub struct $struct_name (pub NodeId);

            impl<D: ::sbor::Decoder<::scrypto::prelude::ScryptoCustomValueKind>>
                ::scrypto::prelude::Decode<::scrypto::prelude::ScryptoCustomValueKind, D>
                for $struct_name
            {
                #[inline]
                fn decode_body_with_value_kind(
                    decoder: &mut D,
                    value_kind: ::scrypto::prelude::ValueKind<
                        ::scrypto::prelude::ScryptoCustomValueKind,
                    >,
                ) -> std::result::Result<Self, ::scrypto::prelude::DecodeError> {
                    let node_id = match value_kind {
                        ValueKind::Custom(
                            ::scrypto::prelude::ScryptoCustomValueKind::Reference,
                        ) => {
                            <::scrypto::prelude::Reference as ::scrypto::prelude::Decode<
                                ::scrypto::prelude::ScryptoCustomValueKind,
                                D,
                            >>::decode_body_with_value_kind(
                                decoder, value_kind
                            )
                            .map(|reference| reference.0)
                        }
                        ValueKind::Custom(
                            ::scrypto::prelude::ScryptoCustomValueKind::Own,
                        ) => <::scrypto::prelude::Own as ::scrypto::prelude::Decode<
                            ::scrypto::prelude::ScryptoCustomValueKind,
                            D,
                        >>::decode_body_with_value_kind(
                            decoder, value_kind
                        )
                        .map(|own| own.0),
                        _ => Err(::scrypto::prelude::DecodeError::InvalidCustomValue),
                    }?;
                    Ok(Self(node_id))
                }
            }

            impl ::core::convert::TryFrom<$struct_name>
                for ::scrypto::prelude::ComponentAddress
            {
                type Error = ::scrypto::prelude::ParseComponentAddressError;
                fn try_from(value: $struct_name) -> ::std::result::Result<Self, Self::Error> {
                    ::scrypto::prelude::ComponentAddress::try_from(value.0)
                }
            }
            impl ::core::convert::TryFrom<$struct_name> for ::scrypto::prelude::ResourceAddress {
                type Error = ::scrypto::prelude::ParseResourceAddressError;
                fn try_from(value: $struct_name) -> ::std::result::Result<Self, Self::Error> {
                    ::scrypto::prelude::ResourceAddress::try_from(value.0)
                }
            }
            impl ::core::convert::TryFrom<$struct_name> for ::scrypto::prelude::PackageAddress {
                type Error = ::scrypto::prelude::ParsePackageAddressError;
                fn try_from(value: $struct_name) -> ::std::result::Result<Self, Self::Error> {
                    ::scrypto::prelude::PackageAddress::try_from(value.0)
                }
            }
            impl ::core::convert::TryFrom<$struct_name> for ::scrypto::prelude::GlobalAddress {
                type Error = ::scrypto::prelude::ParseGlobalAddressError;
                fn try_from(value: $struct_name) -> ::std::result::Result<Self, Self::Error> {
                    ::scrypto::prelude::GlobalAddress::try_from(value.0)
                }
            }
            impl ::core::convert::TryFrom<$struct_name> for ::scrypto::prelude::InternalAddress {
                type Error = ::scrypto::prelude::ParseInternalAddressError;
                fn try_from(value: $struct_name) -> ::std::result::Result<Self, Self::Error> {
                    ::scrypto::prelude::InternalAddress::try_from(value.0)
                }
            }
            impl ::core::convert::From<$struct_name> for ::scrypto::prelude::Own {
                fn from(value: $struct_name) -> Self {
                    Self(value.0)
                }
            }
            impl ::core::convert::From<$struct_name> for ::scrypto::prelude::Reference {
                fn from(value: $struct_name) -> Self {
                    Self(value.0)
                }
            }
            impl ::core::convert::From<$struct_name> for ::scrypto::prelude::NodeId {
                fn from(value: $struct_name) -> ::scrypto::prelude::NodeId {
                    value.0
                }
            }

            impl $struct_name {
                external_functions!{$($function_contents)*}

                external_methods!{$($method_contents)*}

                pub fn blueprint_id(
                    package_address: ::scrypto::prelude::PackageAddress
                ) -> ::scrypto::prelude::BlueprintId {
                    ::scrypto::prelude::BlueprintId {
                        package_address,
                        blueprint_name: stringify!($blueprint).into()
                    }
                }

                fn call_function<Y, E>(
                    package_address: ::scrypto::prelude::PackageAddress,
                    function: &str,
                    arguments: Vec<u8>,
                    env: &mut Y
                ) -> std::result::Result<Vec<u8>, E>
                where
                    Y: ::scrypto::api::ClientApi<E>,
                    E: ::std::fmt::Debug
                {
                    env.call_function(
                        package_address,
                        stringify!($blueprint),
                        function,
                        arguments
                    )
                }
            }
        }
    };
}

#[macro_export]
macro_rules! external_functions {
    (
        fn $method_name:ident(&self$(, $method_args:ident: $method_types:ty)* $(,)?) -> $method_output:ty;
        $($rest:tt)*
    ) => {
        compile_error!("The external_blueprint! macro cannot be used to define component methods which take &self or &mut self. For these component methods, use a separate external_component! macro.");
    };
    (
        fn $method_name:ident(&self$(, $method_args:ident: $method_types:ty)* $(,)?);
        $($rest:tt)*
    ) => {
        compile_error!("The external_blueprint! macro cannot be used to define component methods which take &self or &mut self. For these component methods, use a separate external_component! macro.");
    };
    (
        fn $method_name:ident(&mut self$(, $method_args:ident: $method_types:ty)* $(,)?) -> $method_output:ty;
        $($rest:tt)*
    ) => {
        compile_error!("The external_blueprint! macro cannot be used to define component methods which take &self or &mut self. For these component methods, use a separate external_component! macro.");
    };
    (
        fn $method_name:ident(&mut self$(, $method_args:ident: $method_types:ty)* $(,)?);
        $($rest:tt)*
    ) => {
        compile_error!("The external_blueprint! macro cannot be used to define component methods which take &self or &mut self. For these component methods, use a separate external_component! macro.");
    };
    (
        fn $method_name:ident(self$(, $method_args:ident: $method_types:ty)* $(,)?) -> $method_output:ty;
        $($rest:tt)*
    ) => {
        compile_error!("The external_blueprint! macro cannot be used to define component methods which take &self or &mut self. Also, just self is not supported. For these component methods, use a separate external_component! macro.");
    };
    (
        fn $method_name:ident(self$(, $method_args:ident: $method_types:ty)* $(,)?);
        $($rest:tt)*
    ) => {
        compile_error!("The external_blueprint! macro cannot be used to define component methods which take &self or &mut self. Also, just self is not supported. For these component methods, use a separate external_component! macro.");
    };
    (
        $(#[$meta: meta])*
        fn $func_name:ident($($func_args:ident: $func_types:ty),* $(,)?) -> $func_output:ty;
        $($rest:tt)*
    ) => {
        $(#[$meta])*
        pub fn $func_name<Y, E>(
            $($func_args: $func_types),*,
            blueprint_package_address: ::scrypto::prelude::PackageAddress,
            env: &mut Y
        ) -> std::result::Result<$func_output, E>
        where
            Y: ::scrypto::api::ClientApi<E>,
            E: ::std::fmt::Debug
        {
            let rtn = Self::call_function(
                blueprint_package_address,
                stringify!($func_name),
                ::scrypto::prelude::scrypto_encode(
                    &(
                        $($func_args),*
                    )
                ).unwrap(),
                env
            )?;
            Ok(::scrypto::prelude::scrypto_decode(&rtn).unwrap())
        }

        $crate::external_functions!($($rest)*);
    };
    (
        $(#[$meta: meta])*
        fn $func_name:ident($($func_args:ident: $func_types:ty),* $(,)?);
        $($rest:tt)*
    ) => {
        $(#[$meta])*
        pub fn $func_name<Y, E>(
            $($func_args: $func_types),*,
            blueprint_package_address: ::scrypto::prelude::PackageAddress,
            env: &mut Y
        ) -> std::result::Result<(), E>
        where
            Y: ::scrypto::api::ClientApi<E>,
            E: ::std::fmt::Debug
        {
            let rtn = Self::call_function(
                blueprint_package_address,
                stringify!($func_name),
                ::scrypto::prelude::scrypto_encode(
                    &(
                        $($func_args),*
                    )
                ).unwrap(),
                env
            )?;
            Ok(::scrypto::prelude::scrypto_decode(&rtn).unwrap())
        }

        $crate::external_functions!($($rest)*);
    };
    () => {
    };
}

#[macro_export]
macro_rules! external_methods {
    (
        $(#[$meta: meta])*
        fn $method_name:ident(&self$(, $method_args:ident: $method_types:ty)* $(,)?) -> $method_output:ty;
        $($rest:tt)*
    ) => {
        $(#[$meta])*
        pub fn $method_name<Y, E>(
            &self $(, $method_args: $method_types)*,
            env: &mut Y
        ) -> std::result::Result<$method_output, E>
        where
            Y: ::scrypto::api::ClientApi<E>,
            E: ::std::fmt::Debug
        {
            let rtn = env.call_method(
                &self.0,
                stringify!($method_name),
                scrypto_args!($($method_args),*)
            )?;
            Ok(scrypto_decode(&rtn).unwrap())
        }
        $crate::external_methods!($($rest)*);
    };
    (
        $(#[$meta: meta])*
        fn $method_name:ident(&self$(, $method_args:ident: $method_types:ty)* $(,)?);
        $($rest:tt)*
    ) => {
        $(#[$meta])*
        pub fn $method_name<Y, E>(
            &self $(, $method_args: $method_types)*,
            env: &mut Y
        ) -> std::result::Result<(), E>
        where
            Y: ::scrypto::api::ClientApi<E>,
            E: ::std::fmt::Debug
        {
            let rtn = env.call_method(
                &self.0,
                stringify!($method_name),
                scrypto_args!($($method_args),*)
            )?;
            Ok(scrypto_decode(&rtn).unwrap())
        }
        $crate::external_methods!($($rest)*);
    };
    (
        $(#[$meta: meta])*
        fn $method_name:ident(&mut self$(, $method_args:ident: $method_types:ty)* $(,)?) -> $method_output:ty;
        $($rest:tt)*
    ) => {
        $(#[$meta])*
        pub fn $method_name<Y, E>(
            &mut self $(, $method_args: $method_types)*,
            env: &mut Y
        ) -> std::result::Result<$method_output, E>
        where
            Y: ::scrypto::api::ClientApi<E>,
            E: ::std::fmt::Debug
        {
            let rtn = env.call_method(
                &self.0,
                stringify!($method_name),
                scrypto_args!($($method_args),*)
            )?;
            Ok(scrypto_decode(&rtn).unwrap())
        }
        $crate::external_methods!($($rest)*);
    };
    (
        $(#[$meta: meta])*
        fn $method_name:ident(&mut self$(, $method_args:ident: $method_types:ty)* $(,)?);
        $($rest:tt)*
    ) => {
        $(#[$meta])*
        pub fn $method_name<Y, E>(
            &mut self $(, $method_args: $method_types)*,
            env: &mut Y
        ) -> std::result::Result<(), E>
        where
            Y: ::scrypto::api::ClientApi<E>,
            E: ::std::fmt::Debug
        {
            let rtn = env.call_method(
                &self.0,
                stringify!($method_name),
                scrypto_args!($($method_args),*)
            )?;
            Ok(scrypto_decode(&rtn).unwrap())
        }
        $crate::external_methods!($($rest)*);
    };
    (
        $(#[$meta: meta])*
        fn $method_name:ident(self$(, $method_args:ident: $method_types:ty)* $(,)?) -> $method_output:ty;
        $($rest:tt)*
    ) => {
        compile_error!("Components cannot define methods taking self. Did you mean &self or &mut self instead?");
    };
    (
        $(#[$meta: meta])*
        fn $method_name:ident(self$(, $method_args:ident: $method_types:ty)* $(,)?);
        $($rest:tt)*
    ) => {
        compile_error!("Components cannot define methods taking self. Did you mean &self or &mut self instead?");
    };
    (
        $(#[$meta: meta])*
        fn $method_name:ident($($method_args:ident: $method_types:ty),* $(,)?) -> $method_output:ty;
        $($rest:tt)*
    ) => {
        compile_error!("The external_component! macro cannot be used to define static blueprint methods which don't take &self or &mut self. For these package methods, use a separate external_blueprint! macro.");
    };
    (
        $(#[$meta: meta])*
        fn $method_name:ident($($method_args:ident: $method_types:ty),* $(,)?);
        $($rest:tt)*
    ) => {
        compile_error!("The external_component! macro cannot be used to define static blueprint methods which don't take &self or &mut self. For these package methods, use a separate external_blueprint! macro.");
    };
    () => {}
}
