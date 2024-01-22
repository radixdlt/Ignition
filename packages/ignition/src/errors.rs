macro_rules! define_error {
    (
        $(
            $name: ident => $item: expr;
        )*
    ) => {
        $(
            pub const $name: &'static str = concat!("[Ignition]", " ", $item);
        )*
    };
}

define_error! {
    NO_ADAPTER_FOUND_FOR_POOL_ERROR
        => "No adapter found for liquidity pool.";
    NEITHER_POOL_RESOURCE_IS_PROTOCOL_RESOURCE_ERROR
        => "Neither pool resource is the protocol resource.";
    NO_ASSOCIATED_VAULT_ERROR
        => "The resource has no associated vault in the protocol.";
    NO_ASSOCIATED_LIQUIDITY_RECEIPT_VAULT_ERROR
        => "The liquidity receipt has no associated vault in the protocol.";
    NOT_AN_IGNITION_ADDRESS_ERROR
        => "The passed allocated address is not an ignition address.";
    OPENING_LIQUIDITY_POSITIONS_IS_CLOSED_ERROR
        => "Opening liquidity positions is disabled.";
    CLOSING_LIQUIDITY_POSITIONS_IS_CLOSED_ERROR
        => "Closing liquidity positions is disabled.";
    NO_REWARDS_RATE_ASSOCIATED_WITH_LOCKUP_PERIOD_ERROR
        => "No rewards rate associated with lockup period.";
    POOL_IS_NOT_IN_ALLOW_LIST_ERROR
        => "Pool is not in allow list.";
    ORACLE_REPORTED_PRICE_IS_STALE_ERROR
        => "Oracle reported price is stale.";
    LOCKUP_PERIOD_HAS_NO_ASSOCIATED_REWARDS_RATE_ERROR
        => "Lockup period has no associated rewards rate.";
    UNEXPECTED_ERROR
        => "Unexpected error.";
    RELATIVE_PRICE_DIFFERENCE_LARGER_THAN_ALLOWED_ERROR
        => "Relative price difference between oracle and pool exceeds allowed.";
    USER_ASSET_DOES_NOT_BELONG_TO_POOL_ERROR
        => "The asset of the user does not belong to the pool.";
    MORE_THAN_ONE_LIQUIDITY_RECEIPT_NFTS_ERROR
        => "More than one liquidity receipt non-fungibles were provided.";
    NOT_A_VALID_LIQUIDITY_RECEIPT_ERROR
        => "Not a valid liquidity receipt resource.";
    LIQUIDITY_POSITION_HAS_NOT_MATURED_ERROR
        => "Can't close a liquidity position before it has matured.";
    USER_MUST_NOT_PROVIDE_PROTOCOL_ASSET_ERROR
        => "The user has provided the protocol asset, which is not allowed";
    USER_RESOURCES_VOLATILITY_UNKNOWN_ERROR
        => "A user resource with no registered volatility status was interacted with.";
}
