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
pub struct NotSyncedUpError {
    #[serde(rename = "type")]
    pub r#type: String,

    #[serde(rename = "request_type")]
    pub request_type: String,

    #[serde(rename = "current_sync_delay_seconds")]
    pub current_sync_delay_seconds: i64,

    #[serde(rename = "max_allowed_sync_delay_seconds")]
    pub max_allowed_sync_delay_seconds: i64,
}

impl NotSyncedUpError {
    pub fn new(
        r#type: String,
        request_type: String,
        current_sync_delay_seconds: i64,
        max_allowed_sync_delay_seconds: i64,
    ) -> NotSyncedUpError {
        NotSyncedUpError {
            r#type,
            request_type,
            current_sync_delay_seconds,
            max_allowed_sync_delay_seconds,
        }
    }
}
