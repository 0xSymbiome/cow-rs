use alloy_primitives::Bytes as AlloyBytes;
use alloy_sol_types::SolStruct;
use cow_sdk_core::{Address, Hash32, OrderDigest, OrderUid, TypedDataDomain};

use super::sol_cancellations::OrderCancellations as SolOrderCancellations;
use super::sol_types::Order as SolOrder;
use super::{NormalizedOrder, Order, OrderCancellations};
use crate::ContractsError;
use crate::primitives::{buy_balance_name, order_kind_name, sell_balance_name};

/// Rejects construction paths that would emit `address(0)` as the order
/// receiver. The cow-protocol `GPv2` order surface treats `address(0)` as
/// the "send to owner" sentinel via `GPv2Order.RECEIVER_SAME_AS_OWNER`,
/// and the `EthFlow` contract additionally reverts at calldata-construction
/// time with `ReceiverMustBeSet()` (selector `0xefc9ccdf`) because the
/// order owner is always the `EthFlow` contract itself — routing proceeds
/// to "owner" would strand ERC-20 tokens in the contract.
///
/// See the `EthFlowOrder.toCoWSwapOrder` library function in the
/// `cowprotocol/ethflowcontract` Solidity surface for the upstream
/// rationale; the cow `parity/source-lock.yaml` `id: ethflowcontract`
/// block pins the canonical SHA.
///
/// # Errors
///
/// Returns [`ContractsError::ZeroReceiver`] when `receiver` is the zero
/// address.
#[inline]
pub(crate) fn reject_zero_receiver(receiver: &Address) -> Result<(), ContractsError> {
    if receiver.is_zero() {
        Err(ContractsError::ZeroReceiver)
    } else {
        Ok(())
    }
}

/// Normalizes an order into its canonical contract hashing form.
///
/// # Errors
///
/// Returns [`ContractsError::ZeroReceiver`] when the receiver is explicitly set
/// to the zero address.
pub fn normalize_order(order: &Order) -> Result<NormalizedOrder, ContractsError> {
    if let Some(receiver) = order.receiver.as_ref() {
        reject_zero_receiver(receiver)?;
    }

    Ok(NormalizedOrder::new(
        order.sell_token,
        order.buy_token,
        order.receiver.unwrap_or(Address::ZERO),
        order.sell_amount,
        order.buy_amount,
        order.valid_to,
        order.app_data,
        order.fee_amount,
        order.kind,
        order.partially_fillable,
        order.sell_token_balance.unwrap_or_default(),
        order.buy_token_balance.unwrap_or_default(),
    ))
}

/// Computes the EIP-712 digest for an order.
///
/// Returns the canonical
/// `keccak256(0x19 || 0x01 || domain_separator || struct_hash)`
/// envelope per the EIP-712 specification, evaluated against the
/// macro-emitted [`crate::order::sol_types::Order`] struct hash. The
/// `parity/fixtures/eip712/order_digests.json` rows lock the per-row
/// byte contract.
///
/// # Errors
///
/// Returns [`ContractsError`] if order normalization or address parsing fails.
pub fn hash_order(domain: &TypedDataDomain, order: &Order) -> Result<OrderDigest, ContractsError> {
    let normalized = normalize_order(order)?;
    let sol_order = sol_order_from_normalized(&normalized);
    let alloy_domain = domain.into_alloy_domain();
    let digest = sol_order.eip712_signing_hash(&alloy_domain);
    Ok(OrderDigest::from_bytes(digest.into()))
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
    hash_order_cancellations(domain, &OrderCancellations::new(vec![*order_uid]))
}

/// Computes the EIP-712 digest for a batch order cancellation payload.
///
/// Returns the canonical
/// `keccak256(0x19 || 0x01 || domain_separator || struct_hash)`
/// envelope per the EIP-712 specification, evaluated against the
/// macro-emitted
/// [`crate::order::sol_cancellations::OrderCancellations`] struct hash.
///
/// # Errors
///
/// Returns [`ContractsError`] if UID decoding or address parsing fails.
pub fn hash_order_cancellations(
    domain: &TypedDataDomain,
    cancellations: &OrderCancellations,
) -> Result<Hash32, ContractsError> {
    let order_uids = cancellations
        .order_uids
        .iter()
        .map(decode_order_uid_bytes)
        .collect::<Vec<_>>();
    let sol_cancellations = SolOrderCancellations {
        orderUids: order_uids,
    };
    let alloy_domain = domain.into_alloy_domain();
    let digest = sol_cancellations.eip712_signing_hash(&alloy_domain);
    Ok(Hash32::from_bytes(digest.into()))
}

