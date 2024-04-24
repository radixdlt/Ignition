// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

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
