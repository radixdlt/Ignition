#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProgrammaticScryptoSborValueEnum {
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
    #[serde(rename = "variant_id")]
    pub variant_id: i32,
    #[serde(rename = "variant_name", skip_serializing_if = "Option::is_none")]
    pub variant_name: Option<String>,
    #[serde(rename = "fields")]
    pub fields: Vec<crate::models::ProgrammaticScryptoSborValue>,
}

impl ProgrammaticScryptoSborValueEnum {
    pub fn new(
        kind: crate::models::ProgrammaticScryptoSborValueKind,
        variant_id: i32,
        fields: Vec<crate::models::ProgrammaticScryptoSborValue>,
    ) -> ProgrammaticScryptoSborValueEnum {
        ProgrammaticScryptoSborValueEnum {
            kind,
            type_name: None,
            field_name: None,
            variant_id,
            variant_name: None,
            fields,
        }
    }
}
