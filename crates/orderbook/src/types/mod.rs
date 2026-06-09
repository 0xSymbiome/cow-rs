//! Public wire DTOs and builder-style request models for the orderbook API.

use serde::{Deserialize, Serialize};

pub use cow_sdk_core::{
    Address, Amount, ApiBaseUrls, ApiContext, AppDataHash, BuyTokenDestination, CowEnv, ENVS_LIST,
    EVM_NATIVE_CURRENCY_ADDRESS, ExternalHostPolicy, HostPolicyError, OrderKind, OrderUid,
    QuoteAmountsAndCosts, Redacted, SellTokenSource, SupportedChainId, TransactionHash,
};

pub use self::{app_data::*, auction::*, enums::*, lists::*, order::*, prices::*, quote::*};

mod app_data;
mod auction;
mod enums;
mod lists;
mod order;
mod prices;
mod quote;

/// Partial override applied to an [`ApiContext`] when cloning an orderbook client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ApiContextOverride {
    /// Replacement chain id for endpoint resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<SupportedChainId>,
    /// Replacement deployment environment for endpoint resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<CowEnv>,
    /// Replacement explicit base URL map keyed by numeric chain id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_urls: Option<ApiBaseUrls>,
    /// Replacement partner API key used for request headers and endpoint selection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<Redacted<String>>,
}

impl ApiContextOverride {
    /// Creates an empty context override.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a copy of this override with an explicit chain id.
    #[must_use]
    pub const fn with_chain_id(mut self, chain_id: SupportedChainId) -> Self {
        self.chain_id = Some(chain_id);
        self
    }

    /// Returns a copy of this override with an explicit environment.
    #[must_use]
    pub const fn with_env(mut self, env: CowEnv) -> Self {
        self.env = Some(env);
        self
    }

    /// Returns a copy of this override with an explicit base-URL override map.
    #[must_use]
    pub fn with_base_urls(mut self, base_urls: impl Into<ApiBaseUrls>) -> Self {
        self.base_urls = Some(base_urls.into());
        self
    }

    /// Returns a copy of this override with an attached partner API key.
    #[must_use]
    pub fn with_api_key(mut self, api_key: Redacted<String>) -> Self {
        self.api_key = Some(api_key);
        self
    }
}

/// Per-environment base URL overrides applied ahead of [`ApiContext`] resolution.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EnvBaseUrlOverrides {
    /// Explicit production base URL.
    pub prod: Option<Redacted<String>>,
    /// Explicit staging base URL.
    pub staging: Option<Redacted<String>>,
}

impl EnvBaseUrlOverrides {
    /// Sets the explicit base URL for `env`.
    pub fn set(&mut self, env: CowEnv, base_url: impl Into<String>) {
        match env {
            CowEnv::Prod => self.prod = Some(Redacted::new(base_url.into())),
            CowEnv::Staging => self.staging = Some(Redacted::new(base_url.into())),
            _ => {}
        }
    }

    /// Returns the explicit base URL for `env`, if one is configured.
    #[must_use]
    pub fn get(&self, env: CowEnv) -> Option<&str> {
        match env {
            CowEnv::Prod => self
                .prod
                .as_ref()
                .map(|base_url| base_url.as_inner().as_str()),
            CowEnv::Staging => self
                .staging
                .as_ref()
                .map(|base_url| base_url.as_inner().as_str()),
            _ => None,
        }
    }
}
