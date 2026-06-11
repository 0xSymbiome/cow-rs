//! Conversion helpers between SDK typed-data values and Alloy signing values.

use alloy_dyn_abi::{
    DynSolType,
    eip712::{PropertyDef, Resolver, TypeDef, TypedData},
};
use alloy_primitives::Signature;
use cow_sdk_core::{TypedDataField, TypedDataPayload};

/// Converts an explicit SDK typed-data payload into Alloy's dynamic typed-data shape.
///
/// This is the canonical EIP-712 path because it preserves the payload's
/// primary type and full type map end to end. A field may reference another
/// struct declared in the type map, directly or as an array (for example
/// `Call[]`), so nested multi-type EIP-712 payloads convert end to end.
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
            .map(|field| property_def(type_name, field, types))
            .collect::<Result<Vec<_>, _>>()?;
        let type_def = TypeDef::new(type_name.clone(), props)
            .map_err(|error| format!("TypeDef::new for `{type_name}` failed: {error}"))?;
        resolver.ingest(type_def);
    }

    Ok(resolver)
}

fn property_def(
    type_name: &str,
    field: &TypedDataField,
    declared_types: &cow_sdk_core::TypedDataTypes,
) -> Result<PropertyDef, String> {
    // A field may reference another EIP-712 struct by name, optionally as an
    // array (for example `Call[]`). `DynSolType::parse` recognizes only ABI
    // primitives and rejects a bare struct name, but the resolver links a
    // referenced struct through its ingested `TypeDef`, so a reference to a
    // struct declared in the type map is valid here. Gate the kind through
    // `DynSolType::parse` only when it does not name a declared struct, which
    // keeps the fail-closed rejection for genuinely unsupported primitive kinds
    // while admitting the nested-struct payloads EIP-712 allows.
    if !references_declared_struct(&field.kind, declared_types) {
        DynSolType::parse(&field.kind).map_err(|error| {
            format!(
                "type `{}` field `{}` has unsupported EIP-712 kind `{}`: {error}",
                type_name, field.name, field.kind
            )
        })?;
    }
    PropertyDef::new(field.kind.clone(), field.name.clone()).map_err(|error| {
        format!(
            "type `{}` field `{}` has invalid EIP-712 kind `{}`: {error}",
            type_name, field.name, field.kind
        )
    })
}

/// Returns `true` when `kind` names a struct declared in the type map, either
/// directly (`Call`) or as an array of that struct (`Call[]`, `Call[2]`,
/// `Call[][3]`). The root identifier is the substring before the first `[`.
fn references_declared_struct(kind: &str, declared_types: &cow_sdk_core::TypedDataTypes) -> bool {
    let root = kind.split('[').next().unwrap_or(kind);
    declared_types.contains_key(root)
}

/// Hex-encodes an Alloy signature through the contracts-boundary
/// recoverable-signature typestate.
///
/// Routing through `cow_sdk_contracts::RecoverableSignature` keeps the
/// signer leaf aligned with the workspace's single canonical
/// recovery-byte authority and surfaces the typed
/// `ContractsError::InvalidSignatureRecoveryByte` rejection for
/// trailing bytes outside the ADR 0022 accept set.
///
/// # Errors
///
/// Returns the [`cow_sdk_contracts::ContractsError`] surfaced by
/// `RecoverableSignature::parse_bytes` when the encoded signature is
/// not exactly 65 bytes or carries an unsupported recovery byte.
pub fn alloy_signature_to_hex(
    signature: &Signature,
) -> Result<String, cow_sdk_contracts::ContractsError> {
    cow_sdk_contracts::RecoverableSignature::parse_bytes(&signature.as_bytes())
        .map(|sig| sig.to_hex_string())
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
    fn alloy_signature_to_hex_format() {
        let mut bytes = [1u8; 65];
        bytes[64] = 27;
        let signature = Signature::from_raw(&bytes).unwrap();
        let encoded = alloy_signature_to_hex(&signature).unwrap();

        assert_eq!(encoded.len(), 132);
        assert!(encoded.starts_with("0x"));
        assert!(encoded.ends_with("1b"));
    }

    #[test]
    fn nested_struct_payload_matches_macro_digest() {
        use alloy_primitives::{B256, U256, address};
        use alloy_sol_types::{SolStruct, eip712_domain, sol};

        sol! {
            struct Item {
                address token;
                uint256 amount;
            }
            struct Order {
                Item primary;
                Item[] extras;
                bytes32 nonce;
            }
        }

        let domain = eip712_domain! {
            name: "CoW Nested",
            version: "1",
            chain_id: 1,
            verifying_contract: address!("0x1111111111111111111111111111111111111111"),
        };

        // Reference digest from the macro-emitted `SolStruct` envelope (the same
        // path the workspace uses for nested protocol structs).
        let order = Order {
            primary: Item {
                token: address!("0x000000000000000000000000000000000000aaaa"),
                amount: U256::from(1u64),
            },
            extras: vec![Item {
                token: address!("0x000000000000000000000000000000000000bbbb"),
                amount: U256::from(2u64),
            }],
            nonce: B256::ZERO,
        };
        let expected = order.eip712_signing_hash(&domain);

        // The same nested payload expressed through the SDK dynamic typed-data
        // shape, where `Item` and `Item[]` are struct references resolved
        // through the type map.
        let mut types = cow_sdk_core::TypedDataTypes::new();
        types.insert(
            "Item".to_owned(),
            vec![
                TypedDataField::new("token".to_owned(), "address".to_owned()),
                TypedDataField::new("amount".to_owned(), "uint256".to_owned()),
            ],
        );
        types.insert(
            "Order".to_owned(),
            vec![
                TypedDataField::new("primary".to_owned(), "Item".to_owned()),
                TypedDataField::new("extras".to_owned(), "Item[]".to_owned()),
                TypedDataField::new("nonce".to_owned(), "bytes32".to_owned()),
            ],
        );
        let message = serde_json::json!({
            "primary": { "token": "0x000000000000000000000000000000000000aaaa", "amount": "1" },
            "extras": [ { "token": "0x000000000000000000000000000000000000bbbb", "amount": "2" } ],
            "nonce": "0x0000000000000000000000000000000000000000000000000000000000000000",
        })
        .to_string();
        let payload = TypedDataPayload::new(
            TypedDataDomain::new(
                "CoW Nested".to_owned(),
                "1".to_owned(),
                1,
                Address::new("0x1111111111111111111111111111111111111111").unwrap(),
            ),
            "Order".to_owned(),
            types,
            message,
        );

        let converted =
            cow_typed_data_payload_to_alloy(&payload).expect("nested-struct payload must convert");
        assert_eq!(
            converted.eip712_signing_hash().unwrap(),
            expected,
            "dynamic nested-struct digest must equal the macro-emitted SolStruct digest"
        );
    }

    #[test]
    fn undeclared_struct_reference_is_rejected() {
        let mut types = cow_sdk_core::TypedDataTypes::new();
        types.insert(
            "Wrapper".to_owned(),
            vec![TypedDataField::new(
                "inner".to_owned(),
                "Missing".to_owned(),
            )],
        );
        let payload = TypedDataPayload::new(
            TypedDataDomain::new(
                "CoW".to_owned(),
                "1".to_owned(),
                1,
                Address::new("0x1111111111111111111111111111111111111111").unwrap(),
            ),
            "Wrapper".to_owned(),
            types,
            serde_json::json!({ "inner": {} }).to_string(),
        );
        assert!(
            cow_typed_data_payload_to_alloy(&payload).is_err(),
            "a field referencing an undeclared struct must stay fail-closed"
        );
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
