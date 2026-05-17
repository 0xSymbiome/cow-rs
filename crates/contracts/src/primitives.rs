use alloy_primitives::keccak256;

use cow_sdk_core::{Address, AppDataHash, BuyTokenDestination, OrderKind, SellTokenSource};

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

    #[test]
    fn hex_parsers_round_trip_typed_byte_arrays() {
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
        assert_eq!(order_kind_name(OrderKind::Buy), "buy");
        assert_eq!(order_kind_name(OrderKind::Sell), "sell");
        assert_eq!(
            normalize_hex_payload("0xABcd", "payload").unwrap(),
            "0xabcd"
        );
    }
}
