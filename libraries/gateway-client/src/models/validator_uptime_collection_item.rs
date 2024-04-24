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
pub struct ValidatorUptimeCollectionItem {
    #[serde(rename = "address")]
    pub address: String,

    #[serde(
        rename = "proposals_made",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub proposals_made: Option<Option<i64>>,

    #[serde(
        rename = "proposals_missed",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub proposals_missed: Option<Option<i64>>,

    #[serde(rename = "epochs_active_in")]
    pub epochs_active_in: i64,
}

impl ValidatorUptimeCollectionItem {
    pub fn new(
        address: String,
        epochs_active_in: i64,
    ) -> ValidatorUptimeCollectionItem {
        ValidatorUptimeCollectionItem {
            address,
            proposals_made: None,
            proposals_missed: None,
            epochs_active_in,
        }
    }
}
