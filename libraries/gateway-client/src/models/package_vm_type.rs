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
pub enum PackageVmType {
    #[serde(rename = "Native")]
    Native,
    #[serde(rename = "ScryptoV1")]
    ScryptoV1,
}

impl ToString for PackageVmType {
    fn to_string(&self) -> String {
        match self {
            Self::Native => String::from("Native"),
            Self::ScryptoV1 => String::from("ScryptoV1"),
        }
    }
}

impl Default for PackageVmType {
    fn default() -> PackageVmType {
        Self::Native
    }
}
