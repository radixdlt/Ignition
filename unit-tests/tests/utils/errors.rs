#![allow(dead_code)]

use scrypto_test::prelude::*;

pub fn is_wasm_panic<T>(result: &Result<T, RuntimeError>) -> bool {
    matches!(
        result,
        Err(RuntimeError::ApplicationError(
            ApplicationError::PanicMessage(..)
        ))
    )
}

pub fn is_wasm_panic_error_contains<T>(
    result: &Result<T, RuntimeError>,
    str: &str,
) -> bool {
    matches!(
        result,
        Err(RuntimeError::ApplicationError(
            ApplicationError::PanicMessage(error)
        ))
        if error.contains(str)
    )
}

macro_rules! define_error_functions {
    (
        $(
            $func_name: ident => $string: expr;
        )*
    ) => {
        $(
            paste::paste! {
                pub fn $func_name<T>(result: &Result<T, RuntimeError>) -> bool
                where
                    T: Debug
                {
                    is_wasm_panic_error_contains(result, $string)
                }

                pub fn [< assert_ $func_name >]<T>(
                    result: &Result<T, RuntimeError>
                )
                where
                    T: Debug
                {
                    assert!(
                        $func_name(result),
                        "Running \"{}\" failed for result: {:#?}",
                        stringify!($func_name),
                        result
                    )
                }
            }
        )*
    };
}

define_error_functions! {
    is_open_liquidity_position_opening_disabled_error
        => "Opening liquidity positions is not allowed at this time.";
    is_open_liquidity_position_pool_not_allowed_error
        => "is not found in the list of allowed pools";
    is_open_liquidity_position_not_a_valid_lockup_period_error
        => "No reward percentage associated with lockup period.";
    is_open_liquidity_position_no_adapter_error
        => "No adapter found for liquidity pool";
    is_open_liquidity_position_price_difference_too_large_error
        => "when the maximum allowed is:";
    is_open_liquidity_position_neither_side_is_xrd_error
        => "Neither side of the pool is XRD";
    is_add_allowed_pool_no_adapter_found_for_pool
        => "No adapter found for component";
}
