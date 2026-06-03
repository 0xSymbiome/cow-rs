use cow_sdk_app_data::{IpfsConfig, IpfsFetchPolicy};
use cow_sdk_core::REDACTED_PLACEHOLDER;

const CREDENTIAL_URL: &str = "https://user:pass@example.test/ipfs?apiKey=secret-token";

#[test]
fn ipfs_config_public_debug_and_serialize_redact_configured_uris() {
    let config = IpfsConfig {
        uri: Some(CREDENTIAL_URL.to_owned().into()),
        read_uri: Some(
            "https://read.example.test/ipfs?token=secret"
                .to_owned()
                .into(),
        ),
    };

    let debug = format!("{config:#?}");
    // Non-alternate `{:?}` coverage folded in from the former
    // crates/app-data/src/types/ipfs.rs inline test: the compact Debug form
    // must also render the struct name and redact every configured URI.
    let debug_compact = format!("{config:?}");
    let json = serde_json::to_value(&config).expect("ipfs config serializes");

    assert!(debug.contains(REDACTED_PLACEHOLDER));
    assert!(debug_compact.contains("IpfsConfig"));
    assert!(debug_compact.contains(REDACTED_PLACEHOLDER));
    assert_eq!(json["uri"], REDACTED_PLACEHOLDER);
    assert_eq!(json["readUri"], REDACTED_PLACEHOLDER);

    for rendered in [debug, debug_compact, json.to_string()] {
        assert!(!rendered.contains("user:pass"));
        assert!(!rendered.contains("apiKey=secret-token"));
        assert!(!rendered.contains("token=secret"));
        assert!(!rendered.contains("example.test"));
    }
}

#[test]
fn ipfs_config_raw_uri_access_remains_explicit_for_dispatch_policies() {
    let config = IpfsConfig {
        uri: Some(CREDENTIAL_URL.to_owned().into()),
        read_uri: Some("https://read.example.test/ipfs/".to_owned().into()),
    };

    let policy = IpfsFetchPolicy::from_config(&config).expect("read URI is valid");

    assert_eq!(policy.read_base_uri(), "https://read.example.test/ipfs");
    assert_eq!(
        config.uri.as_ref().map(|uri| uri.as_inner().as_str()),
        Some(CREDENTIAL_URL)
    );
}
