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
pub struct StateNonFungibleDetailsResponseItem {
    #[serde(rename = "is_burned")]
    pub is_burned: bool,

    #[serde(rename = "non_fungible_id")]
    pub non_fungible_id: String,
    #[serde(rename = "data", skip_serializing_if = "Option::is_none")]
    pub data: Option<Box<crate::models::ScryptoSborValue>>,

    #[serde(rename = "last_updated_at_state_version")]
    pub last_updated_at_state_version: i64,
}

impl StateNonFungibleDetailsResponseItem {
    pub fn new(
        is_burned: bool,
        non_fungible_id: String,
        last_updated_at_state_version: i64,
    ) -> StateNonFungibleDetailsResponseItem {
        StateNonFungibleDetailsResponseItem {
            is_burned,
            non_fungible_id,
            data: None,
            last_updated_at_state_version,
        }
    }
}
