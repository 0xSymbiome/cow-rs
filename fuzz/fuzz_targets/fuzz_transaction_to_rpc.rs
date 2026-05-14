#![no_main]

//! Fuzz target placeholder for the browser-wallet transaction-request to
//! JSON-RPC normalization helper.
//!
//! **Surface:**
//! `cow_sdk_browser_wallet::provider::async_provider::transaction_to_rpc`.
//! **Property:** `PROP-BWL-002`.
//! **Seed contract:** corpus inputs cover canonical sender/receiver transaction
//! shapes, boundary missing-field payloads, and adversarial oversized value
//! and data fields that would feed the helper through the wallet
//! `eth_sendTransaction` and `eth_call` paths.
//! **Corpus README:** `../corpus/fuzz_transaction_to_rpc/README.md`.
//!
//! The named helper is `pub(crate)` inside
//! `cow_sdk_browser_wallet::provider::async_provider`, and the next
//! public wrapper (`AsyncProvider::call` /
//! `AsyncSigningProvider::send_transaction`) is `async fn`. The fuzz
//! crate does not link an async executor, so the helper cannot be driven
//! directly today. This stub keeps the harness panic-free and exercises
//! an adjacent public surface — `TransactionRequest` deserialization
//! from arbitrary bytes — that feeds the same normalization seam.

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
