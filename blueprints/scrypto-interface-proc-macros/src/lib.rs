use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;

use syn::parse::*;
use syn::spanned::*;
use syn::*;

/// A procedural blueprint macro with the added support for traits allowing for
/// compile-time checking of interfaces.
///
/// This macro performs some logic and then delegates the remaining logic to
/// scrypto's blueprint macro, thus the logic of this macro should not diverge
/// from the main blueprint macro.
///
/// This macro starts by finding all of the trait implementations inside the
/// module and removing them from there. It then copies the implementation of
/// the trait to the `impl` block of the blueprint. Then, a `const _: ()` block
/// is used to house a trait implementation for a private type.
///
/// This means that:
/// * A blueprint can contain trait implementations. These implementations do
/// not have a fixed place and can occur in any order and will be handled as
/// expected by this macro.
/// * Functions and methods implemented through traits will be made public and
/// can not be changed.
/// * Functions and methods implemented through traits will be implemented in
/// the main impl block of the blueprint and will be considered as a normal
/// public function or method.
#[proc_macro_attribute]
pub fn blueprint_with_traits(
    meta: TokenStream,
    item: TokenStream,
) -> TokenStream {
    handle_blueprint_with_traits(meta.into(), item.into())
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn handle_blueprint_with_traits(
    _: TokenStream2,
    item: TokenStream2,
) -> Result<TokenStream2> {
    // Parse the passed token stream as a module. After we do that, we will
    // remove all of the trait impls from inside.
    let span = item.span();
    let mut module = syn::parse2::<ItemMod>(item)?;
    let trait_impls = if let Some((brace, items)) = module.content {
        let (trait_impls, items) =
            items.into_iter().partition::<Vec<_>, _>(|item| {
                matches!(
                    item,
                    Item::Impl(ItemImpl {
                        trait_: Some(_),
                        ..
                    })
                )
            });
        module.content = Some((brace, items));
        trait_impls
    } else {
        vec![]
    };

    // Find the impl block in the module that is not for a trait and then add
    // all of the trait implementations to it.
    if let Some((_, ref mut items)) = module.content {
        let impl_item = items
            .iter_mut()
            .filter_map(|item| {
                if let Item::Impl(item_impl @ ItemImpl { trait_: None, .. }) =
                    item
                {
                    Some(item_impl)
                } else {
                    None
                }
            })
            .next()
            .ok_or(syn::Error::new(
                span,
                "No impl block found that is not for a trait",
            ))?;

        for trait_impl_item in trait_impls.iter() {
            let Item::Impl(ItemImpl { items, .. }) = trait_impl_item else {
                continue;
            };

            // Make any item that accepts a vis become public
            let items = items
                .iter()
                .cloned()
                .map(|item| match item {
                    ImplItem::Const(mut item) => {
                        item.vis = Visibility::Public(Token![pub](span));
                        ImplItem::Const(item)
                    }
                    ImplItem::Fn(mut item) => {
                        item.vis = Visibility::Public(Token![pub](span));
                        ImplItem::Fn(item)
                    }
                    ImplItem::Type(mut item) => {
                        item.vis = Visibility::Public(Token![pub](span));
                        ImplItem::Type(item)
                    }
                    item @ ImplItem::Macro(..)
                    | item @ ImplItem::Verbatim(..) => item,
                    _ => todo!(),
                })
                .collect::<Vec<_>>();

            impl_item.items.extend(items)
        }
    }

    if let Some((_, ref items)) = module.content {
        // Getting the name of the blueprint by finding the first struct item we find inside the
        // module.
        let blueprint_ident = items
            .iter()
            .filter_map(|item| {
                if let Item::Struct(ItemStruct { ident, .. }) = item {
                    Some(ident)
                } else {
                    None
                }
            })
            .next()
            .ok_or(syn::Error::new(
                span,
                "No struct item found inside of module",
            ))?;

        let unreachable_trait_impls = trait_impls
            .clone()
            .into_iter()
            .filter_map(|item| {
                if let Item::Impl(item) = item {
                    Some(item)
                } else {
                    None
                }
            })
            .map(|mut impl_item| {
                impl_item.items = impl_item
                    .items
                    .into_iter()
                    .map(|mut impl_item| {
                        if let ImplItem::Fn(ref mut func_impl_item) = impl_item
                        {
                            func_impl_item.block =
                                parse_quote!({ unreachable!() });
                        };
                        impl_item
                    })
                    .collect();
                impl_item
            });

        // The module should now be a perfectly well structured blueprint that
        // is ready to go through the blueprint code generation process.
        Ok(quote::quote! {
            #[::scrypto::prelude::blueprint]
            #module

            const _: () = {
                struct #blueprint_ident;

                #(#unreachable_trait_impls)*
            };
        })
    } else {
        // The module should now be a perfectly well structured blueprint that
        // is ready to go through the blueprint code generation process.
        Ok(quote::quote! {
            #[::scrypto::prelude::blueprint]
            #module
        })
    }
}
