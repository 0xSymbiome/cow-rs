use std::collections::BTreeMap;

use cow_sdk_core::{
    Address, ApiContext, CoreError, CowEnv, ENVS_LIST, ExternalHostPolicy, HostPolicyError,
    ProtocolOptions, SupportedChainId, UrlParseFailureClass, ValidationError,
    canonical_orderbook_hosts, default_api_base_urls, validate_external_service_url,
    wrapped_native_token,
};

fn core_fixture() -> serde_json::Value {
    cow_sdk_test_utils::fixtures::fixture("core")
}

#[test]
fn environment_defaults_match_core_fixture() {
    let fixture = core_fixture();
    let env_case = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "core-environment-defaults")
        .unwrap();

    let expected_envs: Vec<&str> = env_case["expected"]["envs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();
    let actual_envs: Vec<&str> = ENVS_LIST.iter().map(|env| env.as_str()).collect();
    assert_eq!(actual_envs, expected_envs);

    let ctx = ApiContext::default();
    assert_eq!(u64::from(ctx.chain_id), 1);
    assert_eq!(ctx.env, CowEnv::Prod);
    assert_eq!(
        ctx.resolved_base_url().unwrap(),
        "https://api.cow.fi/mainnet"
    );
}

#[test]
fn protocol_options_and_base_url_resolution_are_chain_aware() {
    let fixture = core_fixture();
    let protocol_case = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["id"] == "core-protocol-options-shape")
        .unwrap();
    let expected_fields: Vec<&str> = protocol_case["expected"]["fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();

    let mut settlement_override = BTreeMap::new();
    settlement_override.insert(
        1u64,
        Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41").unwrap(),
    );
    let mut eth_flow_override = BTreeMap::new();
    eth_flow_override.insert(
        1u64,
        Address::new("0xba3cb449bd2b4adddbc894d8697f5170800eadec").unwrap(),
    );
    let options = ProtocolOptions::new()
        .with_env(CowEnv::Staging)
        .with_settlement_contract_override(settlement_override)
        .with_eth_flow_contract_override(eth_flow_override);

    let json = serde_json::to_value(&options).unwrap();
    let object = json.as_object().unwrap();
    for field in expected_fields {
        assert!(object.contains_key(field));
    }

    let gnosis_staging = ApiContext::new(SupportedChainId::GnosisChain, CowEnv::Staging);
    assert_eq!(
        gnosis_staging.resolved_base_url().unwrap(),
        "https://barn.api.cow.fi/xdai"
    );

    let partner = ApiContext::new(SupportedChainId::Base, CowEnv::Prod)
        .with_api_key("partner-key".to_owned().into());
    assert_eq!(
        partner.resolved_base_url().unwrap(),
        "https://partners.cow.fi/base"
    );

    let staging_urls = default_api_base_urls(CowEnv::Staging, false);
    assert_eq!(
        staging_urls.get(&100).unwrap(),
        "https://barn.api.cow.fi/xdai"
    );
}

#[test]
fn api_context_debug_and_serialize_redact_partner_api_keys() {
    let context = ApiContext::new(SupportedChainId::Base, CowEnv::Prod)
        .with_api_key("partner-key".to_owned().into());

    let debug = format!("{context:?}");
    let json = serde_json::to_value(&context).expect("api context serializes");

    assert!(debug.contains("ApiContext"));
    assert!(debug.contains("[redacted]"));
    assert!(!debug.contains("partner-key"));
    assert_eq!(json["apiKey"], serde_json::json!("[redacted]"));
    assert_eq!(json["chainId"], serde_json::json!(8453));
}

#[test]
fn invalid_partner_api_keys_fail_during_local_route_resolution() {
    let context = ApiContext::new(SupportedChainId::Base, CowEnv::Prod)
        .with_api_key("partner\r\nkey".to_owned().into());

    let error = context
        .resolved_base_url()
        .expect_err("invalid API key must fail before partner routing is selected");

    assert!(matches!(
        error,
        CoreError::Validation(ValidationError::InvalidHttpHeaderValue { field: "api_key" })
    ));
}

