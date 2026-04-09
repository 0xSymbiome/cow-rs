use num_bigint::BigUint;
use sha3::{Digest, Keccak256};

use cow_sdk_core::{Address, AppDataHash, ChainId, OrderBalance, OrderKind, TypedDataDomain};

use crate::ContractsError;

pub(crate) const ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";
pub(crate) const ORDER_UID_LENGTH_BYTES: usize = 56;

pub(crate) fn zero_address() -> Address {
    Address::new(ZERO_ADDRESS).expect("static zero address must remain valid")
}

pub(crate) fn keccak256(bytes: impl AsRef<[u8]>) -> [u8; 32] {
    let digest = Keccak256::digest(bytes.as_ref());
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

pub(crate) fn keccak256_hex(bytes: impl AsRef<[u8]>) -> String {
    format!("0x{}", hex::encode(keccak256(bytes)))
}

pub(crate) fn parse_hex(value: &str, field: &'static str) -> Result<Vec<u8>, ContractsError> {
    let Some(stripped) = value.strip_prefix("0x") else {
        return Err(ContractsError::Decode(format!(
            "{field} must be 0x-prefixed hexadecimal data"
        )));
    };
    hex::decode(stripped)
        .map_err(|_| ContractsError::Decode(format!("{field} contains non-hex characters")))
}

pub(crate) fn parse_hex_exact(
    value: &str,
    field: &'static str,
    expected: usize,
) -> Result<Vec<u8>, ContractsError> {
    let bytes = parse_hex(value, field)?;
    if bytes.len() != expected {
        return Err(ContractsError::Decode(format!(
            "{field} must be {expected} bytes, got {}",
            bytes.len()
        )));
    }
    Ok(bytes)
}

pub(crate) fn parse_address_bytes(address: &Address) -> Result<[u8; 20], ContractsError> {
    let bytes = parse_hex_exact(address.as_str(), "address", 20)?;
    let mut out = [0u8; 20];
    out.copy_from_slice(&bytes);
    Ok(out)
}

pub(crate) fn parse_bytes32_hash(hash: &AppDataHash) -> Result<[u8; 32], ContractsError> {
    let bytes = parse_hex_exact(hash.as_str(), "appData", 32)?;
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

pub(crate) fn parse_hex32(value: &str, field: &'static str) -> Result<[u8; 32], ContractsError> {
    let bytes = parse_hex_exact(value, field, 32)?;
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

pub(crate) fn encode_address(address: &Address) -> Result<[u8; 32], ContractsError> {
    let mut out = [0u8; 32];
    out[12..].copy_from_slice(&parse_address_bytes(address)?);
    Ok(out)
}

pub(crate) fn encode_u32(value: u32) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[28..].copy_from_slice(&value.to_be_bytes());
    out
}

pub(crate) fn encode_u256_str(
    field: &'static str,
    value: &str,
) -> Result<[u8; 32], ContractsError> {
    let parsed = if let Some(stripped) = value.strip_prefix("0x") {
        BigUint::parse_bytes(stripped.as_bytes(), 16)
    } else {
        BigUint::parse_bytes(value.as_bytes(), 10)
    }
    .ok_or_else(|| ContractsError::InvalidNumeric {
        field,
        value: value.to_owned(),
    })?;

    let bytes = parsed.to_bytes_be();
    if bytes.len() > 32 {
        return Err(ContractsError::NumericOverflow {
            field,
            value: value.to_owned(),
        });
    }

    let mut out = [0u8; 32];
    out[32 - bytes.len()..].copy_from_slice(&bytes);
    Ok(out)
}

pub(crate) fn encode_bool(value: bool) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[31] = u8::from(value);
    out
}

pub(crate) fn encode_string_hash(value: &str) -> [u8; 32] {
    keccak256(value.as_bytes())
}

pub(crate) fn chain_id_bytes(chain_id: ChainId) -> Result<[u8; 32], ContractsError> {
    encode_u256_str("chainId", &chain_id.to_string())
}

pub(crate) fn order_kind_name(kind: OrderKind) -> &'static str {
    match kind {
        OrderKind::Buy => "buy",
        OrderKind::Sell => "sell",
    }
}

pub(crate) fn balance_name(balance: OrderBalance) -> &'static str {
    match balance {
        OrderBalance::Erc20 => "erc20",
        OrderBalance::External => "external",
        OrderBalance::Internal => "internal",
    }
}

pub(crate) fn domain_separator(domain: &TypedDataDomain) -> Result<[u8; 32], ContractsError> {
    const DOMAIN_TYPE: &str =
        "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)";

    let mut encoded = Vec::with_capacity(32 * 5);
    encoded.extend_from_slice(&keccak256(DOMAIN_TYPE.as_bytes()));
    encoded.extend_from_slice(&encode_string_hash(&domain.name));
    encoded.extend_from_slice(&encode_string_hash(&domain.version));
    encoded.extend_from_slice(&chain_id_bytes(domain.chain_id)?);
    encoded.extend_from_slice(&encode_address(&domain.verifying_contract)?);
    Ok(keccak256(encoded))
}

pub(crate) fn typed_data_digest(
    domain: &TypedDataDomain,
    struct_hash: [u8; 32],
) -> Result<[u8; 32], ContractsError> {
    let mut payload = Vec::with_capacity(66);
    payload.extend_from_slice(&[0x19, 0x01]);
    payload.extend_from_slice(&domain_separator(domain)?);
    payload.extend_from_slice(&struct_hash);
    Ok(keccak256(payload))
}

pub(crate) fn normalize_hex_payload(
    value: &str,
    field: &'static str,
) -> Result<String, ContractsError> {
    let bytes = parse_hex(value, field)?;
    Ok(format!("0x{}", hex::encode(bytes)))
}

pub(crate) fn function_selector(signature: &str) -> [u8; 4] {
    let hash = keccak256(signature.as_bytes());
    [hash[0], hash[1], hash[2], hash[3]]
}

pub(crate) fn encode_fixed_bytes<const N: usize>(bytes: [u8; N]) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[..N].copy_from_slice(&bytes);
    out
}

pub(crate) fn abi_encode_bytes_array(items: &[Vec<u8>]) -> Vec<u8> {
    let mut encoded = Vec::new();
    encoded.extend_from_slice(&encode_u256_usize(32));

    let mut array_data = Vec::new();
    array_data.extend_from_slice(&encode_u256_usize(items.len()));

    let mut offsets = Vec::new();
    let mut tail = Vec::new();
    let mut current_offset = 32 * items.len();

    for item in items {
        offsets.push(current_offset);
        tail.extend_from_slice(&encode_u256_usize(item.len()));
        tail.extend_from_slice(item);
        let padding = padded_len(item.len()) - item.len();
        tail.extend(std::iter::repeat_n(0u8, padding));
        current_offset += 32 + padded_len(item.len());
    }

    for offset in offsets {
        array_data.extend_from_slice(&encode_u256_usize(offset));
    }
    array_data.extend_from_slice(&tail);
    encoded.extend_from_slice(&array_data);
    encoded
}

pub(crate) fn padded_len(len: usize) -> usize {
    if len == 0 {
        0
    } else {
        ((len - 1) / 32 + 1) * 32
    }
}

pub(crate) fn encode_u256_usize(value: usize) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[24..].copy_from_slice(&(value as u64).to_be_bytes());
    out
}
