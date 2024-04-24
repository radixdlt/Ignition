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
pub struct FungibleResourcesCollectionItemVaultAggregatedVaultItem {
    #[serde(rename = "vault_address")]
    pub vault_address: String,

    #[serde(rename = "amount")]
    pub amount: String,

    #[serde(rename = "last_updated_at_state_version")]
    pub last_updated_at_state_version: i64,
}

impl FungibleResourcesCollectionItemVaultAggregatedVaultItem {
    pub fn new(
        vault_address: String,
        amount: String,
        last_updated_at_state_version: i64,
    ) -> FungibleResourcesCollectionItemVaultAggregatedVaultItem {
        FungibleResourcesCollectionItemVaultAggregatedVaultItem {
            vault_address,
            amount,
            last_updated_at_state_version,
        }
    }
}
