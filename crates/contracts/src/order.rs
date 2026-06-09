//! Order hashing, UID packing, EIP-712 metadata, and the `GPv2` typed-data
//! `sol!` bindings.
//!
//! This module owns the order surface: the EIP-712 digest and type-hash helpers
//! ([`hash_order`], [`hash_order_cancellations`], [`order_eip712_type_hash`]),
//! the 56-byte UID pack/unpack codec ([`compute_order_uid`],
//! [`pack_order_uid_params`], [`extract_order_uid_params`]), the canonical
//! EIP-712 field tables, and the macro-emitted `Order` / `OrderCancellations`
//! codec structs.
//!
//! The generated `sol!` structs live in the private [`sol`] submodule because
//! the codec `OrderCancellations` shares its name with the public domain
//! [`OrderCancellations`] message type. This crate owns no public order *type*
//! of its own — the user-domain order type is [`cow_sdk_core::OrderData`] — and
//! exposes hashing / UID / encoding *functions* over it. The cancellation codec
//! struct is re-exported at the crate root as `GPv2OrderCancellations`.

use alloy_primitives::Bytes as AlloyBytes;
use alloy_sol_types::SolStruct;
use serde::{Deserialize, Serialize};

use cow_sdk_core::{Address, Hash32, OrderData, OrderDigest, OrderUid, TypedDataDomain};

use crate::ContractsError;
use crate::primitives::{
    ORDER_UID_LENGTH_BYTES, buy_balance_name, order_kind_name, sell_balance_name,
};

use self::sol::{Order as SolOrder, OrderCancellations as SolOrderCancellations};

/// Sentinel address used by the protocol to represent native ETH buys.
pub const BUY_ETH_ADDRESS: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";
/// Encoded order UID length in bytes.
pub const ORDER_UID_LENGTH: usize = ORDER_UID_LENGTH_BYTES;

/// EIP-712 field descriptor used for `CoW` order-type metadata.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderTypeField {
    /// Field name.
    pub name: &'static str,
    /// Solidity field type.
    #[serde(rename = "type")]
    pub kind: &'static str,
}

impl OrderTypeField {
    /// Creates an order-type field descriptor.
    #[must_use]
    pub const fn new(name: &'static str, kind: &'static str) -> Self {
        Self { name, kind }
    }
}

/// Canonical order type fields in struct-hash order.
pub const ORDER_TYPE_FIELDS: [OrderTypeField; 12] = [
    OrderTypeField::new("sellToken", "address"),
    OrderTypeField::new("buyToken", "address"),
    OrderTypeField::new("receiver", "address"),
    OrderTypeField::new("sellAmount", "uint256"),
    OrderTypeField::new("buyAmount", "uint256"),
    OrderTypeField::new("validTo", "uint32"),
    OrderTypeField::new("appData", "bytes32"),
    OrderTypeField::new("feeAmount", "uint256"),
    OrderTypeField::new("kind", "string"),
    OrderTypeField::new("partiallyFillable", "bool"),
    OrderTypeField::new("sellTokenBalance", "string"),
    OrderTypeField::new("buyTokenBalance", "string"),
];

/// Canonical EIP-712 field descriptor for order-cancellation payloads.
pub const CANCELLATIONS_TYPE_FIELDS: [OrderTypeField; 1] =
    [OrderTypeField::new("orderUids", "bytes[]")];

/// Macro-emitted `GPv2` typed-data codec structs.
///
/// Kept in a private submodule because the `OrderCancellations` codec struct
/// shares its name with the public domain [`OrderCancellations`] message type.
/// The Rust struct names MUST stay `Order` and `OrderCancellations` (not
/// `GPv2Order`/`GPv2OrderCancellations` or any other variant) because the alloy
/// `sol!` macro derives the EIP-712 type-name prefix from the Rust struct name;
/// renaming either would change the type-hash bytes.
mod sol {
    alloy_sol_types::sol! {
        /// `GPv2` settlement `Order` typed-data struct. The canonical type
        /// string keccak256-hashes to the protocol constant
        /// `0xd5a25ba2e97094ad7d83dc28a6572da797d6b3e7fc6663bd93efb789fc17e489`.
        #[derive(Debug, Default, PartialEq, Eq)]
        struct Order {
            address sellToken;
            address buyToken;
            address receiver;
            uint256 sellAmount;
            uint256 buyAmount;
            uint32 validTo;
            bytes32 appData;
            uint256 feeAmount;
            string kind;
            bool partiallyFillable;
            string sellTokenBalance;
            string buyTokenBalance;
        }

        /// `GPv2` batch order cancellation typed-data struct. Canonical type
        /// string `OrderCancellations(bytes[] orderUids)`.
        #[derive(Debug, Default, PartialEq, Eq)]
        struct OrderCancellations {
            bytes[] orderUids;
        }
    }

