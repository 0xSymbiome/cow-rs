#![no_main]

//! Fuzz target for the EIP-712 typed-data digest pipeline.
//!
//! Builds a plausible [`UnsignedOrder`] field vector plus a
//! [`TypedDataDomain`] from arbitrary input, converts it into the
//! contract-side [`Order`] representation, and hashes it through
//! [`hash_order`]. The target asserts:
//!
//! * [`hash_order`] is panic-free on every arbitrary input.
//!   Inputs rejected by the typed order normalizer return early because
//!   they exercise the documented validation path rather than a digest
//!   contract.
//! * [`hash_order`] is deterministic: hashing the same input twice
//!   produces the same digest.
//! * [`hash_order_cancellations`] is panic-free on every arbitrary
//!   input, exercising the second public typed-data digest helper in
//!   the same module.
//!
//! The target caps the effective structured-input width through the
//! `Arbitrary` derive: each `[u8; N]` field and each scalar consumes a
//! fixed byte budget from the fuzzer, so the input-size cap is
//! inherent to the struct shape rather than needing a separate
//! `MAX_FUZZ_INPUT` constant.

use cow_sdk_contracts::order::{Order, hash_order, hash_order_cancellations};
use cow_sdk_contracts::{OrderCancellations, OrderUidParams, pack_order_uid_params};
use cow_sdk_core::{
    Address, Amount, AppDataHash, BuyTokenDestination, ChainId, OrderDigest, OrderKind,
    SellTokenSource, TypedDataDomain, UnsignedOrder,
};
use libfuzzer_sys::{arbitrary::Arbitrary, fuzz_target};

/// Arbitrary-derived input that produces every field a `CoW` order
/// hashing pipeline needs, without any fallible string parsing.
#[derive(Debug, Arbitrary)]
struct FuzzInput {
    sell_token: [u8; 20],
    buy_token: [u8; 20],
    receiver: [u8; 20],
    sell_amount: u128,
    buy_amount: u128,
    fee_amount: u128,
    valid_to: u32,
    app_data: [u8; 32],
    kind_is_buy: bool,
    partially_fillable: bool,
    sell_token_balance_code: u8,
    buy_token_balance_code: u8,
    chain_id: u64,
    verifying_contract: [u8; 20],
    domain_name_len: u8,
    domain_version_len: u8,
    domain_name_ascii_seed: u8,
    domain_version_ascii_seed: u8,
}

fuzz_target!(|input: FuzzInput| {
    let unsigned = UnsignedOrder::new(
        Address::from_bytes(input.sell_token),
        Address::from_bytes(input.buy_token),
        Address::from_bytes(input.receiver),
        Amount::new(input.sell_amount.to_string())
            .expect("u128-to-string is always a valid uint256 decimal"),
        Amount::new(input.buy_amount.to_string())
            .expect("u128-to-string is always a valid uint256 decimal"),
        input.valid_to,
        AppDataHash::from_bytes(input.app_data),
        Amount::new(input.fee_amount.to_string())
            .expect("u128-to-string is always a valid uint256 decimal"),
        if input.kind_is_buy {
            OrderKind::Buy
        } else {
            OrderKind::Sell
        },
        input.partially_fillable,
        sell_token_source_from_code(input.sell_token_balance_code),
        buy_token_destination_from_code(input.buy_token_balance_code),
    );
    let order: Order = (&unsigned).into();

    let domain = TypedDataDomain {
        name: bounded_ascii(input.domain_name_ascii_seed, input.domain_name_len),
        version: bounded_ascii(input.domain_version_ascii_seed, input.domain_version_len),
        chain_id: ChainId::from(input.chain_id),
        verifying_contract: Address::from_bytes(input.verifying_contract),
    };

    // `hash_order` is deterministic for a fixed accepted input. Rejected
    // orders are valid typed failures for the normalizer and are not
    // crashes for this digest target.
    let first = match hash_order(&domain, &order) {
        Ok(digest) => digest,
        Err(_) => return,
    };
    let second = hash_order(&domain, &order).expect("hash_order must remain deterministic");
    assert_eq!(
        first, second,
        "hash_order must produce the same digest for identical inputs",
    );

    // Exercise the second public typed-data digest helper in the same
    // module with a single-UID cancellation payload built from the
    // just-packed order UID.
    let uid = pack_order_uid_params(&OrderUidParams::new(
        OrderDigest::from_bytes(input.app_data),
        Address::from_bytes(input.receiver),
        input.valid_to,
    ))
    .expect("pack_order_uid_params must accept hex-typed components from from_bytes");
    let cancellations = OrderCancellations::new(vec![uid]);
    let _ = hash_order_cancellations(&domain, &cancellations)
        .expect("hash_order_cancellations must accept a just-packed UID");
});

fn sell_token_source_from_code(code: u8) -> SellTokenSource {
    match code % 3 {
        0 => SellTokenSource::Erc20,
        1 => SellTokenSource::External,
        _ => SellTokenSource::Internal,
    }
}

fn buy_token_destination_from_code(code: u8) -> BuyTokenDestination {
    match code % 2 {
        0 => BuyTokenDestination::Erc20,
        _ => BuyTokenDestination::Internal,
    }
}

/// Builds a bounded ASCII string from a seed byte and a length byte.
///
/// The length is clamped to a short window (at most 16 bytes) and the
/// characters map the seed through a printable-ASCII window so the
/// resulting string never violates `TypedDataDomain` expectations.
fn bounded_ascii(seed: u8, len_byte: u8) -> String {
    let len = usize::from(len_byte) % 17;
    if len == 0 {
        return String::new();
    }
    let base = b'A' + (seed % 26);
    (0..len)
        .map(|offset| {
            let next = b'A' + (((seed as usize + offset) as u8) % 26);
            char::from(if offset == 0 { base } else { next })
        })
        .collect()
}
