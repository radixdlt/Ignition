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

#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum StateEntityDetailsResponseItemDetailsType {
    #[serde(rename = "FungibleResource")]
    FungibleResource,
    #[serde(rename = "NonFungibleResource")]
    NonFungibleResource,
    #[serde(rename = "FungibleVault")]
    FungibleVault,
    #[serde(rename = "NonFungibleVault")]
    NonFungibleVault,
    #[serde(rename = "Package")]
    Package,
    #[serde(rename = "Component")]
    Component,
}

impl ToString for StateEntityDetailsResponseItemDetailsType {
    fn to_string(&self) -> String {
        match self {
            Self::FungibleResource => String::from("FungibleResource"),
            Self::NonFungibleResource => String::from("NonFungibleResource"),
            Self::FungibleVault => String::from("FungibleVault"),
            Self::NonFungibleVault => String::from("NonFungibleVault"),
            Self::Package => String::from("Package"),
            Self::Component => String::from("Component"),
        }
    }
}

impl Default for StateEntityDetailsResponseItemDetailsType {
    fn default() -> StateEntityDetailsResponseItemDetailsType {
        Self::FungibleResource
    }
}
