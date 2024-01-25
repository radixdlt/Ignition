#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PublicKeyHashEcdsaSecp256k1 {
    #[serde(rename = "key_hash_type")]
    pub key_hash_type: crate::models::PublicKeyHashType,

    #[serde(rename = "hash_hex")]
    pub hash_hex: String,
}

impl PublicKeyHashEcdsaSecp256k1 {
    pub fn new(
        key_hash_type: crate::models::PublicKeyHashType,
        hash_hex: String,
    ) -> PublicKeyHashEcdsaSecp256k1 {
        PublicKeyHashEcdsaSecp256k1 {
            key_hash_type,
            hash_hex,
        }
    }
}
