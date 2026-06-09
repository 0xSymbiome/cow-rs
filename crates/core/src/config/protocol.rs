use serde::{Deserialize, Serialize};

use crate::{
    errors::{CoreError, ValidationError},
    redaction::Redacted,
    types::ChainId,
};

use super::{
    chains::SupportedChainId,
    env::{AddressPerChain, ApiBaseUrls, CowEnv, default_api_base_urls},
    http::validate_header_value,
};

/// Protocol-wide address and environment overrides.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Explicit deployment environment override.
    pub env: Option<CowEnv>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Settlement contract overrides keyed by numeric chain id.
    pub settlement_contract_override: Option<AddressPerChain>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// `EthFlow` contract overrides keyed by numeric chain id.
    pub eth_flow_contract_override: Option<AddressPerChain>,
}

impl ProtocolOptions {
    /// Creates an empty options bundle.
    ///
    /// Callers typically attach overrides through [`ProtocolOptions::with_env`],
    /// [`ProtocolOptions::with_settlement_contract_override`], and
    /// [`ProtocolOptions::with_eth_flow_contract_override`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a copy of these options with an explicit environment override.
    #[must_use]
    pub const fn with_env(mut self, env: CowEnv) -> Self {
        self.env = Some(env);
        self
    }

    /// Returns a copy of these options with explicit settlement-contract overrides.
    #[must_use]
    pub fn with_settlement_contract_override(mut self, overrides: AddressPerChain) -> Self {
        self.settlement_contract_override = Some(overrides);
        self
    }

    /// Returns a copy of these options with explicit `EthFlow`-contract overrides.
    #[must_use]
    pub fn with_eth_flow_contract_override(mut self, overrides: AddressPerChain) -> Self {
        self.eth_flow_contract_override = Some(overrides);
        self
    }
}

/// API routing context used by transport-owning crates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiContext {
    /// Target chain id for endpoint resolution.
    pub chain_id: SupportedChainId,
    /// Target environment for endpoint resolution.
    pub env: CowEnv,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Optional explicit base URLs keyed by numeric chain id.
    pub base_urls: Option<ApiBaseUrls>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Optional partner API key that switches resolution to partner endpoints.
    pub api_key: Option<Redacted<String>>,
}

impl Default for ApiContext {
    fn default() -> Self {
        Self {
            chain_id: SupportedChainId::Mainnet,
            env: CowEnv::Prod,
            base_urls: None,
            api_key: None,
        }
    }
}

impl ApiContext {
    /// Creates a routing context for the supplied chain and environment.
    ///
    /// Every optional field defaults to `None`; callers that need to override
    /// the base-URL map or attach a partner API key can chain
    /// [`ApiContext::with_base_urls`] and [`ApiContext::with_api_key`].
    #[must_use]
    pub const fn new(chain_id: SupportedChainId, env: CowEnv) -> Self {
        Self {
            chain_id,
            env,
            base_urls: None,
            api_key: None,
        }
    }

    /// Returns a copy of this context with an explicit base-URL override map.
    #[must_use]
    pub fn with_base_urls(mut self, base_urls: impl Into<ApiBaseUrls>) -> Self {
        self.base_urls = Some(base_urls.into());
        self
    }

    /// Returns a copy of this context with an attached partner API key.
    #[must_use]
    pub fn with_api_key(mut self, api_key: Redacted<String>) -> Self {
        self.api_key = Some(api_key);
        self
    }

    /// Returns the configured partner API key after local header validation.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::InvalidHttpHeaderValue`] when the configured
    /// API key cannot be encoded as an HTTP header value.
    pub fn validated_api_key(&self) -> Result<Option<&str>, ValidationError> {
        self.api_key
            .as_ref()
            .map(|api_key| {
                let value = api_key.as_inner().as_str();
                validate_header_value(value, "api_key")?;
                Ok(value)
            })
            .transpose()
    }

    /// Resolves the effective base URL for the current chain and environment.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::MissingBaseUrl`] when the chain id has no configured
    /// URL in either the explicit override map or the default map, or
    /// [`CoreError::Validation`] when the configured partner API key is not a
    /// valid HTTP header value.
    pub fn resolved_base_url(&self) -> Result<String, CoreError> {
        let chain_id: ChainId = self.chain_id.into();
        let partner_api = self.validated_api_key()?.is_some();
        let default_urls = default_api_base_urls(self.env, partner_api);
        let base_urls = self.base_urls.as_ref().unwrap_or(&default_urls);

        base_urls
            .as_inner()
            .get(&chain_id)
            .cloned()
            .ok_or(CoreError::MissingBaseUrl {
                chain_id,
                env: self.env,
                partner_api,
            })
    }
}