    #[cfg(test)]
    mod tests {
        use super::Order;
        use alloy_primitives::b256;
        use alloy_sol_types::SolStruct;

        /// Pins the macro-emitted `GPv2` `Order` type hash to the deployed
        /// protocol constant.
        #[test]
        fn order_type_hash_matches_protocol_constant() {
            let expected =
                b256!("0xd5a25ba2e97094ad7d83dc28a6572da797d6b3e7fc6663bd93efb789fc17e489");
            let sample = Order::default();
            assert_eq!(sample.eip712_type_hash(), expected);
        }
    }
}

pub use self::sol::OrderCancellations as GPv2OrderCancellations;

/// Structured order UID components.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderUidParams {
    /// Order digest.
    pub order_digest: OrderDigest,
    /// Order owner address.
    pub owner: Address,
    /// Order expiration timestamp.
    pub valid_to: u32,
}

/// EIP-712 message body for order cancellations.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderCancellations {
    /// Order UIDs being cancelled.
    pub order_uids: Vec<OrderUid>,
}

impl OrderUidParams {
    /// Creates structured order UID components.
    #[must_use]
    pub const fn new(order_digest: OrderDigest, owner: Address, valid_to: u32) -> Self {
        Self {
            order_digest,
            owner,
            valid_to,
        }
    }
}

impl OrderCancellations {
    /// Creates an order-cancellation payload.
    #[must_use]
    pub const fn new(order_uids: Vec<OrderUid>) -> Self {
        Self { order_uids }
    }
}

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