fn sol_order_from_normalized(order: &NormalizedOrder) -> SolOrder {
    // The cow `Amount` newtype is `#[repr(transparent)]` over
    // `alloy_primitives::U256` and `AppDataHash` over
    // `alloy_primitives::B256` per ADR 0052, so the conversions to the
    // sol-typed surface are a single deref of the inner alloy primitive
    // with no intermediate bigint allocation and no overflow guard
    // required.
    SolOrder {
        sellToken: *order.sell_token.as_alloy(),
        buyToken: *order.buy_token.as_alloy(),
        receiver: *order.receiver.as_alloy(),
        sellAmount: *order.sell_amount.as_u256(),
        buyAmount: *order.buy_amount.as_u256(),
        validTo: order.valid_to,
        appData: *order.app_data.as_alloy(),
        feeAmount: *order.fee_amount.as_u256(),
        kind: order_kind_name(order.kind).to_owned(),
        partiallyFillable: order.partially_fillable,
        sellTokenBalance: sell_balance_name(order.sell_token_balance).to_owned(),
        buyTokenBalance: buy_balance_name(order.buy_token_balance).to_owned(),
    }
}

fn decode_order_uid_bytes(uid: &OrderUid) -> AlloyBytes {
    AlloyBytes::from(uid.as_slice().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deployments::{ContractId, Registry};
    use crate::encode_address_word;
    use alloy_primitives::U256;
    use cow_sdk_core::{
        Amount, AppDataHash, BuyTokenDestination, CowEnv, OrderKind, SellTokenSource,
        SupportedChainId,
    };
    use sha3::{Digest, Keccak256};
    use std::str::FromStr;

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

    fn encode_u256_word(value: &str) -> [u8; 32] {
        // Test oracle helper: `U256::from_str` recognises both the decimal
        // and `0x`-prefixed hex forms used by the parity fixtures, so the
        // cow newtype migration drops the historical BigUint dependency
        // without losing the dual-radix surface.
        U256::from_str(value)
            .expect("test fixture value must parse to U256")
            .to_be_bytes::<32>()
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
        const ORDER_TYPE_STRING: &[u8] = b"Order(address sellToken,address buyToken,address receiver,uint256 sellAmount,uint256 buyAmount,uint32 validTo,bytes32 appData,uint256 feeAmount,string kind,bool partiallyFillable,string sellTokenBalance,string buyTokenBalance)";
        let mut encoded = Vec::new();
        let type_hash: [u8; 32] = Keccak256::digest(ORDER_TYPE_STRING).into();
        encoded.extend_from_slice(&type_hash);
        encoded.extend_from_slice(&encode_address_word(&order.sell_token));
        encoded.extend_from_slice(&encode_address_word(&order.buy_token));
        encoded.extend_from_slice(&encode_address_word(&order.receiver));
        encoded.extend_from_slice(&encode_u256_word(&order.sell_amount.to_string()));
        encoded.extend_from_slice(&encode_u256_word(&order.buy_amount.to_string()));
        encoded.extend_from_slice(&encode_u32_word(order.valid_to));
        encoded.extend_from_slice(order.app_data.as_slice());
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

        let sol_order = sol_order_from_normalized(&normalized);
        assert_eq!(sol_order.eip712_hash_struct().0, expected_struct_hash);
        assert_eq!(
            hash_order(&domain, &order).unwrap().to_hex_string(),
            format!("0x{}", hex::encode(expected_digest))
        );
    }

    #[test]
    fn cancellation_hash_and_uid_decoding_preserve_single_uid_bytes() {
        let domain = sample_domain();
        let uid = OrderUid::new(
            "0xdaaa7dddec9ad04cc101a121e3eed017eab4d3927c045d407d5ad6700eea2bf7fb3c7eb936caa12b5a884d612393969a557d430764060343",
        )
        .unwrap();

        let decoded = decode_order_uid_bytes(&uid);
        assert_eq!(decoded.as_ref(), uid.as_slice());

        let single = hash_order_cancellation(&domain, &uid).unwrap();
        let batch = hash_order_cancellations(&domain, &OrderCancellations::new(vec![uid])).unwrap();
        assert_eq!(single, batch);
        assert_ne!(single, Hash32::from_bytes([0u8; 32]));
    }
}
