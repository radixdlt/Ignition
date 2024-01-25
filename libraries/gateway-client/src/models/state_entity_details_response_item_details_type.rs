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
pub enum StateEntityDetailsResponseItemDetailsType {
    #[serde(rename = "FungibleResource")]
    FungibleResource,
    #[serde(rename = "NonFungibleResource")]
    NonFungibleResource,
    #[serde(rename = "FungibleVault")]
    FungibleVault,
    #[serde(rename = "NonFungibleVault")]
    NonFungibleVault,
    #[serde(rename = "Package")]
    Package,
    #[serde(rename = "Component")]
    Component,
}

impl ToString for StateEntityDetailsResponseItemDetailsType {
    fn to_string(&self) -> String {
        match self {
            Self::FungibleResource => String::from("FungibleResource"),
            Self::NonFungibleResource => String::from("NonFungibleResource"),
            Self::FungibleVault => String::from("FungibleVault"),
            Self::NonFungibleVault => String::from("NonFungibleVault"),
            Self::Package => String::from("Package"),
            Self::Component => String::from("Component"),
        }
    }
}

impl Default for StateEntityDetailsResponseItemDetailsType {
    fn default() -> StateEntityDetailsResponseItemDetailsType {
        Self::FungibleResource
    }
}
