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
pub struct TransactionStatusResponseKnownPayloadItem {
    #[serde(rename = "payload_hash")]
    pub payload_hash: String,
    #[serde(rename = "status")]
    pub status: crate::models::TransactionStatus,
    #[serde(
        rename = "payload_status",
        skip_serializing_if = "Option::is_none"
    )]
    pub payload_status: Option<crate::models::TransactionPayloadStatus>,

    #[serde(
        rename = "payload_status_description",
        skip_serializing_if = "Option::is_none"
    )]
    pub payload_status_description: Option<String>,

    #[serde(
        rename = "error_message",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub error_message: Option<Option<String>>,

    #[serde(
        rename = "latest_error_message",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub latest_error_message: Option<Option<String>>,
    #[serde(
        rename = "handling_status",
        skip_serializing_if = "Option::is_none"
    )]
    pub handling_status:
        Option<crate::models::TransactionPayloadGatewayHandlingStatus>,

    #[serde(
        rename = "handling_status_reason",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub handling_status_reason: Option<Option<String>>,

    #[serde(
        rename = "submission_error",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub submission_error: Option<Option<String>>,
}

impl TransactionStatusResponseKnownPayloadItem {
    pub fn new(
        payload_hash: String,
        status: crate::models::TransactionStatus,
    ) -> TransactionStatusResponseKnownPayloadItem {
        TransactionStatusResponseKnownPayloadItem {
            payload_hash,
            status,
            payload_status: None,
            payload_status_description: None,
            error_message: None,
            latest_error_message: None,
            handling_status: None,
            handling_status_reason: None,
            submission_error: None,
        }
    }
}
