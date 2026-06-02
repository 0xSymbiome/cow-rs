//! Composable selector parity contract test: pin the canonical composable-cow
//! custom-error selectors and the muxer interface id against both the committed
//! fixture (`tests/fixtures/composable_canonical_selectors.json`) and the
//! Foundry build artifacts under `crates/contracts/abi/composable-cow/out/`, so
//! upstream method-identifier drift or `alloy::sol!` macro drift across alloy
//! major releases is caught at every CI run.

use std::path::PathBuf;

fn out_artifact(name: &str) -> serde_json::Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("abi")
        .join("composable-cow")
        .join("out")
        .join(name);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    serde_json::from_str(&text).expect("valid json")
}

fn canonical_fixture() -> serde_json::Value {
    cow_sdk_test_utils::fixtures::manifest_fixture(
        env!("CARGO_MANIFEST_DIR"),
        "tests/fixtures/composable_canonical_selectors.json",
    )
}

fn out_selector(artifact: &serde_json::Value, key: &str) -> Option<String> {
    artifact["custom_error_selectors"]
        .as_object()
        .and_then(|obj| obj.get(key))
        .and_then(|v| v.as_str())
        .map(|s| format!("0x{s}"))
}

fn lookup_selector(fixture: &serde_json::Value, name: &str) -> String {
    cow_sdk_test_utils::fixtures::row_by_name(fixture, "custom_errors", name)["selector"]
        .as_str()
        .expect("selector must be a string")
        .to_string()
}

#[test]
fn composable_cow_custom_error_selectors_match_foundry_artifact() {
    let artifact = out_artifact("ComposableCoW.json");
    let fixture = canonical_fixture();
    let expected = [
        "ProofNotAuthed",
        "SingleOrderNotAuthed",
        "SwapGuardRestricted",
        "InvalidHandler",
        "InvalidFallbackHandler",
        "InterfaceNotSupported",
        "InvalidHash",
    ];
    for name in expected {
        let signature = format!("{name}()");
        let from_out = out_selector(&artifact, name).unwrap_or_else(|| {
            panic!("ComposableCoW.json missing custom_error_selector for {name}")
        });
        let canonical = cow_sdk_test_utils::fixtures::row_by_name(
            &fixture,
            "custom_errors",
            signature.as_str(),
        );
        let from_fixture = canonical["selector"]
            .as_str()
            .unwrap_or_else(|| panic!("canonical fixture missing selector for {signature}"))
            .to_string();
        assert_eq!(
            from_out, from_fixture,
            "Foundry artifact and canonical fixture must agree on {name}: out={from_out}, fixture={from_fixture}"
        );
    }
}

#[test]
fn every_handler_artifact_pins_order_not_valid_selector() {
    let handlers = [
        "TWAP.json",
        "GoodAfterTime.json",
        "StopLoss.json",
        "TradeAboveThreshold.json",
        "PerpetualStableSwap.json",
    ];
    let expected = "0xc8fc2725";
    for handler in handlers {
        let artifact = out_artifact(handler);
        let selector = out_selector(&artifact, "OrderNotValid")
            .unwrap_or_else(|| panic!("{handler} missing OrderNotValid custom_error_selector"));
        assert_eq!(
            selector, expected,
            "{handler} must pin OrderNotValid to {expected}; got {selector}"
        );
    }
}

#[test]
fn twelve_canonical_custom_error_selectors_pinned() {
    let fixture = canonical_fixture();
    let expected = [
        ("OrderNotValid(string)", "0xc8fc2725"),
        ("PollTryNextBlock(string)", "0xd05f3065"),
        ("PollTryAtBlock(uint256,string)", "0x1fe8506e"),
        ("PollTryAtEpoch(uint256,string)", "0x7e334637"),
        ("PollNever(string)", "0x981b64cd"),
        ("ProofNotAuthed()", "0x4a821464"),
        ("SingleOrderNotAuthed()", "0x7a933234"),
        ("SwapGuardRestricted()", "0x03fc2a7e"),
        ("InvalidHandler()", "0xd8f59fa5"),
        ("InvalidFallbackHandler()", "0x79ac63cd"),
        ("InterfaceNotSupported()", "0x2c7ca6d7"),
        ("InvalidHash()", "0x0af806e0"),
    ];
    for (name, selector) in expected {
        let actual = lookup_selector(&fixture, name);
        assert_eq!(
            actual, selector,
            "canonical selector for `{name}` must be `{selector}`; got `{actual}`"
        );
    }
}

#[test]
fn extensible_fallback_handler_pins_muxer_interface_id() {
    let artifact = out_artifact("ExtensibleFallbackHandler.json");
    let muxer = artifact["interface_ids"]["SIGNATURE_VERIFIER_MUXER_INTERFACE_ID"]
        .as_str()
        .expect("ExtensibleFallbackHandler.json must pin the muxer interface id");
    assert_eq!(muxer, "62af8dc2");
}

#[test]
fn signature_verifier_muxer_interface_id_pinned() {
    let fixture = canonical_fixture();
    let interface_ids = fixture["interface_ids"]
        .as_array()
        .expect("interface_ids must be a json array");
    let muxer = interface_ids
        .iter()
        .find(|row| row["name"].as_str() == Some("SIGNATURE_VERIFIER_MUXER_INTERFACE_ID"))
        .expect("SIGNATURE_VERIFIER_MUXER_INTERFACE_ID must be present");
    assert_eq!(
        muxer["value"].as_str(),
        Some("0x62af8dc2"),
        "SIGNATURE_VERIFIER_MUXER_INTERFACE_ID must be 0x62af8dc2"
    );
}
