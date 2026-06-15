//! Native Alloy provider construction helpers.

use std::sync::Arc;

use alloy_network::Ethereum;
use alloy_provider::{DynProvider, Provider as _, ProviderBuilder};
use alloy_rpc_client::ClientBuilder;

use crate::retry::RetryConfig;

/// Builds the type-erased Alloy provider used by [`crate::RpcAlloyProvider`].
///
/// When `retry` is `None` the provider issues each request once over the given
/// HTTP client — the runtime-neutral default. When a [`RetryConfig`] is
/// supplied the JSON-RPC client is wrapped in a bounded exponential backoff
/// layer that transparently retries rate-limited requests.
pub(crate) fn build_http_provider(
    client: reqwest::Client,
    url: reqwest::Url,
    retry: Option<&RetryConfig>,
) -> Arc<DynProvider<Ethereum>> {
    let rpc_client = match retry {
        None => ClientBuilder::default().http_with_client(client, url),
        Some(config) => ClientBuilder::default()
            .layer(config.backoff_layer())
            .http_with_client(client, url),
    };
    Arc::new(ProviderBuilder::new().connect_client(rpc_client).erased())
}
