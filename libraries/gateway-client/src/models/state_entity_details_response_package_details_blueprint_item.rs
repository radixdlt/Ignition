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
pub struct StateEntityDetailsResponsePackageDetailsBlueprintItem {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "version")]
    pub version: String,

    #[serde(rename = "definition")]
    pub definition: serde_json::Value,
    #[serde(
        rename = "dependant_entities",
        skip_serializing_if = "Option::is_none"
    )]
    pub dependant_entities: Option<Vec<String>>,

    #[serde(rename = "auth_template", skip_serializing_if = "Option::is_none")]
    pub auth_template: Option<serde_json::Value>,
    #[serde(
        rename = "auth_template_is_locked",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub auth_template_is_locked: Option<Option<bool>>,

    #[serde(
        rename = "royalty_config",
        skip_serializing_if = "Option::is_none"
    )]
    pub royalty_config: Option<serde_json::Value>,
    #[serde(
        rename = "royalty_config_is_locked",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub royalty_config_is_locked: Option<Option<bool>>,
}

impl StateEntityDetailsResponsePackageDetailsBlueprintItem {
    pub fn new(
        name: String,
        version: String,
        definition: serde_json::Value,
    ) -> StateEntityDetailsResponsePackageDetailsBlueprintItem {
        StateEntityDetailsResponsePackageDetailsBlueprintItem {
            name,
            version,
            definition,
            dependant_entities: None,
            auth_template: None,
            auth_template_is_locked: None,
            royalty_config: None,
            royalty_config_is_locked: None,
        }
    }
}
