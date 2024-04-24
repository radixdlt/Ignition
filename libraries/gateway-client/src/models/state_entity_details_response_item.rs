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
pub struct StateEntityDetailsResponseItem {
    #[serde(rename = "address")]
    pub address: String,
    #[serde(
        rename = "fungible_resources",
        skip_serializing_if = "Option::is_none"
    )]
    pub fungible_resources:
        Option<Box<crate::models::FungibleResourcesCollection>>,
    #[serde(
        rename = "non_fungible_resources",
        skip_serializing_if = "Option::is_none"
    )]
    pub non_fungible_resources:
        Option<Box<crate::models::NonFungibleResourcesCollection>>,
    #[serde(
        rename = "ancestor_identities",
        skip_serializing_if = "Option::is_none"
    )]
    pub ancestor_identities: Option<
        Box<crate::models::StateEntityDetailsResponseItemAncestorIdentities>,
    >,
    #[serde(rename = "metadata")]
    pub metadata: Box<crate::models::EntityMetadataCollection>,
    #[serde(
        rename = "explicit_metadata",
        skip_serializing_if = "Option::is_none"
    )]
    pub explicit_metadata: Option<Box<crate::models::EntityMetadataCollection>>,
    #[serde(rename = "details", skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl StateEntityDetailsResponseItem {
    pub fn new(
        address: String,
        metadata: crate::models::EntityMetadataCollection,
    ) -> StateEntityDetailsResponseItem {
        StateEntityDetailsResponseItem {
            address,
            fungible_resources: None,
            non_fungible_resources: None,
            ancestor_identities: None,
            metadata: Box::new(metadata),
            explicit_metadata: None,
            details: None,
        }
    }
}
