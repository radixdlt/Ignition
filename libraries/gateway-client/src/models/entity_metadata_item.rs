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
pub struct EntityMetadataItem {
    #[serde(rename = "key")]
    pub key: String,
    #[serde(rename = "value")]
    pub value: Box<crate::models::EntityMetadataItemValue>,
    #[serde(rename = "is_locked")]
    pub is_locked: bool,

    #[serde(rename = "last_updated_at_state_version")]
    pub last_updated_at_state_version: i64,
}

impl EntityMetadataItem {
    pub fn new(
        key: String,
        value: crate::models::EntityMetadataItemValue,
        is_locked: bool,
        last_updated_at_state_version: i64,
    ) -> EntityMetadataItem {
        EntityMetadataItem {
            key,
            value: Box::new(value),
            is_locked,
            last_updated_at_state_version,
        }
    }
}
