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
pub enum ProgrammaticScryptoSborValueKind {
    #[serde(rename = "Bool")]
    Bool,
    #[serde(rename = "I8")]
    I8,
    #[serde(rename = "I16")]
    I16,
    #[serde(rename = "I32")]
    I32,
    #[serde(rename = "I64")]
    I64,
    #[serde(rename = "I128")]
    I128,
    #[serde(rename = "U8")]
    U8,
    #[serde(rename = "U16")]
    U16,
    #[serde(rename = "U32")]
    U32,
    #[serde(rename = "U64")]
    U64,
    #[serde(rename = "U128")]
    U128,
    #[serde(rename = "String")]
    String,
    #[serde(rename = "Enum")]
    Enum,
    #[serde(rename = "Array")]
    Array,
    #[serde(rename = "Bytes")]
    Bytes,
    #[serde(rename = "Map")]
    Map,
    #[serde(rename = "Tuple")]
    Tuple,
    #[serde(rename = "Reference")]
    Reference,
    #[serde(rename = "Own")]
    Own,
    #[serde(rename = "Decimal")]
    Decimal,
    #[serde(rename = "PreciseDecimal")]
    PreciseDecimal,
    #[serde(rename = "NonFungibleLocalId")]
    NonFungibleLocalId,
}

impl ToString for ProgrammaticScryptoSborValueKind {
    fn to_string(&self) -> String {
        match self {
            Self::Bool => String::from("Bool"),
            Self::I8 => String::from("I8"),
            Self::I16 => String::from("I16"),
            Self::I32 => String::from("I32"),
            Self::I64 => String::from("I64"),
            Self::I128 => String::from("I128"),
            Self::U8 => String::from("U8"),
            Self::U16 => String::from("U16"),
            Self::U32 => String::from("U32"),
            Self::U64 => String::from("U64"),
            Self::U128 => String::from("U128"),
            Self::String => String::from("String"),
            Self::Enum => String::from("Enum"),
            Self::Array => String::from("Array"),
            Self::Bytes => String::from("Bytes"),
            Self::Map => String::from("Map"),
            Self::Tuple => String::from("Tuple"),
            Self::Reference => String::from("Reference"),
            Self::Own => String::from("Own"),
            Self::Decimal => String::from("Decimal"),
            Self::PreciseDecimal => String::from("PreciseDecimal"),
            Self::NonFungibleLocalId => String::from("NonFungibleLocalId"),
        }
    }
}

impl Default for ProgrammaticScryptoSborValueKind {
    fn default() -> ProgrammaticScryptoSborValueKind {
        Self::Bool
    }
}
