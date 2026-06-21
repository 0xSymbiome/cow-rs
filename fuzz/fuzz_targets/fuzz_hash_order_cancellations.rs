#![no_main]

//! Fuzz target for the EIP-712 order-cancellation digest pipeline.
//!
//! **Surface:** `cow_sdk_contracts::hash_order_cancellations` and
//! `cow_sdk_contracts::hash_order_cancellation` (singular wrapper).
//! **Property:** `PROP-CON-001`.
//! **Seed contract:** corpus inputs cover an empty cancellation vector,
//! a single canonical UID, a fully zero-padded UID, a fully `0xff`-padded
//! UID, a saturated `valid_to = u32::MAX` UID, the maximum-length
//! cancellation batch, and a duplicate-UID adversarial set that exercises
//! the bytes[] concatenation path documented for the contract.
//!
//! The target maps an `Arbitrary`-derived input into an explicit
//! `TypedDataDomain` plus an `OrderCancellations` payload whose UID
//! count is bounded to keep individual fuzzer runs deterministic. It
//! asserts:
//!
//! * `hash_order_cancellations` is panic-free on every constructible
//!   input.
//! * Determinism: hashing the same input twice yields the same digest.
//! * Singular/batch consistency: for every single-UID payload,
//!   `hash_order_cancellation(domain, uid) ==
//!   hash_order_cancellations(domain, OrderCancellations::new(vec![uid]))`.
//! * Collision check: replacing the first UID with a structurally
//!   different UID produces a different batch digest.

use cow_sdk_contracts::{
    OrderCancellations, OrderUidParams, hash_order_cancellation, hash_order_cancellations,
    pack_order_uid_params,
};
use cow_sdk_core::{Address, ChainId, OrderDigest, OrderUid, TypedDataDomain};
use libfuzzer_sys::{arbitrary::Arbitrary, fuzz_target};

const MAX_UID_COUNT: usize = 16;
const BOUNDED_NAME_WINDOW: usize = 17;

/// Arbitrary-derived input that captures every wire field a canonical
/// cancellation batch digest needs without any fallible string parsing.
#[derive(Debug, Arbitrary)]
struct FuzzInput {
    domain_name_seed: u8,
    domain_name_len: u8,
    domain_version_seed: u8,
    domain_version_len: u8,
    chain_id: u64,
    verifying_contract: [u8; 20],
    uid_count: u8,
    seed_digest: [u8; 32],
    seed_owner: [u8; 20],
    seed_valid_to: u32,
    rotation_seed: u8,
}

fuzz_target!(|input: FuzzInput| {
    let domain = TypedDataDomain::new(
        bounded_ascii(input.domain_name_seed, input.domain_name_len),
        bounded_ascii(input.domain_version_seed, input.domain_version_len),
        ChainId::from(input.chain_id),
        Address::from_bytes(input.verifying_contract),
    );

    let uid_count = usize::from(input.uid_count) % (MAX_UID_COUNT + 1);
    let uids = build_uids(
        uid_count,
        input.seed_digest,
        input.seed_owner,
        input.seed_valid_to,
        input.rotation_seed,
    );
    let cancellations = OrderCancellations::new(uids.clone());

    // Empty cancellation batches still must hash deterministically.
    let first = hash_order_cancellations(&domain, &cancellations);
    let second = hash_order_cancellations(&domain, &cancellations);
    assert_eq!(
        first, second,
        "hash_order_cancellations must produce the same digest for identical inputs",
    );

    if let Some(first_uid) = uids.first() {
        // Singular-vs-batch consistency: the documented wrapper must hash
        // identically to a single-UID batch.
        let single_batch = OrderCancellations::new(vec![first_uid.clone()]);
        let batched = hash_order_cancellations(&domain, &single_batch);
        let single = hash_order_cancellation(&domain, first_uid);
        assert_eq!(
            single, batched,
            "hash_order_cancellation must equal a single-UID hash_order_cancellations",
        );

        // Collision check: replacing the first UID with a structurally
        // different UID must change the digest.
        let mut mutated = uids.clone();
        mutated[0] = rotate_uid(&input.seed_digest, &input.seed_owner, input.seed_valid_to);
        if &mutated[0] != first_uid {
            let mutated_batch = OrderCancellations::new(mutated);
            let mutated_digest = hash_order_cancellations(&domain, &mutated_batch);
            assert_ne!(
                first, mutated_digest,
                "two structurally distinct UID batches must hash to different digests",
            );
        }
    }
});

fn build_uids(
    count: usize,
    seed_digest: [u8; 32],
    seed_owner: [u8; 20],
    seed_valid_to: u32,
    rotation_seed: u8,
) -> Vec<OrderUid> {
    let mut uids = Vec::with_capacity(count);
    for index in 0..count {
        let mut digest = seed_digest;
        let mut owner = seed_owner;
        digest[0] = digest[0].wrapping_add(index as u8);
        owner[0] = owner[0].wrapping_add(rotation_seed.wrapping_mul(index as u8));
        let valid_to = seed_valid_to.wrapping_add(index as u32);
        uids.push(pack_order_uid_params(&OrderUidParams::new(
            OrderDigest::from_bytes(digest),
            Address::from_bytes(owner),
            valid_to,
        )));
    }
    uids
}

fn rotate_uid(seed_digest: &[u8; 32], seed_owner: &[u8; 20], seed_valid_to: u32) -> OrderUid {
    let mut digest = *seed_digest;
    digest[31] = digest[31].wrapping_add(1);
    pack_order_uid_params(&OrderUidParams::new(
        OrderDigest::from_bytes(digest),
        Address::from_bytes(*seed_owner),
        seed_valid_to.wrapping_add(1),
    ))
}

/// Builds a bounded ASCII string from a seed byte and a length byte.
///
/// The length is clamped to a short window and the characters map the
/// seed through a printable-ASCII window so the resulting string never
/// violates `TypedDataDomain` expectations.
fn bounded_ascii(seed: u8, len_byte: u8) -> String {
    let len = usize::from(len_byte) % BOUNDED_NAME_WINDOW;
    if len == 0 {
        return String::new();
    }
    (0..len)
        .map(|offset| char::from(b'A' + (((seed as usize + offset) as u8) % 26)))
        .collect()
}
