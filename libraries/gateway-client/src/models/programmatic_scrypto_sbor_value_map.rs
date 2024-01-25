#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProgrammaticScryptoSborValueMap {
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
    #[serde(rename = "key_kind")]
    pub key_kind: crate::models::ProgrammaticScryptoSborValueKind,
    #[serde(rename = "key_type_name", skip_serializing_if = "Option::is_none")]
    pub key_type_name: Option<String>,
    #[serde(rename = "value_kind")]
    pub value_kind: crate::models::ProgrammaticScryptoSborValueKind,
    #[serde(
        rename = "value_type_name",
        skip_serializing_if = "Option::is_none"
    )]
    pub value_type_name: Option<String>,
    #[serde(rename = "entries")]
    pub entries: Vec<crate::models::ProgrammaticScryptoSborValueMapEntry>,
}

impl ProgrammaticScryptoSborValueMap {
    pub fn new(
        kind: crate::models::ProgrammaticScryptoSborValueKind,
        key_kind: crate::models::ProgrammaticScryptoSborValueKind,
        value_kind: crate::models::ProgrammaticScryptoSborValueKind,
        entries: Vec<crate::models::ProgrammaticScryptoSborValueMapEntry>,
    ) -> ProgrammaticScryptoSborValueMap {
        ProgrammaticScryptoSborValueMap {
            kind,
            type_name: None,
            field_name: None,
            key_kind,
            key_type_name: None,
            value_kind,
            value_type_name: None,
            entries,
        }
    }
}
