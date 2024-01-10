#[macro_use]
mod decl_macros;
mod handlers;
mod types;

use proc_macro::TokenStream;

#[proc_macro]
pub fn define_interface(input: TokenStream) -> TokenStream {
    handlers::handle_define_interface(input.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_attribute]
pub fn blueprint_with_traits(
    meta: TokenStream,
    item: TokenStream,
) -> TokenStream {
    handlers::handle_blueprint_with_traits(meta.into(), item.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
