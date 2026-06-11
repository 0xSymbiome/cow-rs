//! Behaviour tests for the native Alloy provider builder.
//!
//! These tests pin every branch on `RpcAlloyProviderBuilder` that the
//! happy-path wiremock integration tests do not exercise: the `InvalidUrl`
//! rejection (and the redaction posture on rendered errors), the `timeout`
//! setter on both transport states, and the `Debug` impls on the two
//! reachable typestate forms.
//!
//! The native HTTP-client initialization failure path (`TransportInit`) is
//! intentionally not exercised: `reqwest::Client::builder().build()` cannot
//! fail under any documented configuration this builder exposes. The variant
//! remains in the public surface as a forward-compatible boundary for
//! transport configurations that may fail at construction time in future
//! releases.

#![cfg(not(target_arch = "wasm32"))]

use std::time::Duration;

use cow_sdk_alloy_provider::{RpcAlloyProviderBuilder, RpcAlloyProviderBuilderError};

// -------------------------------------------------------------------------
// InvalidUrl rejection
// -------------------------------------------------------------------------

#[test]
fn http_rejects_malformed_url_without_echoing_the_input() {
    let result = RpcAlloyProviderBuilder::new().http("not a url with spaces");
    let Err(error) = result else {
        panic!("malformed URL must be rejected");
    };

    assert!(
        matches!(error, RpcAlloyProviderBuilderError::InvalidUrl),
        "expected InvalidUrl, got {error:?}",
    );
    let display = error.to_string();
    assert!(
        !display.contains("not a url"),
        "InvalidUrl display must not echo the offending input; got {display:?}",
    );
    assert_eq!(display, "rpc url failed to parse");
}

// -------------------------------------------------------------------------
// timeout setter — both call orders work
// -------------------------------------------------------------------------

#[tokio::test]
async fn timeout_setter_works_on_both_transport_states_unset_first_or_http_first() {
    // Order 1: unset.timeout(...).http(...).build() — timeout selected before transport.
    let provider_1 = RpcAlloyProviderBuilder::new()
        .timeout(Duration::from_millis(500))
        .http("https://example.invalid/rpc")
        .expect("URL parses")
        .build()
        .expect("provider builds with timeout-first ordering");
    drop(provider_1);

    // Order 2: unset.http(...).timeout(...).build() — timeout selected after transport.
    let provider_2 = RpcAlloyProviderBuilder::new()
        .http("https://example.invalid/rpc")
        .expect("URL parses")
        .timeout(Duration::from_millis(500))
        .build()
        .expect("provider builds with http-first ordering");
    drop(provider_2);
}

// -------------------------------------------------------------------------
// Debug impls
// -------------------------------------------------------------------------

#[test]
fn unset_builder_debug_reports_unset_transport_and_no_timeout() {
    let builder = RpcAlloyProviderBuilder::default();
    let debug = format!("{builder:?}");
    assert!(
        debug.contains("RpcAlloyProviderBuilder")
            && debug.contains("transport")
            && debug.contains("unset"),
        "unset builder debug must surface unset state; got {debug:?}",
    );
    assert!(
        debug.contains("None"),
        "no timeout configured; got {debug:?}"
    );
}

#[test]
fn http_builder_debug_redacts_transport_and_surfaces_timeout() {
    let builder = RpcAlloyProviderBuilder::new()
        .http("https://example.invalid/rpc")
        .expect("URL parses")
        .timeout(Duration::from_millis(750));
    let debug = format!("{builder:?}");
    assert!(
        debug.contains("[redacted]"),
        "http builder debug must redact the transport; got {debug:?}",
    );
    assert!(
        !debug.contains("example.invalid"),
        "http builder debug must never echo the configured URL host; got {debug:?}",
    );
    assert!(
        debug.contains("750ms") || debug.contains("Duration"),
        "http builder debug must surface the configured timeout; got {debug:?}",
    );
}

// -------------------------------------------------------------------------
// Builder error Display
// -------------------------------------------------------------------------

#[test]
fn builder_error_display_renders_each_variant_safely() {
    assert_eq!(
        RpcAlloyProviderBuilderError::InvalidUrl.to_string(),
        "rpc url failed to parse",
    );

    // TransportInit forwards the redacted detail; we can construct one to
    // verify the Display format even though we cannot trip the runtime path.
    let init_err = RpcAlloyProviderBuilderError::TransportInit {
        detail: cow_sdk_core::Redacted::new("local detail".to_owned()),
    };
    let rendered = init_err.to_string();
    assert!(rendered.contains("transport stack failed to initialize"));
    assert!(rendered.contains("[redacted]"));
    assert!(
        !rendered.contains("local detail"),
        "TransportInit display must not leak detail; got {rendered:?}",
    );
}
