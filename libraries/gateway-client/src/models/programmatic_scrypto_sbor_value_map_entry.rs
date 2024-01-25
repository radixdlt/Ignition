#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProgrammaticScryptoSborValueMapEntry {
    #[serde(rename = "key")]
    pub key: Box<crate::models::ProgrammaticScryptoSborValue>,
    #[serde(rename = "value")]
    pub value: Box<crate::models::ProgrammaticScryptoSborValue>,
}

impl ProgrammaticScryptoSborValueMapEntry {
    pub fn new(
        key: crate::models::ProgrammaticScryptoSborValue,
        value: crate::models::ProgrammaticScryptoSborValue,
    ) -> ProgrammaticScryptoSborValueMapEntry {
        ProgrammaticScryptoSborValueMapEntry {
            key: Box::new(key),
            value: Box::new(value),
        }
    }
}
