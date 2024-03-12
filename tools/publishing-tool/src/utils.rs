use sbor::representations::SerializationParameters;
use transaction::prelude::*;

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
    let schema = schema.into_latest();

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
