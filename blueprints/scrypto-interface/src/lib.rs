/// A macro used to define interfaces that can be used on blueprints, their
/// scrypto stubs, and the scrypto-test stubs.
///
/// ```no_run
/// define_interface! {
///     Calculator {
///         fn add(
///             num1: u64,
///             num2: u64
///         ) -> u64;
///     }
/// }
/// ```
///
/// The created trait will have the name [< $blueprint_ident InterfaceTrait >],
/// with a post-fix of ScryptoStub and ScryptoTestStub in the case of scrypto
/// and scrypto-test respectively.
#[macro_export]
macro_rules! define_interface {
    (
        $blueprint_ident: ident {
            $($functions:tt)*
        }
    ) => {
        $crate::define_interface!($blueprint_ident as $blueprint_ident {
            $($functions)*
        });
    };
    (
        $blueprint_ident: ident as $struct_ident: ident {
            $($functions:tt)*
        }
    ) => {
        paste::paste! {
            // Creating a trait for the given interface.
            pub trait [< $blueprint_ident InterfaceTrait >] {
                $crate::handle_functions_trait!( $($functions)* );
            }

            // Creating the ScryptoStubs for the given interface.
            #[derive(
                ::radix_engine_interface::prelude::ScryptoSbor,
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
            pub struct [< $blueprint_ident InterfaceScryptoStub >](
                ::radix_engine_interface::prelude::Reference
            );

            // Creating the ScryptoTestStubs for the given interface.
            #[derive(
                ::radix_engine_interface::prelude::ScryptoSbor,
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
            pub struct [< $blueprint_ident InterfaceScryptoTestStub >](
                ::radix_engine_interface::prelude::Reference
            );

            impl [< $blueprint_ident InterfaceScryptoStub >] {
                $crate::handle_functions_scrypto_stub!( $($functions)* );

                fn call_function(
                    package_address: ::radix_engine_interface::prelude::PackageAddress,
                    function_name: &str,
                    args: Vec<u8>
                ) -> Vec<u8> {
                    ::scrypto::prelude::ScryptoVmV1Api::blueprint_call(
                        package_address,
                        stringify!($blueprint_ident),
                        function_name,
                        args
                    )
                }
            }

            impl [< $blueprint_ident InterfaceScryptoTestStub >] {
                $crate::handle_functions_scrypto_test_stub!( $($functions)* );

                fn call_function<Y, E>(
                    package_address: ::radix_engine_interface::prelude::PackageAddress,
                    function_name: &str,
                    args: Vec<u8>,
                    env: &mut Y
                ) -> Result<Vec<u8>, E>
                where
                    Y: ::radix_engine_interface::prelude::ClientApi<E>,
                    E: ::core::fmt::Debug
                {
                    env.call_function(
                        package_address,
                        stringify!($blueprint_ident),
                        function_name,
                        args
                    )
                }
            }
        }
    };
}

#[macro_export]
macro_rules! handle_functions_trait {
    (
        $($tokens: tt)*
    ) => {
        $($tokens)*
    };
}

