use std::{collections::BTreeMap, fmt, time::Duration};

use http::HeaderValue;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{
    errors::{CoreError, ValidationError},
    types::{Address, ChainId, TokenInfo},
};

/// All supported `CoW` API environments.
pub const ENVS_LIST: [CowEnv; 2] = [CowEnv::Prod, CowEnv::Staging];
/// Sentinel address used by `CoW` Protocol to represent the native chain asset.
pub const EVM_NATIVE_CURRENCY_ADDRESS: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";
/// Default timeout applied to HTTP-backed SDK clients.
pub const DEFAULT_HTTP_TIMEOUT: Duration = Duration::from_secs(10);
/// Maximum valid-to timestamp accepted by the protocol `uint32` field.
pub const MAX_VALID_TO_EPOCH: u32 = 4_294_967_295;

const PROD_BASE_URL: &str = "https://api.cow.fi";
const STAGING_BASE_URL: &str = "https://barn.api.cow.fi";
const PARTNER_PROD_BASE_URL: &str = "https://partners.cow.fi";
const PARTNER_STAGING_BASE_URL: &str = "https://partners.barn.cow.fi";
const SETTLEMENT_CONTRACT_ADDRESS: &str = "0x9008D19f58AAbD9eD0D60971565AA8510560ab41";
const SETTLEMENT_CONTRACT_ADDRESS_STAGING: &str = "0xf553d092b50bdcbddeD1A99aF2cA29FBE5E2CB13";
const VAULT_RELAYER_ADDRESS: &str = "0xC92E8bdf79f0507f65a392b0ab4667716BFE0110";
const VAULT_RELAYER_ADDRESS_STAGING: &str = "0xC7242d167563352E2BCA4d71C043fbe542DB8FB2";
const ETH_FLOW_ADDRESS: &str = "0xba3cb449bd2b4adddbc894d8697f5170800eadec";
const ETH_FLOW_ADDRESS_STAGING: &str = "0xb37aDD6AC288BD3825a901Cba6ec65A89f31B8CC";
const TOKEN_LIST_IMAGES_PATH: &str = "https://files.cow.fi/token-lists/images";
const REDACTED_SECRET: &str = "<redacted>";

/// Supported `CoW` Protocol chain ids with explicit API configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u64)]
pub enum SupportedChainId {
    /// Ethereum mainnet.
    Mainnet = 1,
    /// BNB Smart Chain.
    Bnb = 56,
    /// Gnosis Chain.
    GnosisChain = 100,
    /// Polygon `PoS`.
    Polygon = 137,
    /// Base.
    Base = 8453,
    /// Plasma.
    Plasma = 9745,
    /// Arbitrum One.
    ArbitrumOne = 42161,
    /// Avalanche C-Chain.
    Avalanche = 43114,
    /// Ink.
    Ink = 57073,
    /// Linea.
    Linea = 59144,
    /// Ethereum Sepolia.
    Sepolia = 11_155_111,
}

impl SupportedChainId {
    /// Complete list of supported chain ids in deterministic iteration order.
    pub const ALL: [Self; 11] = [
        Self::Mainnet,
        Self::Bnb,
        Self::GnosisChain,
        Self::Polygon,
        Self::Base,
        Self::Plasma,
        Self::ArbitrumOne,
        Self::Avalanche,
        Self::Ink,
        Self::Linea,
        Self::Sepolia,
    ];

    /// Returns the path segment used by `CoW` API base URLs for this chain.
    #[must_use]
    pub fn api_path(self) -> &'static str {
        match self {
            Self::Mainnet => "mainnet",
            Self::Bnb => "bnb",
            Self::GnosisChain => "xdai",
            Self::Polygon => "polygon",
            Self::Base => "base",
            Self::Plasma => "plasma",
            Self::ArbitrumOne => "arbitrum_one",
            Self::Avalanche => "avalanche",
            Self::Ink => "ink",
            Self::Linea => "linea",
            Self::Sepolia => "sepolia",
        }
    }
}

impl TryFrom<ChainId> for SupportedChainId {
    type Error = ValidationError;

    fn try_from(value: ChainId) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Mainnet),
            56 => Ok(Self::Bnb),
            100 => Ok(Self::GnosisChain),
            137 => Ok(Self::Polygon),
            8453 => Ok(Self::Base),
            9745 => Ok(Self::Plasma),
            42161 => Ok(Self::ArbitrumOne),
            43114 => Ok(Self::Avalanche),
            57073 => Ok(Self::Ink),
            59144 => Ok(Self::Linea),
            11_155_111 => Ok(Self::Sepolia),
            chain_id => Err(ValidationError::UnsupportedChain { chain_id }),
        }
    }
}

