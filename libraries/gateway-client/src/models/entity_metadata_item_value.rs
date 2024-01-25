#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EntityMetadataItemValue {
    #[serde(rename = "raw_hex")]
    pub raw_hex: String,
    #[serde(rename = "programmatic_json")]
    pub programmatic_json: Box<crate::models::ProgrammaticScryptoSborValue>,
    #[serde(rename = "typed")]
    pub typed: Box<crate::models::MetadataTypedValue>,
}

impl EntityMetadataItemValue {
    pub fn new(
        raw_hex: String,
        programmatic_json: crate::models::ProgrammaticScryptoSborValue,
        typed: crate::models::MetadataTypedValue,
    ) -> EntityMetadataItemValue {
        EntityMetadataItemValue {
            raw_hex,
            programmatic_json: Box::new(programmatic_json),
            typed: Box::new(typed),
        }
    }
}
