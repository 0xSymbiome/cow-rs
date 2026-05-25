#![no_main]

//! Fuzz target for the browser-wallet `ContractCall` serde boundary.
//!
//! **Surface:** `cow_sdk_core::ContractCall` `Deserialize` and
//! `Serialize` impls plus the embedded `abi_json` and `args_json`
//! free-form JSON strings the wallet contract-read pipeline forwards
//! into the crate-private dyn-value coercion path.
//! **Property:** `PROP-BWL-002`.
//! **Seed contract:** corpus inputs cover canonical ERC20 `balanceOf`
//! and `bytes32`-returning calls, empty and empty-string boundaries,
//! and adversarial payloads including malformed `args_json`, non-JSON
//! noise, and oversized integer arguments.
//! **Corpus README:** `../corpus/fuzz_contract_call_serde/README.md`.
//!
//! The target maps arbitrary bytes through `serde_json::from_slice`
//! into `ContractCall`, asserts no panic on any input, asserts the
//! typed value survives a serde round-trip byte-identically, and
//! parses the embedded `abi_json` and `args_json` strings as free-form
//! JSON to exercise the upstream deserializer the crate-private
//! coercion path consumes.
//!
//! ## Related coverage gap
//!
//! The dyn-value coercion helpers in
//! `cow_sdk_browser_wallet::provider::async_provider`
//! (`json_to_dyn_value`, `parse_u256`, `parse_i256`, `bytes_from_json`,
//! `decode_hex`) are module-private and reachable only through the
//! `async fn` wrapper `AsyncProvider::read_contract`. The fuzz crate
//! does not link an async executor, so those helpers cannot be driven
//! directly here today. The gap and the committed future-target names
//! are tracked in `docs/audit/fuzz-coverage-audit.md`; when
//! async-runtime support is added to the fuzz crate, the dedicated
//! targets join the inventory under their own names without
//! disturbing this one.

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
