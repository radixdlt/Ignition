#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct MetadataNonFungibleGlobalIdArrayValueAllOfValues {
    #[serde(rename = "resource_address")]
    pub resource_address: String,

    #[serde(rename = "non_fungible_id")]
    pub non_fungible_id: String,
}

impl MetadataNonFungibleGlobalIdArrayValueAllOfValues {
    pub fn new(
        resource_address: String,
        non_fungible_id: String,
    ) -> MetadataNonFungibleGlobalIdArrayValueAllOfValues {
        MetadataNonFungibleGlobalIdArrayValueAllOfValues {
            resource_address,
            non_fungible_id,
        }
    }
}
