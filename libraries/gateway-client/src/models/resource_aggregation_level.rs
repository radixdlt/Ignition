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
pub enum ResourceAggregationLevel {
    #[serde(rename = "Global")]
    Global,
    #[serde(rename = "Vault")]
    Vault,
}

impl ToString for ResourceAggregationLevel {
    fn to_string(&self) -> String {
        match self {
            Self::Global => String::from("Global"),
            Self::Vault => String::from("Vault"),
        }
    }
}

impl Default for ResourceAggregationLevel {
    fn default() -> ResourceAggregationLevel {
        Self::Global
    }
}
