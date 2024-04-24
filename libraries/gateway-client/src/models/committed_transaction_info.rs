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
pub struct CommittedTransactionInfo {
    #[serde(rename = "state_version")]
    pub state_version: i64,
    #[serde(rename = "epoch")]
    pub epoch: i64,
    #[serde(rename = "round")]
    pub round: i64,
    #[serde(rename = "round_timestamp")]
    pub round_timestamp: String,
    #[serde(rename = "transaction_status")]
    pub transaction_status: crate::models::TransactionStatus,

    #[serde(rename = "payload_hash", skip_serializing_if = "Option::is_none")]
    pub payload_hash: Option<String>,

    #[serde(rename = "intent_hash", skip_serializing_if = "Option::is_none")]
    pub intent_hash: Option<String>,

    #[serde(rename = "fee_paid", skip_serializing_if = "Option::is_none")]
    pub fee_paid: Option<String>,
    #[serde(
        rename = "affected_global_entities",
        skip_serializing_if = "Option::is_none"
    )]
    pub affected_global_entities: Option<Vec<String>>,
    #[serde(
        rename = "confirmed_at",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub confirmed_at: Option<Option<String>>,
    #[serde(
        rename = "error_message",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub error_message: Option<Option<String>>,

    #[serde(rename = "raw_hex", skip_serializing_if = "Option::is_none")]
    pub raw_hex: Option<String>,
    #[serde(rename = "receipt", skip_serializing_if = "Option::is_none")]
    pub receipt: Option<Box<crate::models::TransactionReceipt>>,

    #[serde(rename = "message", skip_serializing_if = "Option::is_none")]
    pub message: Option<serde_json::Value>,
    #[serde(
        rename = "balance_changes",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub balance_changes:
        Option<Option<Box<crate::models::TransactionBalanceChanges>>>,
}

impl CommittedTransactionInfo {
    pub fn new(
        state_version: i64,
        epoch: i64,
        round: i64,
        round_timestamp: String,
        transaction_status: crate::models::TransactionStatus,
    ) -> CommittedTransactionInfo {
        CommittedTransactionInfo {
            state_version,
            epoch,
            round,
            round_timestamp,
            transaction_status,
            payload_hash: None,
            intent_hash: None,
            fee_paid: None,
            affected_global_entities: None,
            confirmed_at: None,
            error_message: None,
            raw_hex: None,
            receipt: None,
            message: None,
            balance_changes: None,
        }
    }
}
