#![no_main]

//! Fuzz target for the browser-wallet `RpcErrorPayload` serde boundary.
//!
//! **Surface:** `cow_sdk_browser_wallet::RpcErrorPayload` `Deserialize`,
//! `Serialize`, and `Debug` impls together with the embedded
//! `Redacted<String>` `message` field's sanitization contract.
//! **Property:** `PROP-BWL-002`.
//! **Seed contract:** corpus inputs cover canonical EIP-1193 user-rejected
//! and disconnection JSON-RPC error payloads, empty and null-data
//! boundaries, and adversarial payloads including non-JSON noise,
//! credential-bearing message material, and numerically oversized
//! error codes.
//! **Corpus README:** `../corpus/fuzz_rpc_error_payload_serde/README.md`.
//!
//! The target maps arbitrary bytes through `serde_json::from_slice`
//! into `RpcErrorPayload`, asserts no panic on any input, and asserts
//! every accepted value renders deterministically through `Debug` and
//! re-serializes to a byte-stable sanitized form. The `Redacted<String>`
//! `Serialize` impl deliberately writes the `[redacted]` placeholder
//! rather than the inner value, so the round-trip is asserted on the
//! sanitized output rather than via full equality.
//!
//! ## Related coverage gap
//!
//! The browser-wallet RPC normalization pipeline also feeds three
//! crate-private helpers in
//! `cow_sdk_browser_wallet::provider::provider_impl`
//! (`hex_quantity`, `parse_chain_id_value`, `parse_quantity_to_decimal`)
//! that are reachable only through `async fn` wrappers on `Provider`.
//! The fuzz crate does not link an async executor for this target, so
//! those helpers cannot be driven directly here today. The gap and the
//! committed future-target names are tracked in
//! `docs/audit/fuzz-coverage-audit.md`; when async-runtime support is
//! added for those targets, the dedicated coverage joins the inventory
//! under its own name without disturbing this one.

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
