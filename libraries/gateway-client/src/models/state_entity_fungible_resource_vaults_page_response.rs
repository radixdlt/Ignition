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
pub struct StateEntityFungibleResourceVaultsPageResponse {
    #[serde(rename = "ledger_state")]
    pub ledger_state: Box<crate::models::LedgerState>,

    #[serde(
        rename = "total_count",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub total_count: Option<Option<i64>>,

    #[serde(
        rename = "next_cursor",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub next_cursor: Option<Option<String>>,
    #[serde(rename = "items")]
    pub items: Vec<
        crate::models::FungibleResourcesCollectionItemVaultAggregatedVaultItem,
    >,

    #[serde(rename = "address")]
    pub address: String,

    #[serde(rename = "resource_address")]
    pub resource_address: String,
}

impl StateEntityFungibleResourceVaultsPageResponse {
    pub fn new(
        ledger_state: crate::models::LedgerState,
        items: Vec<
            crate::models::FungibleResourcesCollectionItemVaultAggregatedVaultItem,
        >,
        address: String,
        resource_address: String,
    ) -> StateEntityFungibleResourceVaultsPageResponse {
        StateEntityFungibleResourceVaultsPageResponse {
            ledger_state: Box::new(ledger_state),
            total_count: None,
            next_cursor: None,
            items,
            address,
            resource_address,
        }
    }
}
