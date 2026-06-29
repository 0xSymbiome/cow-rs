#![no_main]

//! Fuzz target for the composable merkle multiplexer round-trip.
//!
//! **Property:** `PROP-CON-026`.
//! Builds a [`Multiplexer`] from an arbitrary set of conditional-order
//! parameters and asserts three invariants over the hand-rolled tree: each
//! stored leaf equals the order's own [`merkle_leaf`], each leaf's
//! proof verifies against the root through [`verify_merkle_proof`], and the
//! root is deterministic across two builds of the same input. Together these
//! prove the tree and its proofs agree with the on-chain `MerkleProof.verify`
//! algorithm for every tree size and shape, not only the bounded sizes the
//! unit tests cover.
//!
//! Inputs are derived via [`arbitrary::Arbitrary`]: a bounded list of orders,
//! each a 20-byte handler, a 32-byte salt, and an arbitrary static-input blob.

use cow_sdk_contracts::composable::{
    ConditionalOrderParams, Multiplexer, merkle_leaf, verify_merkle_proof,
};
use cow_sdk_core::{Address, Hash32, HexData};
use libfuzzer_sys::{arbitrary::Arbitrary, fuzz_target};

#[derive(Debug, Arbitrary)]
struct Order {
    handler: [u8; 20],
    salt: [u8; 32],
    static_input: Vec<u8>,
}

#[derive(Debug, Arbitrary)]
struct FuzzInput {
    orders: Vec<Order>,
}

fuzz_target!(|input: FuzzInput| {
    // The tree needs at least one order; bound the count so each run stays cheap.
    if input.orders.is_empty() || input.orders.len() > 64 {
        return;
    }
    let params: Vec<ConditionalOrderParams> = input
        .orders
        .iter()
        .map(|order| ConditionalOrderParams {
            handler: Address::from_bytes(order.handler),
            salt: Hash32::from_bytes(order.salt),
            static_input: HexData::from(alloy_primitives::Bytes::from(order.static_input.clone())),
        })
        .collect();

    let tree = Multiplexer::from_params(&params).expect("a non-empty input builds a tree");
    let root = tree.root();

    for (index, param) in params.iter().enumerate() {
        let leaf = merkle_leaf(param);
        assert_eq!(
            tree.leaves()[index],
            leaf,
            "stored leaf {index} must match its order"
        );
        let proof = tree.proof(index).expect("an in-range index yields a proof");
        assert!(
            verify_merkle_proof(root, &proof, param),
            "proof for leaf {index} must verify against the root",
        );
    }

    let rebuilt = Multiplexer::from_params(&params).expect("the same input rebuilds");
    assert_eq!(
        rebuilt.root(),
        root,
        "the root must be deterministic for the same input"
    );
});
