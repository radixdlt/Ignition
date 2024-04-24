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
pub enum NonFungibleIdType {
    #[serde(rename = "String")]
    String,
    #[serde(rename = "Integer")]
    Integer,
    #[serde(rename = "Bytes")]
    Bytes,
    #[serde(rename = "Ruid")]
    Ruid,
}

impl ToString for NonFungibleIdType {
    fn to_string(&self) -> String {
        match self {
            Self::String => String::from("String"),
            Self::Integer => String::from("Integer"),
            Self::Bytes => String::from("Bytes"),
            Self::Ruid => String::from("Ruid"),
        }
    }
}

impl Default for NonFungibleIdType {
    fn default() -> NonFungibleIdType {
        Self::String
    }
}
