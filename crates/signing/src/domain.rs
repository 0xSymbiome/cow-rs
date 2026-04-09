use std::collections::BTreeMap;

use cow_sdk_contracts::{CANCELLATIONS_TYPE_FIELDS, ORDER_TYPE_FIELDS};
use cow_sdk_core::{
    Address, CowEnv, ProtocolOptions, SupportedChainId, TypedDataDomain, TypedDataField,
    UnsignedOrder, settlement_contract_address,
};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};

use crate::SigningError;

pub const ORDER_PRIMARY_TYPE: &str = "Order";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderTypedData {
    pub domain: TypedDataDomain,
    pub primary_type: String,
    pub types: BTreeMap<String, Vec<TypedDataField>>,
    pub message: UnsignedOrder,
}

pub fn get_domain(
    chain_id: SupportedChainId,
    options: Option<&ProtocolOptions>,
) -> Result<TypedDataDomain, SigningError> {
    let env = options
        .and_then(|options| options.env)
        .unwrap_or(CowEnv::Prod);
    let override_address = options
        .and_then(|options| options.settlement_contract_override.as_ref())
        .and_then(|addresses| addresses.get(&u64::from(chain_id)).cloned());

    Ok(TypedDataDomain {
        name: "Gnosis Protocol".to_owned(),
        version: "v2".to_owned(),
        chain_id: chain_id.into(),
        verifying_contract: override_address
            .unwrap_or_else(|| settlement_contract_address(chain_id, env)),
    })
}

pub fn domain_separator(
    chain_id: SupportedChainId,
    options: Option<&ProtocolOptions>,
) -> Result<String, SigningError> {
    let domain = get_domain(chain_id, options)?;
    domain_separator_for(&domain)
}

pub fn domain_separator_for(domain: &TypedDataDomain) -> Result<String, SigningError> {
    let mut encoded = Vec::with_capacity(32 * 5);
    encoded.extend_from_slice(&keccak256(
        "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
            .as_bytes(),
    ));
    encoded.extend_from_slice(&keccak256(domain.name.as_bytes()));
    encoded.extend_from_slice(&keccak256(domain.version.as_bytes()));
    encoded.extend_from_slice(&encode_u256_u64(domain.chain_id));
    encoded.extend_from_slice(&encode_address(&domain.verifying_contract)?);
    Ok(format!("0x{}", hex::encode(keccak256(encoded))))
}

pub fn order_typed_data(
    chain_id: SupportedChainId,
    order: &UnsignedOrder,
    options: Option<&ProtocolOptions>,
) -> Result<OrderTypedData, SigningError> {
    let mut types = BTreeMap::new();
    types.insert(ORDER_PRIMARY_TYPE.to_owned(), order_fields());
    types.insert("EIP712Domain".to_owned(), domain_fields());

    Ok(OrderTypedData {
        domain: get_domain(chain_id, options)?,
        primary_type: ORDER_PRIMARY_TYPE.to_owned(),
        types,
        message: order.clone(),
    })
}

pub fn order_fields() -> Vec<TypedDataField> {
    ORDER_TYPE_FIELDS
        .iter()
        .map(|field| TypedDataField {
            name: field.name.to_owned(),
            kind: field.kind.to_owned(),
        })
        .collect()
}

pub fn cancellation_fields() -> Vec<TypedDataField> {
    CANCELLATIONS_TYPE_FIELDS
        .iter()
        .map(|field| TypedDataField {
            name: field.name.to_owned(),
            kind: field.kind.to_owned(),
        })
        .collect()
}

pub fn domain_fields() -> Vec<TypedDataField> {
    [
        ("name", "string"),
        ("version", "string"),
        ("chainId", "uint256"),
        ("verifyingContract", "address"),
    ]
    .into_iter()
    .map(|(name, kind)| TypedDataField {
        name: name.to_owned(),
        kind: kind.to_owned(),
    })
    .collect()
}

fn keccak256(bytes: impl AsRef<[u8]>) -> [u8; 32] {
    let digest = Keccak256::digest(bytes.as_ref());
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

fn encode_u256_u64(value: u64) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[24..].copy_from_slice(&value.to_be_bytes());
    out
}

fn encode_address(address: &Address) -> Result<[u8; 32], SigningError> {
    let Some(stripped) = address.as_str().strip_prefix("0x") else {
        return Err(SigningError::Serialization(
            "address must be 0x-prefixed".to_owned(),
        ));
    };
    let bytes = hex::decode(stripped).map_err(|_| {
        SigningError::Serialization("address contains non-hex characters".to_owned())
    })?;
    if bytes.len() != 20 {
        return Err(SigningError::Serialization(
            "address must be 20 bytes".to_owned(),
        ));
    }

    let mut out = [0u8; 32];
    out[12..].copy_from_slice(&bytes);
    Ok(out)
}
