use proc_macro2::TokenStream as TokenStream2;
use syn::spanned::*;
use syn::*;

pub fn handle_define_interface(_: TokenStream2) -> syn::Result<TokenStream2> {
    todo!()
}

pub fn handle_blueprint_with_traits(
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

                #[allow(unused_variables)]
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

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use proc_macro2::TokenStream as TokenStream2;

    use super::handle_blueprint_with_traits;

    #[test]
    fn blueprint_with_trait_generates_expected_code() {
        // Arrange
        let input = r#"
        #[blueprint_with_traits]
        mod blueprint{
            struct Blueprint;

            impl Blueprint {}

            impl MyTrait for Blueprint {
                fn func1() {
                    todo!("func1");
                }
                fn func2(item: u32) {
                    todo!("func2");
                }
                fn func3() -> u32 {
                    todo!("func3");
                }
                fn func4(item: u32) -> u32 {
                    todo!("func4");
                }

                fn ref_method1(&self) {
                    todo!("ref_method1");
                }
                fn ref_method2(&self, item: u32) {
                    todo!("ref_method2");
                }
                fn ref_method3(&self) -> u32 {
                    todo!("ref_method3");
                }
                fn ref_method4(&self, item: u32) -> u32 {
                    todo!("ref_method4");
                }

                fn mut_ref_method1(&mut self) {
                    todo!("mut_ref_method1");
                }
                fn mut_ref_method2(&mut self, item: u32) {
                    todo!("mut_ref_method2");
                }
                fn mut_ref_method3(&mut self) -> u32 {
                    todo!("mut_ref_method3");
                }
                fn mut_ref_method4(&mut self, item: u32) -> u32 {
                    todo!("mut_ref_method4");
                }
            }
        }
        "#;
        let expected_output = r#"
        #[::scrypto::prelude::blueprint] 
        #[blueprint_with_traits]
        mod blueprint{
            struct Blueprint;

            impl Blueprint {
                pub fn func1() {
                    todo!("func1");
                }
                pub fn func2(item: u32) {
                    todo!("func2");
                }
                pub fn func3() -> u32 {
                    todo!("func3");
                }
                pub fn func4(item: u32) -> u32 {
                    todo!("func4");
                }

                pub fn ref_method1(&self) {
                    todo!("ref_method1");
                }
                pub fn ref_method2(&self, item: u32) {
                    todo!("ref_method2");
                }
                pub fn ref_method3(&self) -> u32 {
                    todo!("ref_method3");
                }
                pub fn ref_method4(&self, item: u32) -> u32 {
                    todo!("ref_method4");
                }

                pub fn mut_ref_method1(&mut self) {
                    todo!("mut_ref_method1");
                }
                pub fn mut_ref_method2(&mut self, item: u32) {
                    todo!("mut_ref_method2");
                }
                pub fn mut_ref_method3(&mut self) -> u32 {
                    todo!("mut_ref_method3");
                }
                pub fn mut_ref_method4(&mut self, item: u32) -> u32 {
                    todo!("mut_ref_method4");
                }
            }
        }

        const _: () = {
            struct Blueprint;

            #[allow (unused_variables)]
            impl MyTrait for Blueprint {
                fn func1() {
                    unreachable!()
                }
                fn func2(item: u32) {
                    unreachable!()
                }
                fn func3() -> u32 {
                    unreachable!()
                }
                fn func4(item: u32) -> u32 {
                    unreachable!()
                }

                fn ref_method1(&self) {
                    unreachable!()
                }
                fn ref_method2(&self, item: u32) {
                    unreachable!()
                }
                fn ref_method3(&self) -> u32 {
                    unreachable!()
                }
                fn ref_method4(&self, item: u32) -> u32 {
                    unreachable!()
                }

                fn mut_ref_method1(&mut self) {
                    unreachable!()
                }
                fn mut_ref_method2(&mut self, item: u32) {
                    unreachable!()
                }
                fn mut_ref_method3(&mut self) -> u32 {
                    unreachable!()
                }
                fn mut_ref_method4(&mut self, item: u32) -> u32 {
                    unreachable!()
                }
            }
        };
        "#;

        // Act
        let output = handle_blueprint_with_traits(
            TokenStream2::from_str("").unwrap(),
            TokenStream2::from_str(input).unwrap(),
        )
        .unwrap();

        // Assert
        assert_eq!(
            output.to_string(),
            TokenStream2::from_str(expected_output).unwrap().to_string()
        );
    }
}
