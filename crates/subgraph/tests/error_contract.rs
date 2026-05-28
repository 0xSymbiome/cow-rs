//! Public error-surface contract assertions for `cow-sdk-subgraph`.

use cow_sdk_core::{SupportedChainId, TransportErrorClass};
use cow_sdk_subgraph::{
    SubgraphError, SubgraphGraphQlError, SubgraphGraphQlErrorLocation, SubgraphRequestErrorContext,
    error::classify_reqwest_error,
};

#[test]
fn transport_variant_carries_typed_class_and_sanitized_detail() {
    // The bracketed host token is not a valid IPv6 literal so the URL
    // fails at the builder layer and no real network traffic is attempted.
    // The query payload is a deterministic redaction fixture used only to
    // verify the classifier strips it before returning.
    let client = reqwest::Client::new();
    let raw_error = client
        .request(
            reqwest::Method::GET,
            "https://[invalid ipv6]/private?api_key=redaction-fixture-token",
        )
        .build()
        .expect_err("malformed URL must produce a builder-layer reqwest error");
    let (class, details) = classify_reqwest_error(raw_error);

    let error = SubgraphError::Transport {
        context: Box::new(SubgraphRequestErrorContext::new(
            u64::from(SupportedChainId::Mainnet),
            "https://subgraph.example",
            "query Totals { totals { orders } }",
            Some("Totals".to_owned()),
            None,
        )),
        class,
        details: details.into(),
    };

    let SubgraphError::Transport {
        class,
        details,
        context,
    } = &error
    else {
        panic!("expected Transport variant, got {error:?}");
    };

    assert_eq!(*class, TransportErrorClass::Builder);
    assert_eq!(context.chain_id, u64::from(SupportedChainId::Mainnet));
    assert!(
        !details.as_inner().contains("api_key")
            && !details.as_inner().contains("redaction-fixture-token")
            && !details.as_inner().contains("https://"),
        "transport details must not expose URL fragments or query payload: {details}",
    );
    let display = error.to_string();
    assert!(
        display.contains("builder"),
        "transport Display must include the typed class label: {display}",
    );
    assert!(
        display.contains(&format!("chain {}", u64::from(SupportedChainId::Mainnet))),
        "transport Display must include the chain id as plaintext diagnostic: {display}",
    );
}

#[test]
fn graphql_display_includes_error_count_singular() {
    let error = SubgraphError::GraphQl {
        context: Box::new(sample_context()),
        errors: vec![SubgraphGraphQlError::new(
            "graphql validation failed",
            Vec::new(),
        )],
    };
    let display = error.to_string();
    assert!(
        display.contains("1 error"),
        "single-error Display must read \"1 error\" (no plural s): {display}",
    );
    assert!(
        !display.contains("1 errors"),
        "single-error Display must not pluralize to \"1 errors\": {display}",
    );
}

#[test]
fn graphql_display_includes_error_count_plural() {
    let error = SubgraphError::GraphQl {
        context: Box::new(sample_context()),
        errors: vec![
            SubgraphGraphQlError::new("first failure", Vec::new()),
            SubgraphGraphQlError::new("second failure", Vec::new()),
            SubgraphGraphQlError::new("third failure", Vec::new()),
        ],
    };
    let display = error.to_string();
    assert!(
        display.contains("3 errors"),
        "multi-error Display must pluralize to \"3 errors\": {display}",
    );
}

#[test]
fn graphql_display_includes_chain_id() {
    let error = SubgraphError::GraphQl {
        context: Box::new(sample_context()),
        errors: vec![SubgraphGraphQlError::new("anything", Vec::new())],
    };
    let display = error.to_string();
    let chain_marker = format!("chain {}", u64::from(SupportedChainId::Mainnet));
    assert!(
        display.contains(&chain_marker),
        "graphql Display must surface chain id as plaintext diagnostic: {display}",
    );
}

