#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StateEntityDetailsResponsePackageDetailsBlueprintCollection {
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
    #[serde(rename = "items")]
    pub items: Vec<
        crate::models::StateEntityDetailsResponsePackageDetailsBlueprintItem,
    >,
}

impl StateEntityDetailsResponsePackageDetailsBlueprintCollection {
    pub fn new(
        items: Vec<crate::models::StateEntityDetailsResponsePackageDetailsBlueprintItem>,
    ) -> StateEntityDetailsResponsePackageDetailsBlueprintCollection {
        StateEntityDetailsResponsePackageDetailsBlueprintCollection {
            total_count: None,
            next_cursor: None,
            items,
        }
    }
}
