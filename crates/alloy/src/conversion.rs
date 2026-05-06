//! Conversion helpers for the native composed Alloy adapter.

use std::borrow::Cow;

use alloy_dyn_abi::{
    DynSolType,
    eip712::{PropertyDef, Resolver, TypeDef, TypedData},
};
use alloy_primitives::{Address as AlloyAddress, Signature, U256};
use alloy_sol_types::Eip712Domain;
use cow_sdk_core::{HexData, TypedDataDomain, TypedDataField, TypedDataPayload};

pub(crate) use cow_sdk_alloy_provider::__seam::{
    alloy_to_cow_block_info, alloy_to_cow_receipt, cow_block_tag_to_alloy, cow_request_to_alloy,
    cow_to_alloy_address, cow_to_alloy_hash, rpc_error_to_class_and_detail,
};

/// Converts legacy flat typed-data fields into Alloy's dynamic typed-data shape.
///
/// The flat shape has no primary type name, so this compatibility path uses the
/// placeholder primary type `"Message"`. Canonical `CoW` order signing must
/// use [`cow_typed_data_payload_to_alloy`] so the original primary type is
/// preserved.
pub(crate) fn cow_flat_to_alloy_typed_data(
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
pub(crate) fn cow_typed_data_payload_to_alloy(
    payload: &TypedDataPayload,
) -> Result<TypedData, String> {
    let domain = build_eip712_domain(&payload.domain)?;
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

fn build_eip712_domain(domain: &TypedDataDomain) -> Result<Eip712Domain, String> {
    let verifying_contract: AlloyAddress =
        domain.verifying_contract.as_str().parse().map_err(|_| {
            format!(
                "EIP-712 domain verifying_contract `{}` is not a valid address",
                domain.verifying_contract.as_str()
            )
        })?;

    Ok(Eip712Domain {
        name: Some(Cow::Owned(domain.name.clone())),
        version: Some(Cow::Owned(domain.version.clone())),
        chain_id: Some(U256::from(domain.chain_id)),
        verifying_contract: Some(verifying_contract),
        salt: None,
    })
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

/// Hex-encodes an Alloy signature through the shared ECDSA normalizer.
pub(crate) fn alloy_signature_to_hex(
    signature: &Signature,
) -> Result<String, cow_sdk_contracts::ContractsError> {
    let raw = format!("0x{}", hex::encode(signature.as_bytes()));
    cow_sdk_contracts::normalized_ecdsa_signature(&raw)
}

pub(crate) fn parse_u256_quantity(value: &str, field: &str) -> Result<U256, String> {
    value.strip_prefix("0x").map_or_else(
        || {
            U256::from_str_radix(value, 10)
                .map_err(|error| format!("{field} `{value}` is not a valid U256: {error}"))
        },
        |hex| {
            U256::from_str_radix(hex, 16)
                .map_err(|error| format!("{field} `{value}` is not a valid U256: {error}"))
        },
    )
}

pub(crate) fn hex_data_from_bytes(bytes: &[u8]) -> Result<HexData, crate::AlloyClientError> {
    HexData::new(format!("0x{}", hex::encode(bytes)))
        .map_err(|error| crate::AlloyClientError::Internal(format!("hex conversion: {error}")))
}

pub(crate) fn decode_0x_hex(value: &str) -> Result<Vec<u8>, String> {
    let stripped = value
        .strip_prefix("0x")
        .ok_or_else(|| "hex value must be 0x-prefixed".to_owned())?;
    hex::decode(stripped).map_err(|error| error.to_string())
}
