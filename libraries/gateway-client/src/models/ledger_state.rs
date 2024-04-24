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
pub struct LedgerState {
    #[serde(rename = "network")]
    pub network: String,

    #[serde(rename = "state_version")]
    pub state_version: i64,

    #[serde(rename = "proposer_round_timestamp")]
    pub proposer_round_timestamp: String,

    #[serde(rename = "epoch")]
    pub epoch: i64,

    #[serde(rename = "round")]
    pub round: i64,
}

impl LedgerState {
    pub fn new(
        network: String,
        state_version: i64,
        proposer_round_timestamp: String,
        epoch: i64,
        round: i64,
    ) -> LedgerState {
        LedgerState {
            network,
            state_version,
            proposer_round_timestamp,
            epoch,
            round,
        }
    }
}
