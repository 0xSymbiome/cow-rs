use alloy_primitives::keccak256;
use cow_sdk_core::{Address, Hash32, OrderDigest, OrderUid, TypedDataDomain};

use super::{NormalizedOrder, ORDER_TYPE_HASH, ORDER_UID_LENGTH, Order, OrderCancellations};
use crate::{
    ContractsError,
    primitives::{
        buy_balance_name, encode_address, encode_bool, encode_string_hash, encode_u32,
        encode_u256_biguint, order_kind_name, parse_bytes32_hash, parse_hex_exact, parse_hex32,
        sell_balance_name, typed_data_digest, zero_address,
    },
};

/// Normalizes an order into its canonical contract hashing form.
///
/// # Errors
///
/// Returns [`ContractsError::ZeroReceiver`] when the receiver is explicitly set
/// to the zero address.
pub fn normalize_order(order: &Order) -> Result<NormalizedOrder, ContractsError> {
    if matches!(
        order
            .receiver
            .as_ref()
            .map(Address::normalized_key)
            .as_deref(),
        Some(ZERO_ADDRESS_LOWER)
    ) {
        return Err(ContractsError::ZeroReceiver);
    }

    Ok(NormalizedOrder::new(
        order.sell_token.clone(),
        order.buy_token.clone(),
        order.receiver.clone().unwrap_or_else(zero_address),
        order.sell_amount.clone(),
        order.buy_amount.clone(),
        order.valid_to,
        order.app_data.clone(),
        order.fee_amount.clone(),
        order.kind,
        order.partially_fillable,
        order.sell_token_balance.unwrap_or_default(),
        order.buy_token_balance.unwrap_or_default(),
    ))
}

/// Computes the EIP-712 digest for an order.
///
/// # Errors
///
/// Returns [`ContractsError`] if normalization or struct hashing fails.
pub fn hash_order(domain: &TypedDataDomain, order: &Order) -> Result<OrderDigest, ContractsError> {
    let digest = typed_data_digest(domain, order_struct_hash(&normalize_order(order)?)?)?;
    OrderDigest::new(format!("0x{}", hex::encode(digest))).map_err(Into::into)
}

/// Computes the EIP-712 digest for a single order cancellation.
///
/// # Errors
///
/// Returns [`ContractsError`] if UID decoding or typed-data hashing fails.
pub fn hash_order_cancellation(
    domain: &TypedDataDomain,
    order_uid: &OrderUid,
) -> Result<Hash32, ContractsError> {
    hash_order_cancellations(domain, &OrderCancellations::new(vec![order_uid.clone()]))
}

/// Computes the EIP-712 digest for a batch order cancellation payload.
///
/// # Errors
///
/// Returns [`ContractsError`] if UID decoding or typed-data hashing fails.
pub fn hash_order_cancellations(
    domain: &TypedDataDomain,
    cancellations: &OrderCancellations,
) -> Result<Hash32, ContractsError> {
    let type_hash = keccak256(b"OrderCancellations(bytes[] orderUids)");
    let mut concatenated = Vec::with_capacity(cancellations.order_uids.len() * 32);
    for uid in &cancellations.order_uids {
        let bytes = parse_hex_exact(uid.as_str(), "orderUid", ORDER_UID_LENGTH)?;
        concatenated.extend_from_slice(keccak256(&bytes).as_slice());
    }
    let array_hash = keccak256(&concatenated);

    let mut encoded = Vec::with_capacity(64);
    encoded.extend_from_slice(type_hash.as_slice());
    encoded.extend_from_slice(array_hash.as_slice());
    let digest = typed_data_digest(domain, keccak256(&encoded).0)?;
    Hash32::new(format!("0x{}", hex::encode(digest))).map_err(Into::into)
}

fn order_struct_hash(order: &NormalizedOrder) -> Result<[u8; 32], ContractsError> {
    let mut encoded = Vec::with_capacity(32 * 13);
    encoded.extend_from_slice(&parse_hex32(ORDER_TYPE_HASH, "orderTypeHash")?);
    encoded.extend_from_slice(&encode_address(&order.sell_token)?);
    encoded.extend_from_slice(&encode_address(&order.buy_token)?);
    encoded.extend_from_slice(&encode_address(&order.receiver)?);
    encoded.extend_from_slice(&encode_u256_biguint(order.sell_amount.as_biguint())?);
    encoded.extend_from_slice(&encode_u256_biguint(order.buy_amount.as_biguint())?);
    encoded.extend_from_slice(&encode_u32(order.valid_to));
    encoded.extend_from_slice(&parse_bytes32_hash(&order.app_data)?);
    encoded.extend_from_slice(&encode_u256_biguint(order.fee_amount.as_biguint())?);
    encoded.extend_from_slice(&encode_string_hash(order_kind_name(order.kind)));
    encoded.extend_from_slice(&encode_bool(order.partially_fillable));
    encoded.extend_from_slice(&encode_string_hash(sell_balance_name(
        order.sell_token_balance,
    )));
    encoded.extend_from_slice(&encode_string_hash(buy_balance_name(
        order.buy_token_balance,
    )));
    Ok(keccak256(&encoded).0)
}

