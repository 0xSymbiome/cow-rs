use cow_sdk_contracts::{CANCELLATIONS_TYPE_FIELDS, ContractId, ORDER_TYPE_FIELDS, Registry};
use cow_sdk_core::{
    CowEnv, OrderData, ProtocolOptions, SupportedChainId, TypedDataDomain, TypedDataEnvelope,
    TypedDataField, TypedDataPayload, TypedDataTypes,
};
use serde::Serialize;

use crate::SigningError;

/// Primary type name for `CoW` order typed-data payloads.
pub const ORDER_PRIMARY_TYPE: &str = "Order";
/// Typed-data envelope alias for explicit `CoW` order signing.
pub type OrderTypedData = TypedDataEnvelope<OrderData>;

/// Builds the `CoW` typed-data domain for a chain and optional protocol overrides.
///
/// # Errors
///
/// Returns [`SigningError`] if any override address is invalid through lower-level contracts.
///
/// # Panics
///
/// Panics if the embedded deployment registry is missing the canonical
/// settlement-contract entry for the resolved chain and environment. The
/// shipped registry manifest is validated at compile time, so this panic
/// cannot be reached from an unmodified binary.
pub fn domain(
    chain_id: SupportedChainId,
    options: Option<&ProtocolOptions>,
) -> Result<TypedDataDomain, SigningError> {
    let env = options
        .and_then(|options| options.env)
        .unwrap_or(CowEnv::Prod);
    let override_address = options
        .and_then(|options| options.settlement_contract_override.as_ref())
        .and_then(|addresses| addresses.get(&u64::from(chain_id)).copied());

    Ok(TypedDataDomain::new(
        "Gnosis Protocol".to_owned(),
        "v2".to_owned(),
        chain_id.into(),
        override_address.unwrap_or_else(|| {
            // SAFETY: Registry::default parses the build-validated embedded
            // manifest, which must include settlement addresses for supported
            // chain/environment pairs.
            Registry::default()
                .address(ContractId::Settlement, chain_id, env)
                .expect("canonical settlement address is registered for every supported chain/env")
        }),
    ))
}

/// Computes the domain separator for a chain and optional protocol overrides.
///
/// # Errors
///
/// Returns [`SigningError`] if domain construction or address encoding fails.
pub fn domain_separator(
    chain_id: SupportedChainId,
    options: Option<&ProtocolOptions>,
) -> Result<String, SigningError> {
    let domain = domain(chain_id, options)?;
    domain_separator_for(&domain)
}

/// Computes the domain separator for an explicit typed-data domain.
///
/// Returns the `0x`-prefixed lowercase 32-byte separator
/// `keccak256(type_hash || encoded_data)` for the canonical
/// `EIP712Domain(string name,string version,uint256 chainId,address
/// verifyingContract)` type string, where the encoded data packs
/// `(name_hash, version_hash, chain_id_word, verifying_contract_word)`
/// per EIP-712. The
/// `crates/signing/tests/fixtures/domain_separator_parity.json` row
/// locks the per-chain byte contract.
///
/// # Errors
///
/// Returns [`SigningError`] if the verifying-contract address cannot be parsed.
pub fn domain_separator_for(domain: &TypedDataDomain) -> Result<String, SigningError> {
    Ok(format!(
        "0x{}",
        alloy_primitives::hex::encode(domain.into_alloy_domain().separator().as_slice())
    ))
}

/// Builds the typed-data envelope with the fully typed order message body.
///
/// # Errors
///
/// Returns [`SigningError`] if domain construction or message serialization fails.
pub fn order_typed_data(
    chain_id: SupportedChainId,
    order: &OrderData,
    options: Option<&ProtocolOptions>,
) -> Result<OrderTypedData, SigningError> {
    Ok(order_typed_data_payload(chain_id, order, options)?.with_message(order.clone()))
}

/// Builds the signer-facing typed-data payload with a JSON message body.
///
/// # Errors
///
/// Returns [`SigningError`] if domain construction or message serialization fails.
pub fn order_typed_data_payload(
    chain_id: SupportedChainId,
    order: &OrderData,
    options: Option<&ProtocolOptions>,
) -> Result<TypedDataPayload, SigningError> {
    Ok(TypedDataPayload::new(
        domain(chain_id, options)?,
        ORDER_PRIMARY_TYPE.to_owned(),
        typed_data_types(ORDER_PRIMARY_TYPE, order_fields()),
        serialize_message(order)?,
    ))
}

/// Returns `CoW` order fields as core typed-data field descriptors.
#[must_use]
pub fn order_fields() -> Vec<TypedDataField> {
    ORDER_TYPE_FIELDS
        .iter()
        .map(|field| TypedDataField::new(field.name.to_owned(), field.kind.to_owned()))
        .collect()
}

/// Returns order-cancellation fields as core typed-data field descriptors.
#[must_use]
pub fn cancellation_fields() -> Vec<TypedDataField> {
    CANCELLATIONS_TYPE_FIELDS
        .iter()
        .map(|field| TypedDataField::new(field.name.to_owned(), field.kind.to_owned()))
        .collect()
}

/// Returns the canonical `EIP712Domain` field list.
#[must_use]
pub fn domain_fields() -> Vec<TypedDataField> {
    [
        ("name", "string"),
        ("version", "string"),
        ("chainId", "uint256"),
        ("verifyingContract", "address"),
    ]
    .into_iter()
    .map(|(name, kind)| TypedDataField::new(name.to_owned(), kind.to_owned()))
    .collect()
}

pub(crate) fn typed_data_types(primary_type: &str, fields: Vec<TypedDataField>) -> TypedDataTypes {
    let mut types = TypedDataTypes::new();
    types.insert(primary_type.to_owned(), fields);
    types.insert("EIP712Domain".to_owned(), domain_fields());
    types
}

pub(crate) fn serialize_message<T: Serialize>(value: &T) -> Result<String, SigningError> {
    serde_json::to_string(value)
        .map_err(|error| SigningError::Serialization(error.to_string().into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cow_sdk_core::Address;

    #[test]
    fn domain_separator_matches_shared_parity_fixture() {
        let (domain, expected_separator) = domain_separator_parity_fixture();
        let actual_separator = domain_separator_for(&domain).unwrap();

        assert_eq!(actual_separator, expected_separator);
    }

    fn domain_separator_parity_fixture() -> (TypedDataDomain, String) {
        const FIXTURE: &str = include_str!("../tests/fixtures/domain_separator_parity.json");

        let fixture: serde_json::Value =
            serde_json::from_str(FIXTURE).expect("domain separator fixture must parse");
        assert_eq!(fixture["schema_version"].as_u64(), Some(1));

        let case = &fixture["case"];
        let name = case["name"]
            .as_str()
            .expect("fixture case must carry name")
            .to_owned();
        let version = case["version"]
            .as_str()
            .expect("fixture case must carry version")
            .to_owned();
        let chain_id = case["chain_id"]
            .as_u64()
            .expect("fixture case must carry chain_id");
        let verifying_contract = Address::new(
            case["verifying_contract"]
                .as_str()
                .expect("fixture case must carry verifying_contract"),
        )
        .expect("fixture verifying_contract must be a valid address");
        let expected_separator = case["domain_separator"]
            .as_str()
            .expect("fixture case must carry domain_separator")
            .to_owned();

        (
            TypedDataDomain::new(name, version, chain_id, verifying_contract),
            expected_separator,
        )
    }
}