#[test]
fn wrapped_and_protocol_addresses_match_pinned_upstream_values() {
    let mainnet_wrapped = wrapped_native_token(SupportedChainId::Mainnet);
    assert_eq!(
        mainnet_wrapped.address,
        Address::new("0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2").unwrap()
    );
    assert_eq!(mainnet_wrapped.symbol, "WETH");

    let gnosis_wrapped = wrapped_native_token(SupportedChainId::GnosisChain);
    assert_eq!(
        gnosis_wrapped.address,
        Address::new("0xe91D153E0b41518A2Ce8Dd3D7944Fa863463a97d").unwrap()
    );
    assert_eq!(gnosis_wrapped.symbol, "WXDAI");

    // Canonical GPv2Settlement, GPv2VaultRelayer, and EthFlow address
    // assertions live in the typed registry test suite under
    // `crates/contracts/tests/registry.rs` now that the address authority
    // has moved out of the core crate into `cow_sdk_contracts::deployments::Registry`.
}

#[test]
fn protocol_constants_surface_byte_equivalent_addresses_across_every_accessor() {
    for chain in SupportedChainId::ALL {
        let wrapped = wrapped_native_token(chain);
        assert_eq!(
            wrapped.address.byte_length(),
            20,
            "every wrapped-native address must decode into 20 bytes for chain {chain:?}"
        );
        assert!(
            wrapped.address.to_hex_string().starts_with("0x"),
            "every wrapped-native address must stay 0x-prefixed for chain {chain:?}"
        );
    }

    // Cross-contract distinctness assertions now live alongside the
    // address registry at `crates/contracts/tests/registry.rs`; the
    // legacy `(SupportedChainId, CowEnv)` accessors in this crate have
    // been retired in favour of the typed `Registry` surface.
}

#[test]
fn external_host_policy_accepts_canonical_and_explicit_hosts_only() {
    let canonical = canonical_orderbook_hosts();

    validate_external_service_url(
        "https://api.cow.fi/mainnet",
        canonical,
        &ExternalHostPolicy::Default,
    )
    .unwrap();

    let blocked = validate_external_service_url(
        "https://user:pass@mirror.example/mainnet?token=secret",
        canonical,
        &ExternalHostPolicy::Default,
    )
    .unwrap_err();
    assert!(matches!(blocked, HostPolicyError::HostNotAllowed { .. }));

    let display = blocked.to_string();
    let debug = format!("{blocked:?}");
    let json = serde_json::to_string(&blocked).unwrap();
    for rendered in [display, debug, json] {
        assert!(rendered.contains("[redacted]"));
        assert!(!rendered.contains("user:pass"));
        assert!(!rendered.contains("token=secret"));
        assert!(!rendered.contains("mirror.example"));
    }

    validate_external_service_url(
        "https://mirror.example/mainnet",
        canonical,
        &ExternalHostPolicy::Allow(vec!["mirror.example".to_owned()]),
    )
    .unwrap();
    validate_external_service_url(
        "https://arbitrary.example/mainnet",
        canonical,
        &ExternalHostPolicy::AllowAny,
    )
    .unwrap();
}

#[test]
fn external_host_policy_classifies_parse_scheme_and_loopback_cases() {
    let canonical = canonical_orderbook_hosts();

    let parse_error = validate_external_service_url(
        "api.cow.fi/mainnet",
        canonical,
        &ExternalHostPolicy::Default,
    )
    .unwrap_err();
    assert!(matches!(
        parse_error,
        HostPolicyError::UnparsableUrl {
            class: UrlParseFailureClass::MalformedScheme
        }
    ));

    let port_error = validate_external_service_url(
        "https://api.cow.fi:abc/mainnet",
        canonical,
        &ExternalHostPolicy::Default,
    )
    .unwrap_err();
    assert!(matches!(
        port_error,
        HostPolicyError::UnparsableUrl {
            class: UrlParseFailureClass::InvalidPort
        }
    ));

    let scheme_error = validate_external_service_url(
        "ftp://api.cow.fi/mainnet",
        canonical,
        &ExternalHostPolicy::Default,
    )
    .unwrap_err();
    assert!(matches!(
        scheme_error,
        HostPolicyError::UnsupportedScheme { scheme: "ftp" }
    ));

    for url in [
        "http://127.0.0.1:39111/mainnet",
        "http://localhost:39111/mainnet",
        "http://[::1]:39111/mainnet",
    ] {
        validate_external_service_url(url, canonical, &ExternalHostPolicy::Test).unwrap();
    }
}