#[test]
fn graphql_display_includes_first_location_when_present() {
    let error = SubgraphError::GraphQl {
        context: Box::new(sample_context()),
        errors: vec![
            SubgraphGraphQlError::new(
                "first failure",
                vec![SubgraphGraphQlErrorLocation::new(4, 7)],
            ),
            SubgraphGraphQlError::new(
                "second failure",
                vec![SubgraphGraphQlErrorLocation::new(99, 99)],
            ),
        ],
    };
    let display = error.to_string();
    assert!(
        display.contains("at 4:7"),
        "graphql Display must surface the first error's first source location: {display}",
    );
    assert!(
        !display.contains("99:99"),
        "graphql Display must only render the first error's first location, not later ones: {display}",
    );
}

#[test]
fn graphql_display_omits_location_when_absent() {
    let error = SubgraphError::GraphQl {
        context: Box::new(sample_context()),
        errors: vec![SubgraphGraphQlError::new(
            "no locations on this one",
            Vec::new(),
        )],
    };
    let display = error.to_string();
    assert!(
        !display.contains(" at "),
        "graphql Display must omit the location suffix when locations are empty: {display}",
    );
}

#[test]
fn graphql_display_is_single_line() {
    let error = SubgraphError::GraphQl {
        context: Box::new(sample_context()),
        errors: vec![SubgraphGraphQlError::new(
            "anything",
            vec![SubgraphGraphQlErrorLocation::new(1, 1)],
        )],
    };
    let display = error.to_string();
    assert!(
        !display.contains('\n'),
        "graphql Display must remain single-line for log formatting: {display}",
    );
}

#[test]
fn graphql_display_does_not_leak_message_content() {
    // Defence-in-depth: even with a deliberately distinctive marker stuffed
    // into the carried GraphQL error message, the Display path must keep
    // the marker behind the workspace `Redacted<T>` wrapper. The marker is
    // chosen so a substring search is unambiguous.
    let marker = "MARKER_GRAPHQL_DISPLAY_LEAK_GUARD";
    let error = SubgraphError::GraphQl {
        context: Box::new(sample_context()),
        errors: vec![SubgraphGraphQlError::new(marker, Vec::new())],
    };
    let display = error.to_string();
    assert!(
        !display.contains(marker),
        "graphql Display must not interpolate raw message content: {display}",
    );
}

#[test]
fn serialization_display_includes_body_byte_count() {
    let body_text = "x".repeat(412);
    let error = SubgraphError::Serialization {
        context: Box::new(sample_context()),
        body: body_text.into(),
        details: "expected `data` field at line 1 column 12"
            .to_owned()
            .into(),
    };
    let display = error.to_string();
    assert!(
        display.contains("body 412 bytes"),
        "serialization Display must surface body byte count as plaintext diagnostic: {display}",
    );
    assert!(
        display.contains(&format!("chain {}", u64::from(SupportedChainId::Mainnet))),
        "serialization Display must surface chain id as plaintext diagnostic: {display}",
    );
}

#[test]
fn missingdata_display_includes_chain_id() {
    let error = SubgraphError::MissingData {
        context: Box::new(sample_context()),
    };
    let display = error.to_string();
    let chain_marker = format!("chain {}", u64::from(SupportedChainId::Mainnet));
    assert!(
        display.contains(&chain_marker),
        "missing-data Display must surface chain id as plaintext diagnostic: {display}",
    );
}

#[test]
fn httpstatus_display_includes_chain_id_and_status_code() {
    let error = SubgraphError::HttpStatus {
        context: Box::new(sample_context()),
        status: 503,
        body: "<gateway html>".to_owned().into(),
    };
    let display = error.to_string();
    assert!(
        display.contains("503"),
        "http-status Display must surface the numeric status code: {display}",
    );
    assert!(
        display.contains(&format!("chain {}", u64::from(SupportedChainId::Mainnet))),
        "http-status Display must surface chain id as plaintext diagnostic: {display}",
    );
}

fn sample_context() -> SubgraphRequestErrorContext {
    SubgraphRequestErrorContext::new(
        u64::from(SupportedChainId::Mainnet),
        "https://subgraph.example",
        "query Totals { totals { orders } }",
        Some("Totals".to_owned()),
        None,
    )
}
