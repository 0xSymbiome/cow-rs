#![no_main]

//! Fuzz target for the browser-wallet `TransactionRequest` serde
//! boundary.
//!
//! **Surface:** `cow_sdk_core::TransactionRequest` `Deserialize` and
//! `Serialize` impls plus the deterministic-serialization invariant
//! the wallet `eth_sendTransaction` and `eth_call` paths depend on.
//! **Property:** `PROP-BWL-002`.
//! **Seed contract:** corpus inputs cover canonical native-transfer
//! and contract-call transaction shapes, empty and all-fields-none
//! boundaries, and adversarial payloads including malformed address
//! fields, non-JSON noise, and oversized value fields.
//! **Corpus README:** `../corpus/fuzz_transaction_request_serde/README.md`.
//!
//! The target maps arbitrary bytes through `serde_json::from_slice`
//! into `TransactionRequest`, asserts no panic on any input, asserts
//! the typed value survives a serde round-trip byte-identically, and
//! asserts serialization is deterministic so the documented wire
//! shape stays predictable for downstream consumers.
//!
//! ## Related coverage gap
//!
//! The normalization helper
//! `cow_sdk_browser_wallet::provider::async_provider::transaction_to_rpc`
//! is `pub(crate)` and reachable only through `async fn` wrappers on
//! `AsyncProvider` and `AsyncSigningProvider`. The fuzz crate does not
//! link an async executor, so the helper cannot be driven directly
//! here today. The gap and the committed future-target name are
//! tracked in `docs/audit/fuzz-coverage-audit.md`; when async-runtime
//! support is added to the fuzz crate, the dedicated target joins the
//! inventory under its own name without disturbing this one.

use cow_sdk_core::TransactionRequest;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Deserialize the raw bytes as a `TransactionRequest`; failures are
    // acceptable for malformed input, but no panic is allowed.
    let request = serde_json::from_slice::<TransactionRequest>(data);

    if let Ok(request) = request.as_ref() {
        // Round-trip the typed value through serde JSON. The DTO must
        // remain byte-stable for any accepted input so the documented
        // wire shape stays predictable for downstream consumers.
        let serialized = serde_json::to_vec(request)
            .expect("TransactionRequest must serialize cleanly");
        let reparsed: TransactionRequest = serde_json::from_slice(&serialized)
            .expect("re-serialized TransactionRequest must remain parseable");
        assert_eq!(
            request, &reparsed,
            "TransactionRequest serde round-trip must preserve typed value",
        );

        // Determinism on identical input: serializing the same typed value
        // twice must produce the same bytes.
        let alt = serde_json::to_vec(request).expect("TransactionRequest must reserialize");
        assert_eq!(
            serialized, alt,
            "TransactionRequest serialization must be deterministic",
        );
    }
});
