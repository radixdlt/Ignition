#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ScryptoSborValue {
    #[serde(rename = "raw_hex")]
    pub raw_hex: String,
    #[serde(rename = "programmatic_json")]
    pub programmatic_json: Box<crate::models::ProgrammaticScryptoSborValue>,
}

impl ScryptoSborValue {
    pub fn new(
        raw_hex: String,
        programmatic_json: crate::models::ProgrammaticScryptoSborValue,
    ) -> ScryptoSborValue {
        ScryptoSborValue {
            raw_hex,
            programmatic_json: Box::new(programmatic_json),
        }
    }
}
