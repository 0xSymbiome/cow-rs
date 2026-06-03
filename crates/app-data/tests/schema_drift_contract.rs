//! Schema drift gate for the typed app-data metadata surface.
//!
//! Runtime validation is performed by the typed metadata structs, not by a
//! JSON-Schema validator. The reviewed upstream app-data schemas are retained
//! as fixtures so that a future upstream field rename or addition fails here —
//! at review time — instead of silently diverging from the hand-written typed
//! structs. The checks are deliberately coarse field-name probes: they flag
//! drift for a maintainer to resolve rather than re-implementing schema
//! validation.

use std::{fs, path::PathBuf};

fn read_schema(relative: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("schemas")
        .join(relative);
    fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "schema fixture {} must be readable: {error}",
            path.display()
        )
    })
}

fn assert_mentions(relative: &str, needles: &[&str], typed_surface: &str) {
    let body = read_schema(relative);
    for needle in needles {
        assert!(
            body.contains(needle),
            "{relative} no longer mentions `{needle}`; {typed_surface} may need to change",
        );
    }
}

#[test]
fn latest_root_schema_declares_the_modelled_metadata_families() {
    assert_mentions(
        "v1.14.0.json",
        &[
            "quote",
            "orderClass",
            "hooks",
            "partnerFee",
            "flashloan",
            "signer",
        ],
        "the typed metadata surface",
    );
}

#[test]
fn flashloan_schema_matches_the_typed_flashloan_hint() {
    assert_mentions(
        "flashloan/v0.2.0.json",
        &[
            "liquidityProvider",
            "protocolAdapter",
            "receiver",
            "token",
            "amount",
        ],
        "FlashloanHints",
    );
}

#[test]
fn quote_schema_matches_the_typed_quote_metadata() {
    assert_mentions("quote/v1.1.0.json", &["slippageBips"], "QuoteMetadata");
}

#[test]
fn hook_schema_matches_the_typed_hook() {
    assert_mentions(
        "hook/v0.2.0.json",
        &["target", "callData", "gasLimit"],
        "Hook",
    );
}

#[test]
fn partner_fee_schema_matches_the_typed_policy_shape() {
    assert_mentions(
        "partnerFee/v1.0.0.json",
        &[
            "oneOf",
            "recipient",
            "volumeBps",
            "surplus",
            "priceImprovement",
        ],
        "PartnerFeePolicy",
    );
}
