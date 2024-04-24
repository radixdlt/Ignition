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
pub struct TransactionDetailsOptIns {
    #[serde(rename = "raw_hex", skip_serializing_if = "Option::is_none")]
    pub raw_hex: Option<bool>,

    #[serde(
        rename = "receipt_state_changes",
        skip_serializing_if = "Option::is_none"
    )]
    pub receipt_state_changes: Option<bool>,

    #[serde(
        rename = "receipt_fee_summary",
        skip_serializing_if = "Option::is_none"
    )]
    pub receipt_fee_summary: Option<bool>,

    #[serde(
        rename = "receipt_fee_source",
        skip_serializing_if = "Option::is_none"
    )]
    pub receipt_fee_source: Option<bool>,

    #[serde(
        rename = "receipt_fee_destination",
        skip_serializing_if = "Option::is_none"
    )]
    pub receipt_fee_destination: Option<bool>,

    #[serde(
        rename = "receipt_costing_parameters",
        skip_serializing_if = "Option::is_none"
    )]
    pub receipt_costing_parameters: Option<bool>,

    #[serde(
        rename = "receipt_events",
        skip_serializing_if = "Option::is_none"
    )]
    pub receipt_events: Option<bool>,

    #[serde(
        rename = "receipt_output",
        skip_serializing_if = "Option::is_none"
    )]
    pub receipt_output: Option<bool>,

    #[serde(
        rename = "affected_global_entities",
        skip_serializing_if = "Option::is_none"
    )]
    pub affected_global_entities: Option<bool>,

    #[serde(
        rename = "balance_changes",
        skip_serializing_if = "Option::is_none"
    )]
    pub balance_changes: Option<bool>,
}

impl Default for TransactionDetailsOptIns {
    fn default() -> Self {
        Self::new()
    }
}

impl TransactionDetailsOptIns {
    pub fn new() -> TransactionDetailsOptIns {
        TransactionDetailsOptIns {
            raw_hex: None,
            receipt_state_changes: None,
            receipt_fee_summary: None,
            receipt_fee_source: None,
            receipt_fee_destination: None,
            receipt_costing_parameters: None,
            receipt_events: None,
            receipt_output: None,
            affected_global_entities: None,
            balance_changes: None,
        }
    }
}
