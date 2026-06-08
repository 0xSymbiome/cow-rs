//! Shared loopback test-client helpers for the `cow-sdk-subgraph` integration
//! suites.
//!
//! `SubgraphApiBaseUrls` treats an absent map entry the same as `Some(None)`, so
//! a single-entry map is enough to route the chain under test. These helpers
//! centralize that loopback construction instead of repeating an every-chain
//! map in each test file.

#![allow(
    dead_code,
    reason = "not every test binary that includes this module uses every helper"
)]

use cow_sdk_core::{HttpClientPolicy, SupportedChainId};
use cow_sdk_subgraph::{ExternalHostPolicy, SubgraphApi, SubgraphApiBaseUrls};
use cow_sdk_transport_policy::{DEFAULT_SUBGRAPH_USER_AGENT, TransportPolicy};

/// Single-entry base-URL map routing mainnet at `uri`.
pub fn loopback_base_urls(uri: String) -> SubgraphApiBaseUrls {
    std::iter::once((SupportedChainId::Mainnet, Some(uri))).collect()
}

/// Mainnet subgraph client pointed at `uri` under the loopback host policy.
pub fn loopback_client(uri: String) -> SubgraphApi {
    SubgraphApi::builder()
        .chain(SupportedChainId::Mainnet)
        .api_key("FakeApiKey")
        .external_host_policy(ExternalHostPolicy::Test)
        .base_urls(loopback_base_urls(uri))
        .build()
        .expect("subgraph test client with loopback override must build")
}

/// Like [`loopback_client`] but with request timeouts disabled, for tests that
/// drive deliberately delayed responses.
pub fn loopback_client_no_timeout(uri: String) -> SubgraphApi {
    SubgraphApi::builder()
        .chain(SupportedChainId::Mainnet)
        .api_key("FakeApiKey")
        .external_host_policy(ExternalHostPolicy::Test)
        .transport_policy(
            TransportPolicy::default_subgraph().with_client_policy(
                HttpClientPolicy::new(DEFAULT_SUBGRAPH_USER_AGENT)
                    .expect("default subgraph user-agent must remain valid")
                    .without_timeout(),
            ),
        )
        .base_urls(loopback_base_urls(uri))
        .build()
        .expect("subgraph test client with loopback override must build")
}
