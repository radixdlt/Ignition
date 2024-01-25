#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct OptionalNonFungibleIdsCollection {
    #[serde(
        rename = "total_count",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub total_count: Option<Option<i64>>,

    #[serde(
        rename = "next_cursor",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub next_cursor: Option<Option<String>>,
    #[serde(rename = "items", skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<String>>,
}

impl Default for OptionalNonFungibleIdsCollection {
    fn default() -> Self {
        Self::new()
    }
}

impl OptionalNonFungibleIdsCollection {
    pub fn new() -> OptionalNonFungibleIdsCollection {
        OptionalNonFungibleIdsCollection {
            total_count: None,
            next_cursor: None,
            items: None,
        }
    }
}
