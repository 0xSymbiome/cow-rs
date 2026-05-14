//! Behaviour tests for the `Origin` provider-binding label.
//!
//! The `Origin` newtype carries the documented provider-origin label for
//! an EIP-1193 binding. Its `Debug` and `Display` impls redact the inner
//! value so accidental log capture cannot reveal the binding identity.
//! These tests pin every documented rejection branch on `Origin::new`
//! (empty, whitespace, control characters, undocumented scheme), the
//! reverse-DNS (no-scheme) acceptance path, the case-insensitive scheme
//! match, and the redaction posture on `Debug` and `Display`.

#![cfg(not(target_arch = "wasm32"))]

use cow_sdk_browser_wallet::{BrowserWalletError, Origin};

#[test]
fn origin_accepts_documented_url_schemes() {
    for value in [
        "http://example.invalid",
        "https://example.invalid",
        "test://wallet/sepolia",
        "transport://mock/1",
        // Case-insensitive scheme match:
        "HTTPS://example.invalid",
        "Test://wallet/2",
    ] {
        let origin = Origin::new(value).expect("documented scheme must be accepted");
        // Trimming is applied; the raw inner value is preserved otherwise.
        assert_eq!(origin.as_str(), value.trim());
    }
}

#[test]
fn origin_accepts_reverse_dns_label_without_scheme() {
    // Reverse-DNS identifiers (EIP-6963 wallet info `rdns` form) have no
    // scheme separator and are accepted as-is.
    for value in [
        "io.rabby",
        "com.metamask",
        "com.coinbase.wallet",
        "app.example",
    ] {
        let origin = Origin::new(value).expect("reverse-DNS label must be accepted");
        assert_eq!(origin.as_str(), value);
    }
}

#[test]
fn origin_trims_surrounding_whitespace_before_validation() {
    let origin = Origin::new("   https://example.invalid   ")
        .expect("whitespace-padded URL must be accepted after trimming");
    assert_eq!(origin.as_str(), "https://example.invalid");
}

#[test]
fn origin_rejects_empty_and_whitespace_only_inputs() {
    for value in ["", "   ", "\t\n"] {
        let error =
            Origin::new(value).expect_err("empty or whitespace-only origin must be rejected");
        match error {
            BrowserWalletError::InvalidProviderOrigin { message } => {
                let rendered = message.into_inner();
                assert!(
                    rendered.contains("must not be empty"),
                    "empty origin message must mention emptiness; got {rendered:?}",
                );
            }
            other => panic!("expected InvalidProviderOrigin, got {other:?}"),
        }
    }
}

#[test]
fn origin_rejects_control_characters_in_input() {
    for value in [
        "\u{0001}https://example.invalid",
        "https://example\u{0001}.invalid",
        "https://example\ninvalid",
    ] {
        let error = Origin::new(value).expect_err("control character in origin must be rejected");
        match error {
            BrowserWalletError::InvalidProviderOrigin { message } => {
                let rendered = message.into_inner();
                assert!(
                    rendered.contains("control characters"),
                    "control-char origin message must mention control characters; got {rendered:?}",
                );
            }
            other => panic!("expected InvalidProviderOrigin, got {other:?}"),
        }
    }
}

#[test]
fn origin_rejects_undocumented_url_schemes() {
    for value in [
        "ftp://example.invalid",
        "file:///etc/passwd",
        "javascript:alert(1)",
        "data:text/plain,x",
    ] {
        let error = Origin::new(value).expect_err("undocumented scheme must be rejected");
        match error {
            BrowserWalletError::InvalidProviderOrigin { message } => {
                let rendered = message.into_inner();
                assert!(
                    rendered.contains("scheme"),
                    "undocumented-scheme message must mention scheme; got {rendered:?}",
                );
            }
            other => panic!("expected InvalidProviderOrigin, got {other:?}"),
        }
    }
}

#[test]
fn origin_debug_and_display_redact_the_inner_value() {
    let origin = Origin::new("https://wallet.example.invalid/sepolia?token=secret-token").unwrap();

    let debug = format!("{origin:?}");
    let display = format!("{origin}");

    assert_eq!(debug, "[redacted]", "Debug must emit [redacted]");
    assert_eq!(display, "[redacted]", "Display must emit [redacted]");
    assert!(!debug.contains("secret-token"));
    assert!(!display.contains("secret-token"));
    assert!(!debug.contains("wallet.example"));
    assert!(!display.contains("wallet.example"));

    // The raw value remains accessible via `as_str` for callers that
    // explicitly opt out of redaction.
    assert_eq!(
        origin.as_str(),
        "https://wallet.example.invalid/sepolia?token=secret-token",
    );
}
