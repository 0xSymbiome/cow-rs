use std::collections::BTreeMap;

use cow_sdk_core::{
    Address, ApiContext, CoreError, CowEnv, ENVS_LIST, ProtocolOptions, SupportedChainId,
    ValidationError, default_api_base_urls, eth_flow_contract_address, settlement_contract_address,
    vault_relayer_address, wrapped_native_token,
};

fn core_fixture() -> serde_json::Value {
    serde_json::from_str(include_str!("../../../parity/fixtures/core.json"))
        .expect("core fixture must remain valid json")
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
    let options = ProtocolOptions {
        env: Some(CowEnv::Staging),
        settlement_contract_override: Some(settlement_override),
        eth_flow_contract_override: Some(eth_flow_override),
    };

    let json = serde_json::to_value(&options).unwrap();
    let object = json.as_object().unwrap();
    for field in expected_fields {
        assert!(object.contains_key(field));
    }

    let gnosis_staging = ApiContext {
        chain_id: SupportedChainId::GnosisChain,
        env: CowEnv::Staging,
        base_urls: None,
        api_key: None,
    };
    assert_eq!(
        gnosis_staging.resolved_base_url().unwrap(),
        "https://barn.api.cow.fi/xdai"
    );

    let partner = ApiContext {
        chain_id: SupportedChainId::Base,
        env: CowEnv::Prod,
        base_urls: None,
        api_key: Some("partner-key".to_owned().into()),
    };
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
    let context = ApiContext {
        chain_id: SupportedChainId::Base,
        env: CowEnv::Prod,
        base_urls: None,
        api_key: Some("partner-key".to_owned().into()),
    };

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
    let context = ApiContext {
        chain_id: SupportedChainId::Base,
        env: CowEnv::Prod,
        base_urls: None,
        api_key: Some("partner\r\nkey".to_owned().into()),
    };

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

    assert_eq!(
        settlement_contract_address(SupportedChainId::Mainnet, CowEnv::Prod),
        Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41").unwrap()
    );
    assert_eq!(
        settlement_contract_address(SupportedChainId::Mainnet, CowEnv::Staging),
        Address::new("0xf553d092b50bdcbddeD1A99aF2cA29FBE5E2CB13").unwrap()
    );
    assert_eq!(
        vault_relayer_address(SupportedChainId::Mainnet, CowEnv::Prod),
        Address::new("0xC92E8bdf79f0507f65a392b0ab4667716BFE0110").unwrap()
    );
    assert_eq!(
        eth_flow_contract_address(SupportedChainId::Mainnet, CowEnv::Prod),
        Address::new("0xba3cb449bd2b4adddbc894d8697f5170800eadec").unwrap()
    );
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
            wrapped.address.as_str().starts_with("0x"),
            "every wrapped-native address must stay 0x-prefixed for chain {chain:?}"
        );
    }

    for env in [CowEnv::Prod, CowEnv::Staging] {
        let settlement = settlement_contract_address(SupportedChainId::Mainnet, env);
        let relayer = vault_relayer_address(SupportedChainId::Mainnet, env);
        let ethflow = eth_flow_contract_address(SupportedChainId::Mainnet, env);
        assert_ne!(settlement, relayer);
        assert_ne!(settlement, ethflow);
        assert_ne!(relayer, ethflow);
    }
}
