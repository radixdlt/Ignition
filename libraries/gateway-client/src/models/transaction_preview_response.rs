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

#[serde_with::serde_as]
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TransactionPreviewResponse {
    #[serde(rename = "encoded_receipt")]
    #[serde_as(as = "serde_with::hex::Hex")]
    pub encoded_receipt: Vec<u8>,
    #[serde(rename = "receipt")]
    pub receipt: serde_json::Value,
    #[serde(rename = "resource_changes")]
    pub resource_changes: Vec<serde_json::Value>,
    #[serde(rename = "logs")]
    pub logs: Vec<crate::models::TransactionPreviewResponseLogsInner>,
}

impl TransactionPreviewResponse {
    pub fn new(
        encoded_receipt: Vec<u8>,
        receipt: serde_json::Value,
        resource_changes: Vec<serde_json::Value>,
        logs: Vec<crate::models::TransactionPreviewResponseLogsInner>,
    ) -> TransactionPreviewResponse {
        TransactionPreviewResponse {
            encoded_receipt,
            receipt,
            resource_changes,
            logs,
        }
    }
}
