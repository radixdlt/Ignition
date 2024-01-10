use proc_macro2::TokenStream as TokenStream2;

use syn::*;

pub fn handle_define_interface(_: TokenStream2) -> syn::Result<TokenStream2> {
    todo!()
}

pub fn handle_blueprint_with_traits(
    _: TokenStream2,
    _: TokenStream2,
) -> Result<TokenStream2> {
    todo!()
}

#[cfg(test)]
mod test {}
