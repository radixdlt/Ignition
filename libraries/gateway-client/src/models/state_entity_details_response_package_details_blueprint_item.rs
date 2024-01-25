#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StateEntityDetailsResponsePackageDetailsBlueprintItem {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "version")]
    pub version: String,

    #[serde(rename = "definition")]
    pub definition: serde_json::Value,
    #[serde(
        rename = "dependant_entities",
        skip_serializing_if = "Option::is_none"
    )]
    pub dependant_entities: Option<Vec<String>>,

    #[serde(rename = "auth_template", skip_serializing_if = "Option::is_none")]
    pub auth_template: Option<serde_json::Value>,
    #[serde(
        rename = "auth_template_is_locked",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub auth_template_is_locked: Option<Option<bool>>,

    #[serde(
        rename = "royalty_config",
        skip_serializing_if = "Option::is_none"
    )]
    pub royalty_config: Option<serde_json::Value>,
    #[serde(
        rename = "royalty_config_is_locked",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub royalty_config_is_locked: Option<Option<bool>>,
}

impl StateEntityDetailsResponsePackageDetailsBlueprintItem {
    pub fn new(
        name: String,
        version: String,
        definition: serde_json::Value,
    ) -> StateEntityDetailsResponsePackageDetailsBlueprintItem {
        StateEntityDetailsResponsePackageDetailsBlueprintItem {
            name,
            version,
            definition,
            dependant_entities: None,
            auth_template: None,
            auth_template_is_locked: None,
            royalty_config: None,
            royalty_config_is_locked: None,
        }
    }
}