impl From<SupportedChainId> for ChainId {
    fn from(value: SupportedChainId) -> Self {
        value as Self
    }
}

impl Serialize for SupportedChainId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64((*self).into())
    }
}

impl<'de> Deserialize<'de> for SupportedChainId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = ChainId::deserialize(deserializer)?;
        Self::try_from(value).map_err(serde::de::Error::custom)
    }
}

/// Supported `CoW` deployment environments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CowEnv {
    /// Production endpoints and deployments.
    Prod,
    /// Staging endpoints and deployments.
    Staging,
}

impl CowEnv {
    /// Returns the stable lowercase environment identifier.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Prod => "prod",
            Self::Staging => "staging",
        }
    }
}

/// Mapping from numeric chain id to API base URL.
pub type ApiBaseUrls = BTreeMap<ChainId, String>;
/// Mapping from numeric chain id to deployment address override.
pub type AddressPerChain = BTreeMap<ChainId, Address>;

/// Shared HTTP client policy used by transport-owning crates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpClientPolicy {
    timeout: Option<Duration>,
    user_agent: String,
}

impl HttpClientPolicy {
    /// Creates a policy with the default timeout and a validated user agent.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError`] if the user agent is empty or cannot be
    /// encoded as an HTTP header value.
    pub fn new(user_agent: impl Into<String>) -> Result<Self, ValidationError> {
        Self::with_timeout_and_user_agent(DEFAULT_HTTP_TIMEOUT, user_agent)
    }

    /// Creates a policy with an explicit timeout and validated user agent.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError`] if the user agent is empty or cannot be
    /// encoded as an HTTP header value.
    pub fn with_timeout_and_user_agent(
        timeout: Duration,
        user_agent: impl Into<String>,
    ) -> Result<Self, ValidationError> {
        let user_agent = validate_user_agent(user_agent.into())?;

        Ok(Self {
            timeout: Some(timeout),
            user_agent,
        })
    }

    /// Returns a copy of this policy with timeouts disabled.
    #[must_use]
    pub fn without_timeout(mut self) -> Self {
        self.timeout = None;
        self
    }

    /// Returns a copy of this policy with the supplied timeout.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Returns a copy of this policy with a newly validated user agent.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError`] if the user agent is empty or cannot be
    /// encoded as an HTTP header value.
    pub fn try_with_user_agent(
        mut self,
        user_agent: impl Into<String>,
    ) -> Result<Self, ValidationError> {
        self.user_agent = validate_user_agent(user_agent.into())?;
        Ok(self)
    }

    /// Returns the configured timeout, if one is enabled.
    #[must_use]
    pub fn timeout(&self) -> Option<Duration> {
        self.timeout
    }

    /// Returns the configured user-agent header value.
    #[must_use]
    pub fn user_agent(&self) -> &str {
        &self.user_agent
    }
}

/// Protocol-wide address and environment overrides.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// API routing context used by transport-owning crates.
#[derive(Clone, PartialEq, Eq, Deserialize)]
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
    pub api_key: Option<String>,
}

impl fmt::Debug for ApiContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApiContext")
            .field("chain_id", &self.chain_id)
            .field("env", &self.env)
            .field("base_urls", &self.base_urls)
            .field("api_key", &self.api_key.as_ref().map(|_| REDACTED_SECRET))
            .finish()
    }
}

impl Serialize for ApiContext {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ApiContext", 4)?;
        state.serialize_field("chainId", &self.chain_id)?;
        state.serialize_field("env", &self.env)?;

        if let Some(base_urls) = &self.base_urls {
            state.serialize_field("baseUrls", base_urls)?;
        }
        if self.api_key.is_some() {
            state.serialize_field("apiKey", REDACTED_SECRET)?;
        }

