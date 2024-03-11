macro_rules! name_indexed_struct {
    (
        $(#[$meta: meta])*
        $struct_vis: vis struct $struct_ident: ident <$generic: ident> {
            $(
                $(#[$field_meta: meta])*
                $field_vis: vis $field_ident: ident: $field_ty: ty
            ),* $(,)?
        }
    ) => {
        // Pass through the struct definition
        $(#[$meta])*
        $struct_vis struct $struct_ident <$generic> {
            $(
                $(#[$field_meta])*
                $field_vis $field_ident: $field_ty
            ),*
        }

        impl<$generic> $struct_ident<$generic> {
            // Map function
            pub fn map<F, O>(&self, mut map: F) -> $struct_ident<O>
            where
                F: FnMut(&$generic) -> O,
            {
                $struct_ident::<O> {
                    $(
                        $field_ident: map(&self.$field_ident)
                    ),*
                }
            }

            // Map owned function
            pub fn map_owned<F, O>(self, mut map: F) -> $struct_ident<O>
            where
                F: FnMut($generic) -> O,
            {
                $struct_ident::<O> {
                    $(
                        $field_ident: map(self.$field_ident)
                    ),*
                }
            }

            // Map function
            pub fn try_map<F, O, E>(&self, mut map: F) -> Result<$struct_ident<O>, E>
            where
                F: FnMut(&$generic) -> Result<O, E>,
            {
                Ok($struct_ident::<O> {
                    $(
                        $field_ident: map(&self.$field_ident)?
                    ),*
                })
            }

            // Zip two together
            pub fn zip<Other>(self, other: $struct_ident<Other>) -> $struct_ident<($generic, Other)> {
                $struct_ident {
                    $(
                        $field_ident: (self.$field_ident, other.$field_ident)
                    ),*
                }
            }

            pub fn zip_borrowed<Other>(self, other: &$struct_ident<Other>) -> $struct_ident<($generic, &Other)> {
                $struct_ident {
                    $(
                        $field_ident: (self.$field_ident, &other.$field_ident)
                    ),*
                }
            }

            // Creating from a map
            pub fn from_map<M, S>(
                map: M
            ) -> Result<Self, $crate::publishing::KeyNotFound>
            where
                M: IntoIterator<Item = (S, $generic)>,
                S: AsRef<str>
            {
                $(
                    let mut $field_ident = None::<$generic>;
                )*

                for (key, value) in map.into_iter() {
                    match key.as_ref() {
                        $(
                            stringify!($field_ident) => {
                                $field_ident = Some(value)
                            }
                        ),*
                        _ => {}
                    }
                }

                $(
                    let $field_ident = $field_ident
                        .ok_or_else(
                            || $crate::publishing::KeyNotFound {
                                key: stringify!($field_ident).to_owned()
                            }
                        )?;
                )*

                Ok($struct_ident {
                    $(
                        $field_ident
                    ),*
                })
            }

            pub fn iter(&self) -> impl Iterator<Item = &$generic> {
                vec![
                    $(
                        &self.$field_ident
                    ),*
                ].into_iter()
            }

            // Creating a map of everything in the name indexed struct
            pub fn into_map(&self) -> ::radix_engine_common::prelude::IndexMap<&'static str, &$generic> {
                let mut map = ::radix_engine_common::prelude::IndexMap::<&'static str, &$generic>::new();

                $(
                    map.insert(stringify!($field_ident), &self.$field_ident);
                )*

                map
            }
        }
    };
}
pub(super) use name_indexed_struct;
