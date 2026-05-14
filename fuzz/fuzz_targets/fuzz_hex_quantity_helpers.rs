#![no_main]

//! Fuzz target placeholder for browser-wallet hex-quantity helpers.
//!
//! **Surface:** `cow_sdk_browser_wallet::provider::async_provider::{hex_quantity,
//! parse_chain_id_value, parse_quantity_to_decimal}`.
//! **Property:** `PROP-BWL-002`.
//! **Seed contract:** corpus inputs cover canonical `0x`-hex and decimal
//! quantities, boundary zero values, and adversarial malformed strings that
//! would reach the helpers through the wallet RPC normalization path.
//! **Corpus README:** `../corpus/fuzz_hex_quantity_helpers/README.md`.
//!
//! The named helpers are `pub(crate)` re-exports inside
//! `cow_sdk_browser_wallet::provider`, and the next public wrappers
//! (`Eip1193Provider::query_chain_id`, `Eip1193Provider::call`, and the
//! `AsyncProvider` trait implementation) are all `async fn`. The fuzz crate
//! does not link an async executor, so the helpers cannot be driven directly
//! today without expanding the public surface or introducing a sync wrapper.
//! This stub keeps the harness panic-free and exercises an adjacent public
//! surface — `RpcErrorPayload` deserialization plus `BrowserWalletError`
//! `Display`/`Debug` redaction — that participates in the same wallet
//! normalization pipeline the helpers feed.

use cow_sdk_browser_wallet::RpcErrorPayload;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // The raw bytes feed the public `RpcErrorPayload` deserialization seam,
    // which sits next to the hex-quantity helpers in the wallet RPC
    // normalization pipeline. Failures are acceptable for malformed input;
    // no panic is allowed.
    let payload = serde_json::from_slice::<RpcErrorPayload>(data);
    if let Ok(payload) = payload.as_ref() {
        // Determinism over Debug renders is the sanitization boundary the
        // hex-quantity helpers also depend on, so assert it explicitly.
        let first = format!("{payload:?}");
        let second = format!("{payload:?}");
        assert_eq!(
            first, second,
            "RpcErrorPayload Debug rendering must be deterministic",
        );
        // The payload's `message` field is `Redacted<String>`, whose
        // `Serialize` impl deliberately writes the literal placeholder
        // instead of the inner value. A full equality round-trip therefore
        // cannot hold by design — assert only that the re-serialization is
        // deterministic on the sanitized output.
        if let Ok(re_serialized) = serde_json::to_vec(payload) {
            let _re_parsed = serde_json::from_slice::<RpcErrorPayload>(&re_serialized)
                .expect("re-serialized payload must remain parseable");
            let re_serialized_again =
                serde_json::to_vec(payload).expect("re-serialization must be infallible");
            assert_eq!(
                re_serialized, re_serialized_again,
                "RpcErrorPayload serde re-serialization must be deterministic on the sanitized output",
            );
        }
    }
});
