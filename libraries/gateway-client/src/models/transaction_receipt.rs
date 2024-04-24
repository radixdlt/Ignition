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
pub struct TransactionReceipt {
    #[serde(rename = "status", skip_serializing_if = "Option::is_none")]
    pub status: Option<crate::models::TransactionStatus>,

    #[serde(rename = "fee_summary", skip_serializing_if = "Option::is_none")]
    pub fee_summary: Option<serde_json::Value>,
    #[serde(
        rename = "costing_parameters",
        skip_serializing_if = "Option::is_none"
    )]
    pub costing_parameters: Option<serde_json::Value>,

    #[serde(
        rename = "fee_destination",
        skip_serializing_if = "Option::is_none"
    )]
    pub fee_destination: Option<serde_json::Value>,

    #[serde(rename = "fee_source", skip_serializing_if = "Option::is_none")]
    pub fee_source: Option<serde_json::Value>,
    #[serde(rename = "state_updates", skip_serializing_if = "Option::is_none")]
    pub state_updates: Option<StateUpdates>,
    #[serde(rename = "next_epoch", skip_serializing_if = "Option::is_none")]
    pub next_epoch: Option<serde_json::Value>,
    #[serde(rename = "output", skip_serializing_if = "Option::is_none")]
    pub output: Option<serde_json::Value>,

    #[serde(rename = "events", skip_serializing_if = "Option::is_none")]
    pub events: Option<Vec<crate::models::EventsItem>>,

    #[serde(
        rename = "error_message",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub error_message: Option<Option<String>>,
}

impl Default for TransactionReceipt {
    fn default() -> Self {
        Self::new()
    }
}

impl TransactionReceipt {
    pub fn new() -> TransactionReceipt {
        TransactionReceipt {
            status: None,
            fee_summary: None,
            costing_parameters: None,
            fee_destination: None,
            fee_source: None,
            state_updates: None,
            next_epoch: None,
            output: None,
            events: None,
            error_message: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StateUpdates {
    pub new_global_entities: Vec<Entity>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Entity {
    pub is_global: bool,
    pub entity_type: String,
    pub entity_address: String,
}
