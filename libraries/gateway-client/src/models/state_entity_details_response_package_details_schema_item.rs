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
pub struct StateEntityDetailsResponsePackageDetailsSchemaItem {
    #[serde(rename = "schema_hash_hex")]
    pub schema_hash_hex: String,

    #[serde(rename = "schema_hex")]
    pub schema_hex: String,
}

impl StateEntityDetailsResponsePackageDetailsSchemaItem {
    pub fn new(
        schema_hash_hex: String,
        schema_hex: String,
    ) -> StateEntityDetailsResponsePackageDetailsSchemaItem {
        StateEntityDetailsResponsePackageDetailsSchemaItem {
            schema_hash_hex,
            schema_hex,
        }
    }
}
