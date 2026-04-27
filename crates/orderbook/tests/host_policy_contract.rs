use cow_sdk_orderbook::{
    CowEnv, ExternalHostPolicy, HostPolicyError, OrderBookApi, OrderbookError, SupportedChainId,
};

fn rejected_host(error: OrderbookError) -> HostPolicyError {
    match error {
        OrderbookError::HostPolicy(error) => error,
        other => panic!("expected host policy error, got {other:?}"),
    }
}

#[test]
fn orderbook_builder_blocks_custom_hosts_by_default() {
    let error = OrderBookApi::builder()
        .chain(SupportedChainId::GnosisChain)
        .environment(CowEnv::Prod)
        .base_url("https://user:pass@mirror.example/xdai?token=secret")
        .build()
        .unwrap_err();
    let error = rejected_host(error);

    assert!(matches!(error, HostPolicyError::HostNotAllowed { .. }));
    for rendered in [
        error.to_string(),
        format!("{error:?}"),
        serde_json::to_string(&error).unwrap(),
    ] {
        assert!(rendered.contains("[redacted]"));
        assert!(!rendered.contains("mirror.example"));
        assert!(!rendered.contains("user:pass"));
        assert!(!rendered.contains("token=secret"));
    }
}

#[test]
fn orderbook_builder_accepts_explicit_allow_and_loopback_policy() {
    let allow = OrderBookApi::builder()
        .chain(SupportedChainId::GnosisChain)
        .environment(CowEnv::Prod)
        .with_external_host_policy(ExternalHostPolicy::Allow(vec!["mirror.example".to_owned()]))
        .base_url("https://mirror.example/xdai")
        .build();
    assert!(allow.is_ok());

    for url in [
        "http://127.0.0.1:39111/xdai",
        "http://localhost:39111/xdai",
        "http://[::1]:39111/xdai",
    ] {
        let api = OrderBookApi::builder()
            .chain(SupportedChainId::GnosisChain)
            .environment(CowEnv::Prod)
            .with_external_host_policy(ExternalHostPolicy::Test)
            .base_url(url)
            .build();
        assert!(
            api.is_ok(),
            "loopback fixture URL should be accepted: {url}"
        );
    }
}