/// Computes the EIP-712 digest for an order.
///
/// Returns the canonical
/// `keccak256(0x19 || 0x01 || domain_separator || struct_hash)`
/// envelope per the EIP-712 specification, evaluated against the
/// macro-emitted internal `Order` codec struct hash. The concrete
/// [`cow_sdk_core::OrderData`] maps straight onto that codec struct, so no
/// intermediate normalization step is needed. The
/// `parity/fixtures/eip712/order_digests.json` rows lock the per-row
/// byte contract, and [`order_eip712_type_hash`] exposes the matching
/// EIP-712 type hash.
///
/// A `receiver` of `address(0)` is hashed verbatim: the `GPv2` order surface
/// reads it as the `RECEIVER_SAME_AS_OWNER` (pay-to-owner) sentinel, so this
/// general hash path never rejects it.
///
/// # Errors
///
/// Returns [`ContractsError`] if address parsing fails. The signature stays
/// fallible so callers can thread `?` through the shared [`ContractsError`]
/// envelope.
pub fn hash_order(
    domain: &TypedDataDomain,
    order: &OrderData,
) -> Result<OrderDigest, ContractsError> {
    let sol_order = sol_order_from_order_data(order);
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
/// macro-emitted [`sol::OrderCancellations`] struct hash.
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

/// Returns the canonical EIP-712 `Order` type hash.
///
/// This is `keccak256` of the `Order(address sellToken,…)` EIP-712 type string
/// the `GPv2Settlement` contract verifies against; it matches the upstream
/// services `OrderData::TYPE_HASH`. Callers verifying an order digest or its
/// signed typed-data envelope can pin this value.
#[must_use]
pub fn order_eip712_type_hash() -> Hash32 {
    Hash32::from_bytes(SolOrder::default().eip712_type_hash().0)
}

/// Maps a concrete user-domain [`cow_sdk_core::OrderData`] straight onto the
/// macro-emitted `GPv2` `Order` codec struct.
///
/// There is no "normalize" step: `OrderData` carries a concrete receiver and
/// concrete balance enums, so the historical Option-fill collapses to a
/// verbatim field copy. The cow `Amount` newtype is `#[repr(transparent)]`
/// over `alloy_primitives::U256` and `AppDataHash` over
/// `alloy_primitives::B256` per ADR 0052, so the conversions to the sol-typed
/// surface are a single deref of the inner alloy primitive with no
/// intermediate bigint allocation and no overflow guard required.
fn sol_order_from_order_data(order: &OrderData) -> SolOrder {
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

/// Computes the encoded order UID for an order and owner.
///
/// # Errors
///
/// Returns [`ContractsError`] if order hashing or UID packing fails.
#[inline]
pub fn compute_order_uid(
    domain: &TypedDataDomain,
    order: &OrderData,
    owner: &Address,
) -> Result<OrderUid, ContractsError> {
    pack_order_uid_params(&OrderUidParams::new(
        hash_order(domain, order)?,
        *owner,
        order.valid_to,
    ))
}

/// Packs structured order UID components into the compact UID string.
///
/// # Errors
///
/// Returns [`ContractsError`] if the digest or owner cannot be decoded into the
/// fixed byte lengths required by the UID format.
#[inline]
pub fn pack_order_uid_params(params: &OrderUidParams) -> Result<OrderUid, ContractsError> {
    let digest = params.order_digest.into_alloy().0;
    let owner = params.owner.into_alloy().0.0;
    let mut out = [0u8; ORDER_UID_LENGTH];
    out[..32].copy_from_slice(&digest);
    out[32..52].copy_from_slice(&owner);
    out[52..56].copy_from_slice(&params.valid_to.to_be_bytes());
    Ok(OrderUid::from_bytes(out))
}

/// Extracts structured order UID components from a compact UID string.
///
/// # Errors
///
/// Returns [`ContractsError`] if the UID cannot be decoded into the expected format.
///
/// # Panics
///
/// Cannot panic in practice. The function returns early with
/// [`ContractsError::InvalidOrderUidLength`] when the byte length is
/// not exactly [`ORDER_UID_LENGTH`]; after that guard, the internal
/// 32-byte and 20-byte slice-to-array conversions are infallible by
/// construction. The `expect` calls inside the body document the
/// unreachability proof so a future contributor cannot accidentally
/// weaken the guard without removing the proof first.
#[inline]
pub fn extract_order_uid_params(order_uid: &OrderUid) -> Result<OrderUidParams, ContractsError> {
    let bytes = order_uid.as_slice();
    if bytes.len() != ORDER_UID_LENGTH {
        return Err(ContractsError::InvalidOrderUidLength {
            actual: bytes.len(),
        });
    }

    // SAFETY: the `bytes.len() != ORDER_UID_LENGTH` guard above guarantees
    // `bytes.len() == 56` here, so the `[..32]` and `[32..52]` slices are
    // always 32 and 20 bytes respectively and `try_into` cannot fail.
    let order_digest = OrderDigest::from_bytes(
        bytes[..32]
            .try_into()
            .expect("slice length 32 is guaranteed by the ORDER_UID_LENGTH check above"),
    );
    let owner = Address::from_bytes(
        bytes[32..52]
            .try_into()
            .expect("slice length 20 is guaranteed by the ORDER_UID_LENGTH check above"),
    );
    let valid_to_bytes: [u8; 4] =
        bytes[52..56]
            .try_into()
            .map_err(|_| ContractsError::InvalidOrderUidLength {
                actual: bytes.len(),
            })?;
    let valid_to = u32::from_be_bytes(valid_to_bytes);

    Ok(OrderUidParams::new(order_digest, owner, valid_to))
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::U256;
    use cow_sdk_core::{
        Amount, AppDataHash, BuyTokenDestination, OrderData, OrderKind, SellTokenSource,
    };
    use cow_sdk_test_utils::builders::sample_domain;
    use sha3::{Digest, Keccak256};
    use std::str::FromStr;

    fn sample_order() -> OrderData {
        OrderData::new(
            Address::new("0x1111111111111111111111111111111111111111").unwrap(),
            Address::new("0x2222222222222222222222222222222222222222").unwrap(),
            Address::ZERO,
            Amount::new("1000").unwrap(),
            Amount::new("900").unwrap(),
            1_700_000_000,
            AppDataHash::new("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
                .unwrap(),
            Amount::new("10").unwrap(),
            OrderKind::Sell,
            true,
            SellTokenSource::External,
            BuyTokenDestination::Internal,
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

    // Independent test oracle: right-aligns an address into a 32-byte word
    // by hand so the EIP-712 parity assertions below do not verify alloy's
    // `Address::into_word` against itself.
    fn encode_address_word(address: &Address) -> [u8; 32] {
        let mut out = [0u8; 32];
        out[12..].copy_from_slice(address.as_slice());
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

    fn manual_struct_hash(order: &OrderData) -> [u8; 32] {
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
        let expected_struct_hash = manual_struct_hash(&order);

        let mut digest_payload = Vec::with_capacity(66);
        digest_payload.extend_from_slice(&[0x19, 0x01]);
        digest_payload.extend_from_slice(&manual_domain_separator(&domain));
        digest_payload.extend_from_slice(&expected_struct_hash);
        let expected_digest = Keccak256::digest(&digest_payload);

        let sol_order = sol_order_from_order_data(&order);
        assert_eq!(sol_order.eip712_hash_struct().0, expected_struct_hash);
        assert_eq!(
            hash_order(&domain, &order).unwrap().to_hex_string(),
            alloy_primitives::hex::encode_prefixed(expected_digest)
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
