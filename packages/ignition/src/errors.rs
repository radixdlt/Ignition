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
    NEITHER_POOL_ASSET_IS_PROTOCOL_RESOURCE_ERROR
        => "Neither pool asset is the protocol resource.";
    NO_ASSOCIATED_VAULT_ERROR
        => "The resource has no associated vault in the protocol.";
    NO_ASSOCIATED_LIQUIDITY_RECEIPT_VAULT_ERROR
        => "The liquidity receipt has no associated vault in the protocol.";
    NOT_AN_IGNITION_ADDRESS_ERROR
        => "The passed allocated address is not an ignition address.";
}
