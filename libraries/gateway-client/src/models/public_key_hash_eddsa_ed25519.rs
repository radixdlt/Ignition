#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PublicKeyHashEddsaEd25519 {
    #[serde(rename = "key_hash_type")]
    pub key_hash_type: crate::models::PublicKeyHashType,

    #[serde(rename = "hash_hex")]
    pub hash_hex: String,
}

impl PublicKeyHashEddsaEd25519 {
    pub fn new(
        key_hash_type: crate::models::PublicKeyHashType,
        hash_hex: String,
    ) -> PublicKeyHashEddsaEd25519 {
        PublicKeyHashEddsaEd25519 {
            key_hash_type,
            hash_hex,
        }
    }
}
