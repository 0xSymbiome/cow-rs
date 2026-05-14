#![no_main]

//! Fuzz target placeholder for the browser-wallet JSON-to-`DynSolValue`
//! coercion helpers.
//!
//! **Surface:** `cow_sdk_browser_wallet::provider::async_provider::{
//! json_to_dyn_value, parse_u256, parse_i256, bytes_from_json, decode_hex}`.
//! **Property:** `PROP-BWL-002`.
//! **Seed contract:** corpus inputs cover canonical contract-call JSON
//! shapes, boundary empty / null payloads, and adversarial oversized
//! integers, malformed hex, and credential-bearing strings that the
//! helpers would reject through `BrowserWalletError::MalformedResponse`.
//! **Corpus README:** `../corpus/fuzz_json_to_dyn_value/README.md`.
//!
//! The named helpers are module-private inside
//! `cow_sdk_browser_wallet::provider::async_provider`, and the next
//! public wrapper (`AsyncProvider::read_contract`) is an `async fn`. The
//! fuzz crate does not link an async executor, so the helpers cannot be
//! driven directly today. This stub keeps the harness panic-free and
//! exercises an adjacent public surface — `ContractCall` deserialization
//! from arbitrary bytes — that feeds the same coercion pipeline through
//! the public `read_contract` entry point.

use cow_sdk_core::ContractCall;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Deserialize the raw bytes as a `ContractCall` request; failures are
    // acceptable for malformed input, but no panic is allowed.
    let call = serde_json::from_slice::<ContractCall>(data);

    if let Ok(call) = call.as_ref() {
        // Round-trip the typed value through serde JSON; the documented DTO
        // must remain byte-stable for any accepted input.
        let serialized = serde_json::to_vec(call).expect("ContractCall must serialize cleanly");
        let reparsed: ContractCall = serde_json::from_slice(&serialized)
            .expect("re-serialized ContractCall must remain parseable");
        assert_eq!(
            call, &reparsed,
            "ContractCall serde round-trip must preserve typed value",
        );

        // The embedded `abi_json` and `args_json` strings would feed the
        // crate-private `json_to_dyn_value` coercion path through
        // `AsyncProvider::read_contract`. Round-trip parsing them as
        // free-form JSON exercises the same upstream deserializer the
        // coercion path depends on, again without panicking.
        let _ = serde_json::from_str::<serde_json::Value>(&call.abi_json);
        let _ = serde_json::from_str::<serde_json::Value>(&call.args_json);
    }
});
