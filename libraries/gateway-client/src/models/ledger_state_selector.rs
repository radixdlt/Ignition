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
pub struct LedgerStateSelector {
    #[serde(
        rename = "state_version",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub state_version: Option<Option<i64>>,

    #[serde(
        rename = "timestamp",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub timestamp: Option<Option<String>>,

    #[serde(
        rename = "epoch",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub epoch: Option<Option<i64>>,

    #[serde(
        rename = "round",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub round: Option<Option<i64>>,
}

impl Default for LedgerStateSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl LedgerStateSelector {
    pub fn new() -> LedgerStateSelector {
        LedgerStateSelector {
            state_version: None,
            timestamp: None,
            epoch: None,
            round: None,
        }
    }
}
