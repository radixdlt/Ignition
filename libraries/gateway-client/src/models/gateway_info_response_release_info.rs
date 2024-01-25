#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GatewayInfoResponseReleaseInfo {
    #[serde(rename = "release_version")]
    pub release_version: String,

    #[serde(rename = "open_api_schema_version")]
    pub open_api_schema_version: String,

    #[serde(rename = "image_tag")]
    pub image_tag: String,
}

impl GatewayInfoResponseReleaseInfo {
    pub fn new(
        release_version: String,
        open_api_schema_version: String,
        image_tag: String,
    ) -> GatewayInfoResponseReleaseInfo {
        GatewayInfoResponseReleaseInfo {
            release_version,
            open_api_schema_version,
            image_tag,
        }
    }
}
