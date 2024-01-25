#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProgrammaticScryptoSborValueTuple {
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
    #[serde(rename = "fields")]
    pub fields: Vec<crate::models::ProgrammaticScryptoSborValue>,
}

impl ProgrammaticScryptoSborValueTuple {
    pub fn new(
        kind: crate::models::ProgrammaticScryptoSborValueKind,
        fields: Vec<crate::models::ProgrammaticScryptoSborValue>,
    ) -> ProgrammaticScryptoSborValueTuple {
        ProgrammaticScryptoSborValueTuple {
            kind,
            type_name: None,
            field_name: None,
            fields,
        }
    }
}
