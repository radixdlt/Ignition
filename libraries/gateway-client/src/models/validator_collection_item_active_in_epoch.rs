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
pub struct ValidatorCollectionItemActiveInEpoch {
    #[serde(rename = "stake")]
    pub stake: String,
    #[serde(rename = "stake_percentage")]
    pub stake_percentage: f64,
    #[serde(rename = "key")]
    pub key: Box<crate::models::PublicKey>,
}

impl ValidatorCollectionItemActiveInEpoch {
    pub fn new(
        stake: String,
        stake_percentage: f64,
        key: crate::models::PublicKey,
    ) -> ValidatorCollectionItemActiveInEpoch {
        ValidatorCollectionItemActiveInEpoch {
            stake,
            stake_percentage,
            key: Box::new(key),
        }
    }
}
