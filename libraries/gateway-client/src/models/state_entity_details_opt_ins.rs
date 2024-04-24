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
pub struct StateEntityDetailsOptIns {
    #[serde(
        rename = "ancestor_identities",
        skip_serializing_if = "Option::is_none"
    )]
    pub ancestor_identities: Option<bool>,

    #[serde(
        rename = "component_royalty_vault_balance",
        skip_serializing_if = "Option::is_none"
    )]
    pub component_royalty_vault_balance: Option<bool>,

    #[serde(
        rename = "package_royalty_vault_balance",
        skip_serializing_if = "Option::is_none"
    )]
    pub package_royalty_vault_balance: Option<bool>,

    #[serde(
        rename = "non_fungible_include_nfids",
        skip_serializing_if = "Option::is_none"
    )]
    pub non_fungible_include_nfids: Option<bool>,

    #[serde(
        rename = "explicit_metadata",
        skip_serializing_if = "Option::is_none"
    )]
    pub explicit_metadata: Option<Vec<String>>,
}

impl Default for StateEntityDetailsOptIns {
    fn default() -> Self {
        Self::new()
    }
}

impl StateEntityDetailsOptIns {
    pub fn new() -> StateEntityDetailsOptIns {
        StateEntityDetailsOptIns {
            ancestor_identities: None,
            component_royalty_vault_balance: None,
            package_royalty_vault_balance: None,
            non_fungible_include_nfids: None,
            explicit_metadata: None,
        }
    }
}
