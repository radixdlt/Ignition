#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProgrammaticScryptoSborValueU8 {
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
    #[serde(rename = "value")]
    pub value: String,
}

impl ProgrammaticScryptoSborValueU8 {
    pub fn new(
        kind: crate::models::ProgrammaticScryptoSborValueKind,
        value: String,
    ) -> ProgrammaticScryptoSborValueU8 {
        ProgrammaticScryptoSborValueU8 {
            kind,
            type_name: None,
            field_name: None,
            value,
        }
    }
}