#[macro_export]
macro_rules! handle_functions_scrypto_stub {
    (
        $(#[$meta: meta])*
        fn $fn_ident: ident (
            $($arg_ident: ident: $arg_type: ty),* $(,)?
        ) $(-> $rtn_type: ty)? ; $($functions: tt)*
    ) => {
        $(#[$meta])*
        pub fn $fn_ident(
            $($arg_ident: $arg_type,)*
            blueprint_package_address: ::radix_engine_interface::prelude::PackageAddress
        ) -> $crate::resolve_return_type!($($rtn_type)?) {
            let rtn = Self::call_function(
                blueprint_package_address,
                stringify!($fn_ident),
                ::radix_engine_interface::scrypto_args!($($arg_ident),*)
            );
            ::radix_engine_interface::prelude::scrypto_decode(&rtn).unwrap()
        }

        $crate::handle_functions_scrypto_stub!($($functions)*);
    };
    (
        $(#[$meta: meta])*
        fn $fn_ident: ident (
            &self
            $(, $arg_ident: ident: $arg_type: ty)* $(,)?
        ) $(-> $rtn_type: ty)? ; $($functions: tt)*
    ) => {
        $(#[$meta])*
        pub fn $fn_ident(
            &self
            $(, $arg_ident: $arg_type)*
        ) -> $crate::resolve_return_type!($($rtn_type)?) {
            let rtn = ::scrypto::prelude::ScryptoVmV1Api::object_call(
                &self.0.0,
                stringify!($fn_ident),
                ::radix_engine_interface::scrypto_args!($($arg_ident),*)
            );
            ::radix_engine_interface::prelude::scrypto_decode(&rtn).unwrap()
        }

        $crate::handle_functions_scrypto_stub!($($functions)*);
    };
    (
        $(#[$meta: meta])*
        fn $fn_ident: ident (
            &mut self
            $(, $arg_ident: ident: $arg_type: ty)* $(,)?
        ) $(-> $rtn_type: ty)? ; $($functions: tt)*
    ) => {
        $(#[$meta])*
        pub fn $fn_ident(
            &mut self
            $(, $arg_ident: $arg_type)*
        ) -> $crate::resolve_return_type!($($rtn_type)?) {
            let rtn = ::scrypto::prelude::ScryptoVmV1Api::object_call(
                &self.0.0,
                stringify!($fn_ident),
                ::radix_engine_interface::scrypto_args!($($arg_ident),*)
            );
            ::radix_engine_interface::prelude::scrypto_decode(&rtn).unwrap()
        }

        $crate::handle_functions_scrypto_stub!($($functions)*);
    };
    () => {}
}

#[macro_export]
macro_rules! handle_functions_scrypto_test_stub {
    (
        $(#[$meta: meta])*
        fn $fn_ident: ident (
            $($arg_ident: ident: $arg_type: ty),* $(,)?
        ) $(-> $rtn_type: ty)? ; $($functions: tt)*
    ) => {
        $(#[$meta])*
        pub fn $fn_ident<Y, E>(
            $($arg_ident: $arg_type,)*
            blueprint_package_address: ::radix_engine_interface::prelude::PackageAddress,
            env: &mut Y
        ) -> Result<$crate::resolve_return_type!($($rtn_type)?), E>
        where
            Y: ::radix_engine_interface::prelude::ClientApi<E>,
            E: ::core::fmt::Debug
        {
            Self::call_function(
                blueprint_package_address,
                stringify!($fn_ident),
                ::radix_engine_interface::scrypto_args!($($arg_ident),*),
                env
            )
            .map(|rtn| ::radix_engine_interface::prelude::scrypto_decode(&rtn).unwrap())
        }

        $crate::handle_functions_scrypto_test_stub!($($functions)*);
    };
    (
        $(#[$meta: meta])*
        fn $fn_ident: ident (
            &self
            $(, $arg_ident: ident: $arg_type: ty)* $(,)?
        ) $(-> $rtn_type: ty)? ; $($functions: tt)*
    ) => {
        $(#[$meta])*
        pub fn $fn_ident<Y, E>(
            &self
            $(, $arg_ident: $arg_type)*,
            env: &mut Y
        ) -> Result<$crate::resolve_return_type!($($rtn_type)?), E>
        where
            Y: ::radix_engine_interface::prelude::ClientApi<E>,
            E: ::core::fmt::Debug
        {
            env.call_method(
                &self.0.0,
                stringify!($fn_ident),
                ::radix_engine_interface::scrypto_args!($($arg_ident),*)
            )
            .map(|rtn| ::radix_engine_interface::prelude::scrypto_decode(&rtn).unwrap())
        }

        $crate::handle_functions_scrypto_test_stub!($($functions)*);
    };
    (
        $(#[$meta: meta])*
        fn $fn_ident: ident (
            &mut self
            $(, $arg_ident: ident: $arg_type: ty)* $(,)?
        ) $(-> $rtn_type: ty)? ; $($functions: tt)*
    ) => {
        $(#[$meta])*
        pub fn $fn_ident<Y, E>(
            &mut self
            $(, $arg_ident: $arg_type)*,
            env: &mut Y
        ) -> Result<$crate::resolve_return_type!($($rtn_type)?), E>
        where
            Y: ::radix_engine_interface::prelude::ClientApi<E>,
            E: ::core::fmt::Debug
        {
            env.call_method(
                &self.0.0,
                stringify!($fn_ident),
                ::radix_engine_interface::scrypto_args!($($arg_ident),*)
            )
            .map(|rtn| ::radix_engine_interface::prelude::scrypto_decode(&rtn).unwrap())
        }

        $crate::handle_functions_scrypto_test_stub!($($functions)*);
    };
    () => {}
}

#[macro_export]
macro_rules! resolve_return_type {
    () => {
        ()
    };
    ($type: ty) => {
        $type
    };
}