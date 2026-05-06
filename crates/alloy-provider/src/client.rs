//! Native Alloy provider construction helpers.

use std::sync::Arc;

use alloy_network::Ethereum;
use alloy_provider::{DynProvider, Provider as _, ProviderBuilder};

/// Builds the type-erased Alloy provider used by [`crate::RpcAlloyProvider`].
pub(crate) fn build_http_provider(
    client: reqwest::Client,
    url: reqwest::Url,
) -> Arc<DynProvider<Ethereum>> {
    Arc::new(ProviderBuilder::new().connect_reqwest(client, url).erased())
}
