//! Shared cross-crate EIP-712 reference vectors for the alloy signing tests.
//!
//! The canonical Anvil development key (a public test key, never a secret) and
//! the expected order signature it produces through the typed-data path.
//! Consumed by `crates/alloy/tests/eip712_reference_vectors.rs`,
//! `crates/alloy-signer/tests/eip712_reference_vectors.rs`, and
//! `crates/alloy-signer/tests/signer_contract.rs`.

/// Well-known Anvil/Hardhat account #1 **private key** (a public test key, not a secret).
pub const ANVIL_KEY_1: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";

/// Canonical EIP-712 order signature for the `upstream_signing` order vector
/// signed by [`ANVIL_KEY_1`] through the typed-data payload path.
pub const EXPECTED_ORDER_SIGNATURE: &str = "0x34bc8d9249f7f9399d1db57b96bfc3a2f935a25965fe265292142c305284c7241daf1b3049bc75da81012cf33aeac1de09ec5684bccf03afe7274262703780d01c";
