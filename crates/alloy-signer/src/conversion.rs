//! Conversion helpers between SDK typed-data values and Alloy signing values.

use alloy_dyn_abi::{
    DynSolType,
    eip712::{PropertyDef, Resolver, TypeDef, TypedData},
};
use alloy_primitives::Signature;
use cow_sdk_core::{TypedDataDomain, TypedDataField, TypedDataPayload};

/// Converts legacy flat typed-data fields into Alloy's dynamic typed-data shape.
///
/// The flat shape has no primary type name, so this compatibility path uses the
/// placeholder primary type `"Message"`. Canonical `CoW` order signing must
/// use [`cow_typed_data_payload_to_alloy`] so the original primary type is
/// preserved.
///
/// # Errors
///
/// Returns the error returned by [`cow_typed_data_payload_to_alloy`] when the
/// underlying typed-data payload cannot be converted into Alloy's dynamic
/// typed-data shape.
pub fn cow_flat_to_alloy_typed_data(
    domain: &TypedDataDomain,
    fields: &[TypedDataField],
    value_json: &str,
) -> Result<TypedData, String> {
    let mut types = cow_sdk_core::TypedDataTypes::new();
    types.insert("Message".to_owned(), fields.to_vec());
    let payload = TypedDataPayload::new(
        domain.clone(),
        "Message".to_owned(),
        types,
        value_json.to_owned(),
    );
    cow_typed_data_payload_to_alloy(&payload)
}

/// Converts an explicit SDK typed-data payload into Alloy's dynamic typed-data shape.
///
/// This is the canonical EIP-712 path because it preserves the payload's
/// primary type and full type map end to end.
///
/// # Errors
///
/// Returns a descriptive error string when the payload's primary type is not
/// defined in the type map, when a declared field carries an unsupported or
/// invalid EIP-712 kind, when the payload's message JSON cannot be parsed, or
/// when Alloy rejects the resulting typed-data shape during the EIP-712
/// signing-hash computation.
pub fn cow_typed_data_payload_to_alloy(payload: &TypedDataPayload) -> Result<TypedData, String> {
    let domain = payload.domain.into_alloy_domain();
    let resolver = build_resolver(&payload.types, &payload.primary_type)?;
    let message = serde_json::from_str(payload.message_json())
        .map_err(|error| format!("typed-data message JSON parse error: {error}"))?;

    let typed = TypedData {
        domain,
        resolver,
        primary_type: payload.primary_type.clone(),
        message,
    };
    typed
        .eip712_signing_hash()
        .map_err(|error| format!("alloy TypedData rejected by eip712_signing_hash: {error}"))?;
    Ok(typed)
}

fn build_resolver(
    types: &cow_sdk_core::TypedDataTypes,
    primary_type: &str,
) -> Result<Resolver, String> {
    if !types.contains_key(primary_type) {
        return Err(format!(
            "primary type `{primary_type}` is not defined in the typed-data type map"
        ));
    }

    let mut resolver = Resolver::default();
    for (type_name, fields) in types {
        if type_name == "EIP712Domain" {
            continue;
        }

        let props = fields
            .iter()
            .map(|field| property_def(type_name, field))
            .collect::<Result<Vec<_>, _>>()?;
        let type_def = TypeDef::new(type_name.clone(), props)
            .map_err(|error| format!("TypeDef::new for `{type_name}` failed: {error}"))?;
        resolver.ingest(type_def);
    }

    Ok(resolver)
}

fn property_def(type_name: &str, field: &TypedDataField) -> Result<PropertyDef, String> {
    DynSolType::parse(&field.kind).map_err(|error| {
        format!(
            "type `{}` field `{}` has unsupported EIP-712 kind `{}`: {error}",
            type_name, field.name, field.kind
        )
    })?;
    PropertyDef::new(field.kind.clone(), field.name.clone()).map_err(|error| {
        format!(
            "type `{}` field `{}` has invalid EIP-712 kind `{}`: {error}",
            type_name, field.name, field.kind
        )
    })
}

/// Hex-encodes an Alloy signature through the shared `CoW` ECDSA normalizer.
///
/// Routing through `cow_sdk_contracts::normalized_ecdsa_signature` keeps the
/// signer leaf aligned with the workspace's single recovery-byte normalization
/// authority.
///
/// # Errors
///
/// Returns the [`cow_sdk_contracts::ContractsError`] surfaced by
/// `normalized_ecdsa_signature` when the hex-encoded signature is not exactly
/// 65 bytes or carries an unsupported recovery byte.
pub fn alloy_signature_to_hex(
    signature: &Signature,
) -> Result<String, cow_sdk_contracts::ContractsError> {
    let raw = format!("0x{}", alloy_primitives::hex::encode(signature.as_bytes()));
    cow_sdk_contracts::normalized_ecdsa_signature(&raw)
}

#[cfg(test)]
mod tests {
    use cow_sdk_core::{Address, TypedDataDomain, TypedDataField, TypedDataPayload};

    use super::*;

    #[test]
    fn cow_typed_data_to_alloy_round_trip() {
        let payload = simple_payload("Greeting");
        let typed = cow_typed_data_payload_to_alloy(&payload).unwrap();

        assert_eq!(typed.primary_type, "Greeting");
        assert_eq!(
            typed.message,
            serde_json::json!({
                "message": "hello"
            })
        );
        typed.eip712_signing_hash().unwrap();
    }

    #[test]
    fn flat_conversion_uses_message_placeholder_primary_type() {
        let payload = simple_payload("Greeting");
        let typed = cow_flat_to_alloy_typed_data(
            &payload.domain,
            payload.primary_type_fields().unwrap(),
            payload.message_json(),
        )
        .unwrap();

        assert_eq!(typed.primary_type, "Message");
        assert_ne!(
            typed.eip712_signing_hash().unwrap(),
            cow_typed_data_payload_to_alloy(&payload)
                .unwrap()
                .eip712_signing_hash()
                .unwrap()
        );
    }

    #[test]
    fn alloy_signature_to_hex_format() {
        let mut bytes = [1u8; 65];
        bytes[64] = 27;
        let signature = Signature::from_raw(&bytes).unwrap();
        let encoded = alloy_signature_to_hex(&signature).unwrap();

        assert_eq!(encoded.len(), 132);
        assert!(encoded.starts_with("0x"));
        assert!(encoded.ends_with("1b"));
    }

    fn simple_payload(primary_type: &str) -> TypedDataPayload {
        let domain = TypedDataDomain::new(
            "CoW".to_owned(),
            "1".to_owned(),
            1,
            Address::new("0x1111111111111111111111111111111111111111").unwrap(),
        );
        let mut types = cow_sdk_core::TypedDataTypes::new();
        types.insert(
            primary_type.to_owned(),
            vec![TypedDataField::new(
                "message".to_owned(),
                "string".to_owned(),
            )],
        );
        TypedDataPayload::new(
            domain,
            primary_type.to_owned(),
            types,
            serde_json::json!({ "message": "hello" }).to_string(),
        )
    }
}
