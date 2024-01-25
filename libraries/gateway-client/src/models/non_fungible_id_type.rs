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
pub enum NonFungibleIdType {
    #[serde(rename = "String")]
    String,
    #[serde(rename = "Integer")]
    Integer,
    #[serde(rename = "Bytes")]
    Bytes,
    #[serde(rename = "Ruid")]
    Ruid,
}

impl ToString for NonFungibleIdType {
    fn to_string(&self) -> String {
        match self {
            Self::String => String::from("String"),
            Self::Integer => String::from("Integer"),
            Self::Bytes => String::from("Bytes"),
            Self::Ruid => String::from("Ruid"),
        }
    }
}

impl Default for NonFungibleIdType {
    fn default() -> NonFungibleIdType {
        Self::String
    }
}
