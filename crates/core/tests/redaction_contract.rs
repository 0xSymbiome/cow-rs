use std::collections::BTreeMap;

use cow_sdk_core::{
    ApiContext, CowEnv, REDACTED_PLACEHOLDER, RedactedOptionalUrlMap, RedactedUrlMap,
    SupportedChainId,
};

const CREDENTIAL_URL: &str = "https://user:pass@example.test/path?apiKey=secret-token";
const JWT_SHAPED_TOKEN: &str = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJjb3cifQ.signature";

fn assert_no_credential_bytes(rendered: &str) {
    for forbidden in [
        "user:pass",
        "apiKey=secret-token",
        JWT_SHAPED_TOKEN,
        "example.test",
    ] {
        assert!(
            !rendered.contains(forbidden),
            "rendered output leaked {forbidden}: {rendered}"
        );
    }
}

#[test]
fn redacted_url_map_public_representations_redact_values_and_preserve_keys() {
    let urls = RedactedUrlMap::from(BTreeMap::from([
        (1u64, CREDENTIAL_URL.to_owned()),
        (
            100u64,
            format!("https://rpc.example.invalid/v3/{JWT_SHAPED_TOKEN}"),
        ),
    ]));

    let compact_debug = format!("{urls:?}");
    let pretty_debug = format!("{urls:#?}");
    let display = urls.to_string();
    let json = serde_json::to_value(&urls).expect("redacted URL map serializes");

    assert!(compact_debug.contains(REDACTED_PLACEHOLDER));
    assert!(pretty_debug.contains(REDACTED_PLACEHOLDER));
    assert!(display.contains(REDACTED_PLACEHOLDER));
    assert_eq!(json["1"], REDACTED_PLACEHOLDER);
    assert_eq!(json["100"], REDACTED_PLACEHOLDER);

    for rendered in [compact_debug, pretty_debug, display, json.to_string()] {
        assert_no_credential_bytes(&rendered);
    }
    assert_eq!(
        urls.as_inner().get(&1).map(String::as_str),
        Some(CREDENTIAL_URL)
    );
}

#[test]
fn redacted_optional_url_map_public_representations_redact_some_values_and_keep_none() {
    let urls = RedactedOptionalUrlMap::from(BTreeMap::from([
        (SupportedChainId::Mainnet, Some(CREDENTIAL_URL.to_owned())),
        (SupportedChainId::GnosisChain, None),
    ]));

    let compact_debug = format!("{urls:?}");
    let pretty_debug = format!("{urls:#?}");
    let json = serde_json::to_value(&urls).expect("optional URL map serializes");

    assert!(compact_debug.contains(REDACTED_PLACEHOLDER));
    assert!(pretty_debug.contains(REDACTED_PLACEHOLDER));
    assert_eq!(json["1"], REDACTED_PLACEHOLDER);
    assert_eq!(json["100"], serde_json::Value::Null);

    for rendered in [compact_debug, pretty_debug, json.to_string()] {
        assert_no_credential_bytes(&rendered);
    }
    assert_eq!(
        urls.as_inner()
            .get(&SupportedChainId::Mainnet)
            .and_then(Option::as_deref),
        Some(CREDENTIAL_URL)
    );
}

#[test]
fn api_context_redacts_base_urls_in_debug_and_serialize_but_resolves_raw_url() {
    let context = ApiContext::new(SupportedChainId::Mainnet, CowEnv::Prod)
        .with_base_urls(BTreeMap::from([(1u64, CREDENTIAL_URL.to_owned())]));

    let compact_debug = format!("{context:?}");
    let pretty_debug = format!("{context:#?}");
    let json = serde_json::to_value(&context).expect("api context serializes");

    assert_eq!(
        context.resolved_base_url().expect("raw base URL resolves"),
        CREDENTIAL_URL
    );
    assert!(compact_debug.contains(REDACTED_PLACEHOLDER));
    assert!(pretty_debug.contains(REDACTED_PLACEHOLDER));
    assert_eq!(json["baseUrls"]["1"], REDACTED_PLACEHOLDER);

    for rendered in [compact_debug, pretty_debug, json.to_string()] {
        assert_no_credential_bytes(&rendered);
    }
}