const ZERO_ADDRESS_LOWER: &str = "0x0000000000000000000000000000000000000000";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deployments::{ContractId, Registry};
    use cow_sdk_core::{
        Amount, AppDataHash, BuyTokenDestination, CowEnv, OrderKind, SellTokenSource,
        SupportedChainId,
    };
    use num_bigint::BigUint;
    use sha3::{Digest, Keccak256};

    fn sample_domain() -> TypedDataDomain {
        TypedDataDomain::new(
            "Gnosis Protocol".to_owned(),
            "v2".to_owned(),
            1,
            Registry::default()
                .address(
                    ContractId::Settlement,
                    SupportedChainId::Mainnet,
                    CowEnv::Prod,
                )
                .expect("canonical settlement address is registered for every supported chain"),
        )
    }

    fn sample_order() -> Order {
        Order::new(
            Address::new("0x1111111111111111111111111111111111111111").unwrap(),
            Address::new("0x2222222222222222222222222222222222222222").unwrap(),
            None,
            Amount::new("1000").unwrap(),
            Amount::new("900").unwrap(),
            1_700_000_000,
            AppDataHash::new("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
                .unwrap(),
            Amount::new("10").unwrap(),
            OrderKind::Sell,
            true,
            Some(SellTokenSource::External),
            Some(BuyTokenDestination::Internal),
        )
    }

    fn encode_address_word(address: &Address) -> [u8; 32] {
        let mut out = [0u8; 32];
        let decoded = hex::decode(address.as_str().trim_start_matches("0x")).unwrap();
        out[12..].copy_from_slice(&decoded);
        out
    }

    fn encode_u256_word(value: &str) -> [u8; 32] {
        let parsed = value
            .strip_prefix("0x")
            .map_or_else(
                || BigUint::parse_bytes(value.as_bytes(), 10),
                |stripped| BigUint::parse_bytes(stripped.as_bytes(), 16),
            )
            .unwrap();
        let bytes = parsed.to_bytes_be();
        let mut out = [0u8; 32];
        out[32 - bytes.len()..].copy_from_slice(&bytes);
        out
    }

    fn encode_u32_word(value: u32) -> [u8; 32] {
        let mut out = [0u8; 32];
        out[28..].copy_from_slice(&value.to_be_bytes());
        out
    }

    // Hand-rolled `sha3::Keccak256` helper used by the assertions below.
    // Crate code routes through `alloy_primitives::keccak256` per
    // ADR 0052; this helper deliberately runs `sha3::Keccak256` directly
    // so the parity check compares the crate output against an
    // independent keccak implementation.
    fn keccak_word(value: &str) -> [u8; 32] {
        Keccak256::digest(value.as_bytes()).into()
    }

    fn manual_domain_separator(domain: &TypedDataDomain) -> [u8; 32] {
        let mut encoded = Vec::new();
        encoded.extend_from_slice(&Keccak256::digest(
            "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
                .as_bytes(),
        ));
        encoded.extend_from_slice(&keccak_word(&domain.name));
        encoded.extend_from_slice(&keccak_word(&domain.version));
        encoded.extend_from_slice(&encode_u256_word(&domain.chain_id.to_string()));
        encoded.extend_from_slice(&encode_address_word(&domain.verifying_contract));
        Keccak256::digest(&encoded).into()
    }

    fn manual_struct_hash(order: &NormalizedOrder) -> [u8; 32] {
        let mut encoded = Vec::new();
        encoded.extend_from_slice(&hex::decode(ORDER_TYPE_HASH.trim_start_matches("0x")).unwrap());
        encoded.extend_from_slice(&encode_address_word(&order.sell_token));
        encoded.extend_from_slice(&encode_address_word(&order.buy_token));
        encoded.extend_from_slice(&encode_address_word(&order.receiver));
        encoded.extend_from_slice(&encode_u256_word(&order.sell_amount.to_string()));
        encoded.extend_from_slice(&encode_u256_word(&order.buy_amount.to_string()));
        encoded.extend_from_slice(&encode_u32_word(order.valid_to));
        encoded.extend_from_slice(
            &hex::decode(order.app_data.as_str().trim_start_matches("0x")).unwrap(),
        );
        encoded.extend_from_slice(&encode_u256_word(&order.fee_amount.to_string()));
        encoded.extend_from_slice(&keccak_word("sell"));
        encoded.extend_from_slice(&{
            let mut out = [0u8; 32];
            out[31] = 1;
            out
        });
        encoded.extend_from_slice(&keccak_word("external"));
        encoded.extend_from_slice(&keccak_word("internal"));
        Keccak256::digest(&encoded).into()
    }

    #[test]
    fn order_hash_and_struct_hash_match_manual_eip712_encoding() {
        let domain = sample_domain();
        let order = sample_order();
        let normalized = normalize_order(&order).unwrap();
        let expected_struct_hash = manual_struct_hash(&normalized);

        let mut digest_payload = Vec::with_capacity(66);
        digest_payload.extend_from_slice(&[0x19, 0x01]);
        digest_payload.extend_from_slice(&manual_domain_separator(&domain));
        digest_payload.extend_from_slice(&expected_struct_hash);
        let expected_digest = Keccak256::digest(&digest_payload);

        assert_eq!(
            order_struct_hash(&normalized).unwrap(),
            expected_struct_hash
        );
        assert_eq!(
            hash_order(&domain, &order).unwrap().as_str(),
            format!("0x{}", hex::encode(expected_digest))
        );
    }
}
