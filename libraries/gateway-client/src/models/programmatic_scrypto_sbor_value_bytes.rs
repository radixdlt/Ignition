#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProgrammaticScryptoSborValueBytes {
    #[serde(rename = "kind")]
    pub kind: crate::models::ProgrammaticScryptoSborValueKind,

    #[serde(
        rename = "type_name",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub type_name: Option<Option<String>>,

    #[serde(
        rename = "field_name",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub field_name: Option<Option<String>>,
    #[serde(rename = "element_kind")]
    pub element_kind: crate::models::ProgrammaticScryptoSborValueKind,
    #[serde(
        rename = "element_type_name",
        skip_serializing_if = "Option::is_none"
    )]
    pub element_type_name: Option<String>,

    #[serde(rename = "hex")]
    pub hex: String,
}

impl ProgrammaticScryptoSborValueBytes {
    pub fn new(
        kind: crate::models::ProgrammaticScryptoSborValueKind,
        element_kind: crate::models::ProgrammaticScryptoSborValueKind,
        hex: String,
    ) -> ProgrammaticScryptoSborValueBytes {
        ProgrammaticScryptoSborValueBytes {
            kind,
            type_name: None,
            field_name: None,
            element_kind,
            element_type_name: None,
            hex,
        }
    }
}
