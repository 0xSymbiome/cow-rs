use cow_sdk_core::SupportedChainId;
use cow_sdk_subgraph::{
    ExternalHostPolicy, HostPolicyError, SubgraphApi, SubgraphApiBaseUrls, SubgraphError,
};

fn base_urls(url: &str) -> SubgraphApiBaseUrls {
    std::iter::once((SupportedChainId::Mainnet, Some(url.to_owned()))).collect()
}

fn rejected_host(error: SubgraphError) -> HostPolicyError {
    match error {
        SubgraphError::HostPolicy(error) => error,
        other => panic!("expected host policy error, got {other:?}"),
    }
}

#[test]
fn subgraph_builder_blocks_custom_hosts_by_default() {
    let error = SubgraphApi::builder()
        .chain(SupportedChainId::Mainnet)
        .api_key("partner-key")
        .base_urls(base_urls(
            "https://user:pass@mirror.example/subgraphs?token=secret",
        ))
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
fn subgraph_builder_accepts_explicit_allow_and_loopback_policy() {
    let allow = SubgraphApi::builder()
        .chain(SupportedChainId::Mainnet)
        .api_key("partner-key")
        .with_external_host_policy(ExternalHostPolicy::Allow(vec!["mirror.example".to_owned()]))
        .base_urls(base_urls("https://mirror.example/subgraphs"))
        .build();
    assert!(allow.is_ok());

    for url in [
        "http://127.0.0.1:39111/subgraphs",
        "http://localhost:39111/subgraphs",
        "http://[::1]:39111/subgraphs",
    ] {
        let api = SubgraphApi::builder()
            .chain(SupportedChainId::Mainnet)
            .api_key("partner-key")
            .with_external_host_policy(ExternalHostPolicy::Test)
            .base_urls(base_urls(url))
            .build();
        assert!(
            api.is_ok(),
            "loopback fixture URL should be accepted: {url}"
        );
    }
}
