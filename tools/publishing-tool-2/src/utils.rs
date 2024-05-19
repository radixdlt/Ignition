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

use radix_common::prelude::*;
use radix_transactions::prelude::*;
use sbor::representations::SerializationParameters;

pub fn clone_private_key(private_key: &PrivateKey) -> PrivateKey {
    match private_key {
        PrivateKey::Secp256k1(private_key) => PrivateKey::Secp256k1(
            Secp256k1PrivateKey::from_bytes(&private_key.to_bytes()).unwrap(),
        ),
        PrivateKey::Ed25519(private_key) => PrivateKey::Ed25519(
            Ed25519PrivateKey::from_bytes(&private_key.to_bytes()).unwrap(),
        ),
    }
}

pub fn to_json<S: ScryptoEncode + ScryptoDescribe>(
    value: &S,
    network_definition: &NetworkDefinition,
) -> String {
    let encoder = AddressBech32Encoder::new(network_definition);

    let (local_type_id, schema) =
        generate_full_schema_from_single_type::<S, ScryptoCustomSchema>();
    let schema = schema.fully_update_and_into_latest_version();

    let context =
        ScryptoValueDisplayContext::with_optional_bech32(Some(&encoder));
    let payload = scrypto_encode(value).unwrap();
    let raw_payload = ScryptoRawPayload::new_from_valid_slice(&payload);
    let serializable =
        raw_payload.serializable(SerializationParameters::WithSchema {
            mode: representations::SerializationMode::Natural,
            custom_context: context,
            schema: &schema,
            type_id: local_type_id,
            depth_limit: SCRYPTO_SBOR_V1_MAX_DEPTH,
        });

    serde_json::to_string_pretty(&serializable).unwrap()
}
