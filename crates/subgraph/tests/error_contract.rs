//! Public error-surface contract assertions for `cow-sdk-subgraph`.

use cow_sdk_core::{SupportedChainId, TransportErrorClass};
use cow_sdk_subgraph::{SubgraphError, SubgraphRequestErrorContext, error::classify_reqwest_error};

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
    assert!(
        error.to_string().contains("builder"),
        "transport Display must include the typed class label",
    );
}
