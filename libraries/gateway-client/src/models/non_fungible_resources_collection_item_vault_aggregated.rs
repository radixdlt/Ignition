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
pub struct NonFungibleResourcesCollectionItemVaultAggregated {
    #[serde(rename = "aggregation_level")]
    pub aggregation_level: crate::models::ResourceAggregationLevel,

    #[serde(rename = "resource_address")]
    pub resource_address: String,
    #[serde(
        rename = "explicit_metadata",
        skip_serializing_if = "Option::is_none"
    )]
    pub explicit_metadata: Option<Box<crate::models::EntityMetadataCollection>>,
    #[serde(rename = "vaults")]
    pub vaults: Box<
        crate::models::NonFungibleResourcesCollectionItemVaultAggregatedVault,
    >,
}

impl NonFungibleResourcesCollectionItemVaultAggregated {
    pub fn new(
        aggregation_level: crate::models::ResourceAggregationLevel,
        resource_address: String,
        vaults: crate::models::NonFungibleResourcesCollectionItemVaultAggregatedVault,
    ) -> NonFungibleResourcesCollectionItemVaultAggregated {
        NonFungibleResourcesCollectionItemVaultAggregated {
            aggregation_level,
            resource_address,
            explicit_metadata: None,
            vaults: Box::new(vaults),
        }
    }
}
