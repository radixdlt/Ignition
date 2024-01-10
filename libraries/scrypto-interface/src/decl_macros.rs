macro_rules! impl_enum_parse {
    (
        $(#[$meta: meta])*
        $vis: vis enum $ident: ident {
            $(
                $variant: ident
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $ident {
            $(
                $variant
            ),*
        }

        const _: () = {
            impl ::core::convert::TryFrom<::syn::Ident> for $ident {
                type Error = ::syn::Error;

                fn try_from(ident: ::syn::Ident) -> ::syn::Result<$ident> {
                    match ident.to_string().as_str() {
                        $(
                            stringify!($variant) => Ok(Self::$variant),
                        )*
                        _ => Err(::syn::Error::new(
                            ident.span(),
                            format!("\"{}\" is not a valid \"{}\". Valid values are: {:?}", ident, stringify!($ident), $ident::STRINGS)
                        ))
                    }
                }
            }

            impl $ident {
                pub const STRINGS: &'static [&'static str] = &[
                    $(
                        stringify!($variant)
                    ),*
                ];

                pub const ALL: &'static [$ident] = &[
                    $(
                        Self::$variant
                    ),*
                ];
            }

            impl ::syn::parse::Parse for $ident {
                fn parse(input: ParseStream) -> Result<Self> {
                    <$ident>::try_from(Ident::parse(input)?)
                }
            }
        };
    };
}
