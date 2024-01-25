#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PublicKeyEcdsaSecp256k1 {
    #[serde(rename = "key_type")]
    pub key_type: crate::models::PublicKeyType,

    #[serde(rename = "key_hex")]
    pub key_hex: String,
}

impl PublicKeyEcdsaSecp256k1 {
    pub fn new(
        key_type: crate::models::PublicKeyType,
        key_hex: String,
    ) -> PublicKeyEcdsaSecp256k1 {
        PublicKeyEcdsaSecp256k1 { key_type, key_hex }
    }
}
