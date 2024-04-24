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
pub struct TransactionStatusResponse {
    #[serde(rename = "ledger_state")]
    pub ledger_state: Box<crate::models::LedgerState>,
    #[serde(rename = "status")]
    pub status: crate::models::TransactionStatus,
    #[serde(rename = "intent_status")]
    pub intent_status: crate::models::TransactionIntentStatus,

    #[serde(rename = "intent_status_description")]
    pub intent_status_description: String,
    #[serde(rename = "known_payloads")]
    pub known_payloads:
        Vec<crate::models::TransactionStatusResponseKnownPayloadItem>,

    #[serde(
        rename = "committed_state_version",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub committed_state_version: Option<Option<i64>>,

    #[serde(
        rename = "error_message",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub error_message: Option<Option<String>>,
}

impl TransactionStatusResponse {
    pub fn new(
        ledger_state: crate::models::LedgerState,
        status: crate::models::TransactionStatus,
        intent_status: crate::models::TransactionIntentStatus,
        intent_status_description: String,
        known_payloads: Vec<
            crate::models::TransactionStatusResponseKnownPayloadItem,
        >,
    ) -> TransactionStatusResponse {
        TransactionStatusResponse {
            ledger_state: Box::new(ledger_state),
            status,
            intent_status,
            intent_status_description,
            known_payloads,
            committed_state_version: None,
            error_message: None,
        }
    }
}
