//! Composable selector parity contract test: assert every selector
//! the canonical fixture lists matches the corresponding Foundry
//! artifact under `crates/contracts/abi/composable-cow/out/`. Drift
//! in upstream method identifiers is caught at every CI run.

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
fn extensible_fallback_handler_pins_muxer_interface_id() {
    let artifact = out_artifact("ExtensibleFallbackHandler.json");
    let muxer = artifact["interface_ids"]["SIGNATURE_VERIFIER_MUXER_INTERFACE_ID"]
        .as_str()
        .expect("ExtensibleFallbackHandler.json must pin the muxer interface id");
    assert_eq!(muxer, "62af8dc2");
}
