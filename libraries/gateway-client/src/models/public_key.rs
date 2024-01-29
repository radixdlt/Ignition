#[serde_with::serde_as]
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "key_type")]
pub enum PublicKey {
    EcdsaSecp256k1 {
        #[serde_as(as = "serde_with::hex::Hex")]
        #[serde(rename = "key_hex")]
        key: [u8; 33],
    },
    EddsaEd25519 {
        #[serde_as(as = "serde_with::hex::Hex")]
        #[serde(rename = "key_hex")]
        key: [u8; 32],
    },
}
