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
pub struct StreamTransactionsRequestEventFilterItem {
    #[serde(rename = "event")]
    pub event: Event,

    #[serde(
        rename = "emitter_address",
        skip_serializing_if = "Option::is_none"
    )]
    pub emitter_address: Option<String>,

    #[serde(
        rename = "resource_address",
        skip_serializing_if = "Option::is_none"
    )]
    pub resource_address: Option<String>,
}

impl StreamTransactionsRequestEventFilterItem {
    pub fn new(event: Event) -> StreamTransactionsRequestEventFilterItem {
        StreamTransactionsRequestEventFilterItem {
            event,
            emitter_address: None,
            resource_address: None,
        }
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum Event {
    #[serde(rename = "Deposit")]
    Deposit,
    #[serde(rename = "Withdrawal")]
    Withdrawal,
}

impl Default for Event {
    fn default() -> Event {
        Self::Deposit
    }
}
