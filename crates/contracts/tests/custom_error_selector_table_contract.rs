//! Custom-error selector table contract test: pin the 12 canonical
//! composable custom-error selectors and the muxer interface id
//! against the canonical-selectors fixture. The fixture mirrors the
//! pinned upstream composable-cow SHA
//! `471ca59aa95da1bbf3b03e002de96449bc78e6f0`; any drift in
//! `alloy::sol`! macro semantics across alloy major releases is
//! caught here first.

use std::path::PathBuf;

fn canonical_selectors_fixture() -> serde_json::Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("composable_canonical_selectors.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    serde_json::from_str(&text).expect("valid json")
}

fn lookup_selector(fixture: &serde_json::Value, name: &str) -> String {
    fixture["custom_errors"]
        .as_array()
        .expect("custom_errors must be a json array")
        .iter()
        .find(|row| row["name"].as_str() == Some(name))
        .unwrap_or_else(|| panic!("custom error `{name}` missing from canonical fixture"))["selector"]
        .as_str()
        .expect("selector must be a string")
        .to_string()
}

#[test]
fn twelve_canonical_custom_error_selectors_pinned() {
    let fixture = canonical_selectors_fixture();
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
fn signature_verifier_muxer_interface_id_pinned() {
    let fixture = canonical_selectors_fixture();
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
