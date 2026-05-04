use num_bigint::BigUint;
use sha3::{Digest, Keccak256};

use cow_sdk_core::{
    Address, AppDataHash, BuyTokenDestination, ChainId, OrderKind, SellTokenSource, TypedDataDomain,
};

use crate::ContractsError;

pub(crate) const ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";
pub(crate) const ORDER_UID_LENGTH_BYTES: usize = 56;

/// Returns the EVM zero address constant.
///
/// # Panics
///
/// Panics only if the crate-owned zero-address literal stops being a valid
/// EVM address.
pub(crate) fn zero_address() -> Address {
    // SAFETY: ZERO_ADDRESS is a reviewed protocol literal with the exact EVM
    // address shape.
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
        return Err(ContractsError::InvalidHexPrefix { field });
    };
    hex::decode(stripped).map_err(|source| ContractsError::DecodeHex { field, source })
}

pub(crate) fn parse_hex_exact(
    value: &str,
    field: &'static str,
    expected: usize,
) -> Result<Vec<u8>, ContractsError> {
    let bytes = parse_hex(value, field)?;
    if bytes.len() != expected {
        return Err(ContractsError::InvalidDecodedLength {
            field,
            expected,
            actual: bytes.len(),
        });
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
    let parsed = value
        .strip_prefix("0x")
        .map_or_else(
            || BigUint::parse_bytes(value.as_bytes(), 10),
            |stripped| BigUint::parse_bytes(stripped.as_bytes(), 16),
        )
        .ok_or_else(|| ContractsError::InvalidNumeric {
            field,
            value: value.to_owned().into(),
        })?;

    encode_u256_biguint_inner(field, &parsed, || value.to_owned())
}

pub(crate) fn encode_u256_biguint(value: &BigUint) -> Result<[u8; 32], ContractsError> {
    encode_u256_biguint_inner("amount", value, || value.to_str_radix(10))
}

fn encode_u256_biguint_inner(
    field: &'static str,
    value: &BigUint,
    display: impl FnOnce() -> String,
) -> Result<[u8; 32], ContractsError> {
    let bytes = value.to_bytes_be();
    if bytes.len() > 32 {
        return Err(ContractsError::NumericOverflow {
            field,
            value: display().into(),
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

pub(crate) const fn order_kind_name(kind: OrderKind) -> &'static str {
    match kind {
        OrderKind::Buy => "buy",
        OrderKind::Sell => "sell",
    }
}

/// Returns the settlement flag label for a supported sell-token balance source.
///
/// # Panics
///
/// Panics only if a new balance-source variant reaches this internal codec
/// before the settlement flag mapping is updated.
pub(crate) fn sell_balance_name(balance: SellTokenSource) -> &'static str {
    match balance {
        SellTokenSource::Erc20 => "erc20",
        SellTokenSource::External => "external",
        SellTokenSource::Internal => "internal",
        // SAFETY: all currently representable settlement sell-token balance
        // variants are handled above; new variants must update this codec.
        _ => unreachable!("SellTokenSource variants are exhaustively covered"),
    }
}

/// Returns the settlement flag label for a supported buy-token balance destination.
///
/// # Panics
///
/// Panics only if a new balance-destination variant reaches this internal codec
/// before the settlement flag mapping is updated.
pub(crate) fn buy_balance_name(balance: BuyTokenDestination) -> &'static str {
    match balance {
        BuyTokenDestination::Erc20 => "erc20",
        BuyTokenDestination::Internal => "internal",
        // SAFETY: all currently representable settlement buy-token balance
        // variants are handled above; new variants must update this codec.
        _ => unreachable!("BuyTokenDestination variants are exhaustively covered"),
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

#[cfg(test)]
mod tests {
    use super::*;
    use sha3::{Digest, Keccak256};

    fn u256_word_from_u64(value: u64) -> [u8; 32] {
        let mut out = [0u8; 32];
        out[24..].copy_from_slice(&value.to_be_bytes());
        out
    }

    fn address_word(address: &Address) -> [u8; 32] {
        let mut out = [0u8; 32];
        let decoded = hex::decode(address.as_str().trim_start_matches("0x")).unwrap();
        out[12..].copy_from_slice(&decoded);
        out
    }

    #[test]
    fn hex_parsers_and_scalar_encoders_preserve_exact_abi_words() {
        let address = Address::new("0x1234567890abcdef1234567890abcdef12345678").unwrap();
        let app_data =
            AppDataHash::new("0xabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd")
                .unwrap();

        assert_eq!(parse_address_bytes(&address).unwrap(), {
            let mut expected = [0u8; 20];
            expected
                .copy_from_slice(&hex::decode("1234567890abcdef1234567890abcdef12345678").unwrap());
            expected
        });
        assert_eq!(parse_bytes32_hash(&app_data).unwrap(), {
            let mut expected = [0u8; 32];
            expected.copy_from_slice(
                &hex::decode("abcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcdefabcd")
                    .unwrap(),
            );
            expected
        });
        assert_eq!(
            parse_hex32(
                "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "value"
            )
            .unwrap(),
            [0xaa; 32]
        );
        assert_eq!(encode_address(&address).unwrap(), address_word(&address));
        assert_eq!(encode_u32(0x0102_0304), {
            let mut out = [0u8; 32];
            out[28..].copy_from_slice(&0x0102_0304u32.to_be_bytes());
            out
        });
        assert_eq!(
            encode_u256_str(
                "amount",
                "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
            )
            .unwrap(),
            [0xff; 32]
        );
        assert!(
            encode_u256_str(
                "amount",
                "0x01ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
            )
            .is_err()
        );
        assert_eq!(encode_bool(false), [0u8; 32]);
        assert_eq!(encode_bool(true)[31], 1);
        assert_eq!(order_kind_name(OrderKind::Buy), "buy");
        assert_eq!(order_kind_name(OrderKind::Sell), "sell");
        assert_eq!(chain_id_bytes(1).unwrap(), u256_word_from_u64(1));
        assert_eq!(
            normalize_hex_payload("0xABcd", "payload").unwrap(),
            "0xabcd"
        );
        let expected_string_hash: [u8; 32] = Keccak256::digest(b"hello").into();
        assert_eq!(encode_string_hash("hello"), expected_string_hash);
    }

    #[test]
    fn domain_separator_and_typed_data_digest_match_manual_eip712_encoding() {
        let domain = TypedDataDomain::new(
            "Gnosis Protocol".to_owned(),
            "v2".to_owned(),
            1,
            Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41").unwrap(),
        );
        let struct_hash = [0x55; 32];

        let mut encoded = Vec::new();
        encoded.extend_from_slice(&Keccak256::digest(
            "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
                .as_bytes(),
        ));
        encoded.extend_from_slice(&Keccak256::digest(domain.name.as_bytes()));
        encoded.extend_from_slice(&Keccak256::digest(domain.version.as_bytes()));
        encoded.extend_from_slice(&u256_word_from_u64(domain.chain_id));
        encoded.extend_from_slice(&address_word(&domain.verifying_contract));
        let expected_separator: [u8; 32] = Keccak256::digest(&encoded).into();

        let mut digest_payload = Vec::with_capacity(66);
        digest_payload.extend_from_slice(&[0x19, 0x01]);
        digest_payload.extend_from_slice(&expected_separator);
        digest_payload.extend_from_slice(&struct_hash);
        let expected_digest: [u8; 32] = Keccak256::digest(&digest_payload).into();

        assert_eq!(domain_separator(&domain).unwrap(), expected_separator);
        assert_eq!(
            typed_data_digest(&domain, struct_hash).unwrap(),
            expected_digest
        );
    }

    #[test]
    fn domain_separator_matches_shared_parity_fixture() {
        let (domain, expected_separator) = domain_separator_parity_fixture();
        let actual_separator = format!("0x{}", hex::encode(domain_separator(&domain).unwrap()));

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
