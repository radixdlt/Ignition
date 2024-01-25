#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct StateEntityDetailsResponsePackageDetailsSchemaItem {
    #[serde(rename = "schema_hash_hex")]
    pub schema_hash_hex: String,

    #[serde(rename = "schema_hex")]
    pub schema_hex: String,
}

impl StateEntityDetailsResponsePackageDetailsSchemaItem {
    pub fn new(
        schema_hash_hex: String,
        schema_hex: String,
    ) -> StateEntityDetailsResponsePackageDetailsSchemaItem {
        StateEntityDetailsResponsePackageDetailsSchemaItem {
            schema_hash_hex,
            schema_hex,
        }
    }
}
