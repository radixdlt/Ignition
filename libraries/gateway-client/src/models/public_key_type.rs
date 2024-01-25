#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum PublicKeyType {
    #[serde(rename = "EcdsaSecp256k1")]
    EcdsaSecp256k1,
    #[serde(rename = "EddsaEd25519")]
    EddsaEd25519,
}

impl ToString for PublicKeyType {
    fn to_string(&self) -> String {
        match self {
            Self::EcdsaSecp256k1 => String::from("EcdsaSecp256k1"),
            Self::EddsaEd25519 => String::from("EddsaEd25519"),
        }
    }
}

impl Default for PublicKeyType {
    fn default() -> PublicKeyType {
        Self::EcdsaSecp256k1
    }
}
