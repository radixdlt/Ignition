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
pub struct TransactionPreviewRequest {
    #[serde(rename = "manifest")]
    pub manifest: String,

    #[serde(rename = "blobs_hex", skip_serializing_if = "Option::is_none")]
    pub blobs_hex: Option<Vec<String>>,

    #[serde(rename = "start_epoch_inclusive")]
    pub start_epoch_inclusive: i64,

    #[serde(rename = "end_epoch_exclusive")]
    pub end_epoch_exclusive: i64,
    #[serde(
        rename = "notary_public_key",
        skip_serializing_if = "Option::is_none"
    )]
    pub notary_public_key: Option<Box<crate::models::PublicKey>>,

    #[serde(
        rename = "notary_is_signatory",
        skip_serializing_if = "Option::is_none"
    )]
    pub notary_is_signatory: Option<bool>,

    #[serde(rename = "tip_percentage")]
    pub tip_percentage: i32,

    #[serde(rename = "nonce")]
    pub nonce: i64,

    #[serde(rename = "signer_public_keys")]
    pub signer_public_keys: Vec<crate::models::PublicKey>,
    #[serde(rename = "flags")]
    pub flags: Box<crate::models::TransactionPreviewRequestFlags>,
}

impl TransactionPreviewRequest {
    pub fn new(
        manifest: String,
        start_epoch_inclusive: i64,
        end_epoch_exclusive: i64,
        tip_percentage: i32,
        nonce: i64,
        signer_public_keys: Vec<crate::models::PublicKey>,
        flags: crate::models::TransactionPreviewRequestFlags,
    ) -> TransactionPreviewRequest {
        TransactionPreviewRequest {
            manifest,
            blobs_hex: None,
            start_epoch_inclusive,
            end_epoch_exclusive,
            notary_public_key: None,
            notary_is_signatory: None,
            tip_percentage,
            nonce,
            signer_public_keys,
            flags: Box::new(flags),
        }
    }
}
