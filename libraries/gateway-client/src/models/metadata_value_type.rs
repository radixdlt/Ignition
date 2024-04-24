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
pub enum MetadataValueType {
    #[serde(rename = "String")]
    String,
    #[serde(rename = "Bool")]
    Bool,
    #[serde(rename = "U8")]
    U8,
    #[serde(rename = "U32")]
    U32,
    #[serde(rename = "U64")]
    U64,
    #[serde(rename = "I32")]
    I32,
    #[serde(rename = "I64")]
    I64,
    #[serde(rename = "Decimal")]
    Decimal,
    #[serde(rename = "GlobalAddress")]
    GlobalAddress,
    #[serde(rename = "PublicKey")]
    PublicKey,
    #[serde(rename = "NonFungibleGlobalId")]
    NonFungibleGlobalId,
    #[serde(rename = "NonFungibleLocalId")]
    NonFungibleLocalId,
    #[serde(rename = "Instant")]
    Instant,
    #[serde(rename = "Url")]
    Url,
    #[serde(rename = "Origin")]
    Origin,
    #[serde(rename = "PublicKeyHash")]
    PublicKeyHash,
    #[serde(rename = "StringArray")]
    StringArray,
    #[serde(rename = "BoolArray")]
    BoolArray,
    #[serde(rename = "U8Array")]
    U8Array,
    #[serde(rename = "U32Array")]
    U32Array,
    #[serde(rename = "U64Array")]
    U64Array,
    #[serde(rename = "I32Array")]
    I32Array,
    #[serde(rename = "I64Array")]
    I64Array,
    #[serde(rename = "DecimalArray")]
    DecimalArray,
    #[serde(rename = "GlobalAddressArray")]
    GlobalAddressArray,
    #[serde(rename = "PublicKeyArray")]
    PublicKeyArray,
    #[serde(rename = "NonFungibleGlobalIdArray")]
    NonFungibleGlobalIdArray,
    #[serde(rename = "NonFungibleLocalIdArray")]
    NonFungibleLocalIdArray,
    #[serde(rename = "InstantArray")]
    InstantArray,
    #[serde(rename = "UrlArray")]
    UrlArray,
    #[serde(rename = "OriginArray")]
    OriginArray,
    #[serde(rename = "PublicKeyHashArray")]
    PublicKeyHashArray,
}

impl ToString for MetadataValueType {
    fn to_string(&self) -> String {
        match self {
            Self::String => String::from("String"),
            Self::Bool => String::from("Bool"),
            Self::U8 => String::from("U8"),
            Self::U32 => String::from("U32"),
            Self::U64 => String::from("U64"),
            Self::I32 => String::from("I32"),
            Self::I64 => String::from("I64"),
            Self::Decimal => String::from("Decimal"),
            Self::GlobalAddress => String::from("GlobalAddress"),
            Self::PublicKey => String::from("PublicKey"),
            Self::NonFungibleGlobalId => String::from("NonFungibleGlobalId"),
            Self::NonFungibleLocalId => String::from("NonFungibleLocalId"),
            Self::Instant => String::from("Instant"),
            Self::Url => String::from("Url"),
            Self::Origin => String::from("Origin"),
            Self::PublicKeyHash => String::from("PublicKeyHash"),
            Self::StringArray => String::from("StringArray"),
            Self::BoolArray => String::from("BoolArray"),
            Self::U8Array => String::from("U8Array"),
            Self::U32Array => String::from("U32Array"),
            Self::U64Array => String::from("U64Array"),
            Self::I32Array => String::from("I32Array"),
            Self::I64Array => String::from("I64Array"),
            Self::DecimalArray => String::from("DecimalArray"),
            Self::GlobalAddressArray => String::from("GlobalAddressArray"),
            Self::PublicKeyArray => String::from("PublicKeyArray"),
            Self::NonFungibleGlobalIdArray => {
                String::from("NonFungibleGlobalIdArray")
            }
            Self::NonFungibleLocalIdArray => {
                String::from("NonFungibleLocalIdArray")
            }
            Self::InstantArray => String::from("InstantArray"),
            Self::UrlArray => String::from("UrlArray"),
            Self::OriginArray => String::from("OriginArray"),
            Self::PublicKeyHashArray => String::from("PublicKeyHashArray"),
        }
    }
}

impl Default for MetadataValueType {
    fn default() -> MetadataValueType {
        Self::String
    }
}
