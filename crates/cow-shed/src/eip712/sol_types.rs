//! Canonical COW Shed EIP-712 structs declared via `alloy_sol_types::sol!`.
//!
//! The macro emits `<Call as alloy_sol_types::SolStruct>` and `<ExecuteHooks
//! as alloy_sol_types::SolStruct>` implementations whose
//! `eip712_root_type()`, `eip712_encode_type()`, `eip712_type_hash()`,
//! `eip712_hash_struct()`, and `eip712_signing_hash(&domain)` accessors are
//! the canonical source of truth for the COW Shed EIP-712 surface. The
//! `domain.rs` and `hash.rs` modules in this directory delegate to these
//! types; downstream tasks (Call collapse, signing API) reuse the same
//! generated structs.
//!
//! The macro emits the canonical EIP-712 type-string literals
//! (`"Call(address target,uint256 value,bytes callData,bool allowFailure,bool isDelegateCall)"`
//! and the composite `ExecuteHooks(...)Call(...)`) at macro expansion time
//! with no whitespace between commas and in declaration order, matching the
//! protocol specification. The byte-identity oracle in
//! `crates/cow-shed/tests/eip712_type_hash_parity_contract.rs` proves the
//! macro-emitted type hashes equal the hand-keccak of the canonical strings.

alloy_sol_types::sol! {
    /// EIP-712 `Call` struct as encoded by the COW Shed `executeHooks`
    /// signing path.
    #[derive(Debug, PartialEq, Eq)]
    struct Call {
        address target;
        uint256 value;
        bytes callData;
        bool allowFailure;
        bool isDelegateCall;
    }

    /// EIP-712 `ExecuteHooks` envelope hashed under the COW Shed per-proxy
    /// domain when signing a hook bundle.
    #[derive(Debug, PartialEq, Eq)]
    struct ExecuteHooks {
        Call[] calls;
        bytes32 nonce;
        uint256 deadline;
    }
}
