use proc_macro::*;
use radix_engine_interface::prelude::*;

macro_rules! tri {
    ($expr: expr) => {
        match $expr {
            Ok(item) => item,
            Err(err) => return err.into_compile_error().into(),
        }
    };
}

macro_rules! impl_address_proc_macro {
    (
        $type_ident: ident
    ) => {
        paste::paste! {
            #[proc_macro]
            pub fn [< $type_ident: snake >](item: TokenStream) -> TokenStream {
                let literal_string = tri!(syn::parse::<syn::LitStr>(item));
                let node_id = tri!(decode_string_into_node_id(&literal_string));
                let node_id_bytes = node_id.0;
                let _ = tri!($type_ident::try_from(node_id_bytes).map_err(|err| {
                    syn::Error::new_spanned(&literal_string, format!("{err:?}"))
                }));
                quote::quote! {
                    ::radix_engine_interface::prelude::$type_ident::new_or_panic(
                        [ #(#node_id_bytes),* ]
                    )
                }
                .into()
            }
        }
    };
}

impl_address_proc_macro!(ComponentAddress);
impl_address_proc_macro!(ResourceAddress);
impl_address_proc_macro!(PackageAddress);
impl_address_proc_macro!(InternalAddress);
impl_address_proc_macro!(GlobalAddress);

#[proc_macro]
pub fn node_id(item: TokenStream) -> TokenStream {
    let literal_string = tri!(syn::parse::<syn::LitStr>(item));
    let node_id = tri!(decode_string_into_node_id(&literal_string));
    let node_id_bytes = node_id.0;

    quote::quote! {
        ::radix_engine_interface::prelude::NodeId([ #(#node_id_bytes),* ])
    }
    .into()
}

fn decode_string_into_node_id(
    address: &syn::LitStr,
) -> Result<NodeId, syn::Error> {
    // Attempt to decode the value without network context. Error out if we
    // can't decode it.
    let (_, _, node_id_bytes) =
        AddressBech32Decoder::validate_and_decode_ignore_hrp(&address.value())
            .map_err(|err| {
                syn::Error::new_spanned(address, format!("{err:?}"))
            })?;

    // Try to convert this into a NodeId which can fail due to the length.
    node_id_bytes.try_into().map(NodeId).map_err(|_| {
        syn::Error::new_spanned(address, "Address length is invalid")
    })
}