        state.end()
    }
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
    /// Returns the configured partner API key after local header validation.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::InvalidHttpHeaderValue`] when the configured
    /// API key cannot be encoded as an HTTP header value.
    pub fn validated_api_key(&self) -> Result<Option<&str>, ValidationError> {
        self.api_key
            .as_deref()
            .map(|api_key| {
                validate_header_value(api_key, "api_key")?;
                Ok(api_key)
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
            .get(&chain_id)
            .cloned()
            .ok_or_else(|| CoreError::MissingBaseUrl {
                chain_id,
                env: self.env.as_str().to_owned(),
                partner_api,
            })
    }
}

/// Returns the default `CoW` API base URLs for every supported chain.
#[must_use]
pub fn default_api_base_urls(env: CowEnv, partner_api: bool) -> ApiBaseUrls {
    SupportedChainId::ALL
        .into_iter()
        .map(|chain_id| {
            let base = match (env, partner_api) {
                (CowEnv::Prod, false) => PROD_BASE_URL,
                (CowEnv::Staging, false) => STAGING_BASE_URL,
                (CowEnv::Prod, true) => PARTNER_PROD_BASE_URL,
                (CowEnv::Staging, true) => PARTNER_STAGING_BASE_URL,
            };
            (chain_id.into(), format!("{base}/{}", chain_id.api_path()))
        })
        .collect()
}

/// Returns the settlement contract address for the requested environment.
#[must_use]
pub fn settlement_contract_address(_chain_id: SupportedChainId, env: CowEnv) -> Address {
    match env {
        CowEnv::Prod => address_literal(SETTLEMENT_CONTRACT_ADDRESS),
        CowEnv::Staging => address_literal(SETTLEMENT_CONTRACT_ADDRESS_STAGING),
    }
}

/// Returns the Balancer vault relayer address for the requested environment.
#[must_use]
pub fn vault_relayer_address(_chain_id: SupportedChainId, env: CowEnv) -> Address {
    match env {
        CowEnv::Prod => address_literal(VAULT_RELAYER_ADDRESS),
        CowEnv::Staging => address_literal(VAULT_RELAYER_ADDRESS_STAGING),
    }
}

/// Returns the `EthFlow` contract address for the requested environment.
#[must_use]
pub fn eth_flow_contract_address(_chain_id: SupportedChainId, env: CowEnv) -> Address {
    match env {
        CowEnv::Prod => address_literal(ETH_FLOW_ADDRESS),
        CowEnv::Staging => address_literal(ETH_FLOW_ADDRESS_STAGING),
    }
}

/// Returns wrapped-native token metadata for a supported chain.
#[must_use]
pub fn wrapped_native_token(chain_id: SupportedChainId) -> TokenInfo {
    let (address, decimals, name, symbol) = match chain_id {
        SupportedChainId::Mainnet => (
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
            18,
            "Wrapped Ether",
            "WETH",
        ),
        SupportedChainId::GnosisChain => (
            "0xe91D153E0b41518A2Ce8Dd3D7944Fa863463a97d",
            18,
            "Wrapped XDAI",
            "WXDAI",
        ),
        SupportedChainId::ArbitrumOne => (
            "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1",
            18,
            "Wrapped Ether",
            "WETH",
        ),
        SupportedChainId::Base | SupportedChainId::Ink => (
            "0x4200000000000000000000000000000000000006",
            18,
            "Wrapped Ether",
            "WETH",
        ),
        SupportedChainId::Sepolia => (
            "0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14",
            18,
            "Wrapped Ether",
            "WETH",
        ),
        SupportedChainId::Polygon => (
            "0x0d500b1d8e8ef31e21c99d1db9a6444d3adf1270",
            18,
            "Wrapped POL",
            "WPOL",
        ),
        SupportedChainId::Avalanche => (
            "0xb31f66aa3c1e785363f0875a1b74e27b85fd66c7",
            18,
            "Wrapped AVAX",
            "WAVAX",
        ),
        SupportedChainId::Bnb => (
            "0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c",
            18,
            "Wrapped BNB",
            "WBNB",
        ),
        SupportedChainId::Plasma => (
            "0x6100e367285b01f48d07953803a2d8dca5d19873",
            18,
            "Wrapped XPL",
            "WXPL",
        ),
        SupportedChainId::Linea => (
            "0xe5d7c2a44ffddf6b295a15c148167daaaf5cf34f",
            18,
            "Wrapped Ether",
            "WETH",
        ),
    };

    let address = address_literal(address);

    TokenInfo {
        chain_id: chain_id.into(),
        logo_url: Some(format!(
            "{TOKEN_LIST_IMAGES_PATH}/{}/{}/logo.png",
            ChainId::from(chain_id),
            address.normalized_key()
        )),
        address,
        decimals,
        name: name.to_owned(),
        symbol: symbol.to_owned(),
    }
}

fn address_literal(value: &str) -> Address {
    Address::new(value).expect("static address literals must remain valid")
}

fn validate_user_agent(user_agent: String) -> Result<String, ValidationError> {
    if user_agent.trim().is_empty() {
        return Err(ValidationError::EmptyField {
            field: "user_agent",
        });
    }

    validate_header_value(&user_agent, "user_agent")?;

    Ok(user_agent)
}

fn validate_header_value(value: &str, field: &'static str) -> Result<(), ValidationError> {
    HeaderValue::from_str(value).map_err(|_| ValidationError::InvalidHttpHeaderValue { field })?;
    Ok(())
}
