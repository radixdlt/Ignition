#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PublicKeyEddsaEd25519 {
    #[serde(rename = "key_type")]
    pub key_type: crate::models::PublicKeyType,

    #[serde(rename = "key_hex")]
    pub key_hex: String,
}

impl PublicKeyEddsaEd25519 {
    pub fn new(
        key_type: crate::models::PublicKeyType,
        key_hex: String,
    ) -> PublicKeyEddsaEd25519 {
        PublicKeyEddsaEd25519 { key_type, key_hex }
    }
}
