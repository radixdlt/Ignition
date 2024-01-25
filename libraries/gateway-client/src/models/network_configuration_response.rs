#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct NetworkConfigurationResponse {
    #[serde(rename = "network_id")]
    pub network_id: i32,

    #[serde(rename = "network_name")]
    pub network_name: String,
    #[serde(rename = "well_known_addresses")]
    pub well_known_addresses:
        Box<crate::models::NetworkConfigurationResponseWellKnownAddresses>,
}

impl NetworkConfigurationResponse {
    pub fn new(
        network_id: i32,
        network_name: String,
        well_known_addresses: crate::models::NetworkConfigurationResponseWellKnownAddresses,
    ) -> NetworkConfigurationResponse {
        NetworkConfigurationResponse {
            network_id,
            network_name,
            well_known_addresses: Box::new(well_known_addresses),
        }
    }
}
