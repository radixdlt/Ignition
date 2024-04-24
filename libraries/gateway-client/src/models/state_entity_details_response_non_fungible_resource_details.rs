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
pub struct StateEntityDetailsResponseNonFungibleResourceDetails {
    #[serde(rename = "type")]
    pub r#type: crate::models::StateEntityDetailsResponseItemDetailsType,
    #[serde(rename = "role_assignments")]
    pub role_assignments: Box<crate::models::ComponentEntityRoleAssignments>,
    #[serde(rename = "non_fungible_id_type")]
    pub non_fungible_id_type: crate::models::NonFungibleIdType,

    #[serde(rename = "total_supply")]
    pub total_supply: String,

    #[serde(rename = "total_minted")]
    pub total_minted: String,

    #[serde(rename = "total_burned")]
    pub total_burned: String,
}

impl StateEntityDetailsResponseNonFungibleResourceDetails {
    pub fn new(
        r#type: crate::models::StateEntityDetailsResponseItemDetailsType,
        role_assignments: crate::models::ComponentEntityRoleAssignments,
        non_fungible_id_type: crate::models::NonFungibleIdType,
        total_supply: String,
        total_minted: String,
        total_burned: String,
    ) -> StateEntityDetailsResponseNonFungibleResourceDetails {
        StateEntityDetailsResponseNonFungibleResourceDetails {
            r#type,
            role_assignments: Box::new(role_assignments),
            non_fungible_id_type,
            total_supply,
            total_minted,
            total_burned,
        }
    }
}
