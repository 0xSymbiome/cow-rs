//! Browser-wallet discovery, session, and typed chain-management entrypoints.
//!
//! This module keeps injected-wallet behavior explicit. Discovery is bounded, multi-wallet
//! selection is visible, and typed add-chain or switch-chain helpers do not imply universal wallet
//! support across browser extensions or broaden the crate into a raw JS bridge.

use std::{cell::RefCell, fmt, rc::Rc};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use cow_sdk_core::{Address, ChainId, Redacted, SupportedChainId};

use crate::{
    BrowserWalletError, Eip1193Provider, Eip1193ProviderBuilder, Eip1193Signer, Eip1193Transport,
    EventLog, Origin, WalletSession, provider::hex_quantity,
};

/// Source used to discover one injected wallet candidate.
///
/// The enum is `#[non_exhaustive]` so new browser discovery channels can land
/// additively without breaking downstream consumers. In-crate matches may stay
/// exhaustive; external matches must include a wildcard arm.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InjectedWalletDiscoverySource {
    /// Candidate discovered through the EIP-6963 provider-announcement flow.
    Eip6963,
    #[default]
    /// Candidate discovered through direct `window.ethereum` lookup.
    LegacyWindowEthereum,
}

/// Options that bound injected-wallet discovery behavior.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InjectedWalletDetectionOptions {
    /// Maximum wait time, in milliseconds, for EIP-6963 announcements.
    pub timeout_ms: u32,
}

impl InjectedWalletDetectionOptions {
    /// Default bounded wait time, in milliseconds, for injected-wallet discovery.
    pub const DEFAULT_TIMEOUT_MS: u32 = 500;

    /// Creates a new injected-wallet discovery configuration.
    #[must_use]
    pub const fn new(timeout_ms: u32) -> Self {
        Self { timeout_ms }
    }

    /// Returns the configured discovery timeout in milliseconds.
    #[must_use]
    pub const fn timeout_ms(self) -> u32 {
        self.timeout_ms
    }
}

impl Default for InjectedWalletDetectionOptions {
    fn default() -> Self {
        Self::new(Self::DEFAULT_TIMEOUT_MS)
    }
}

/// Metadata describing one discovered injected wallet candidate.
#[non_exhaustive]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InjectedWalletInfo {
    /// Human-readable provider label.
    pub provider_label: String,
    #[serde(default)]
    /// Discovery source used for this provider.
    pub discovery_source: InjectedWalletDiscoverySource,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Provider UUID reported by EIP-6963, when present.
    pub provider_uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Provider reverse-DNS identifier, when present.
    pub provider_rdns: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Provider icon URL or data URI, when present.
    pub provider_icon: Option<String>,
    /// Whether the provider advertises `MetaMask` compatibility flags.
    pub is_meta_mask: bool,
    /// Whether the provider advertises Coinbase Wallet compatibility flags.
    pub is_coinbase_wallet: bool,
    /// Whether the provider advertises Rabby compatibility flags.
    pub is_rabby: bool,
}

impl InjectedWalletInfo {
    /// Creates injected-wallet metadata from the current public field set.
    #[must_use]
    #[allow(
        clippy::too_many_arguments,
        reason = "the constructor intentionally mirrors the public field set so downstream callers can stop using struct literals"
    )]
    pub fn new(
        provider_label: impl Into<String>,
        discovery_source: InjectedWalletDiscoverySource,
        provider_uuid: Option<String>,
        provider_rdns: Option<String>,
        provider_icon: Option<String>,
        is_meta_mask: bool,
        is_coinbase_wallet: bool,
        is_rabby: bool,
    ) -> Self {
        Self {
            provider_label: provider_label.into(),
            discovery_source,
            provider_uuid,
            provider_rdns,
            provider_icon,
            is_meta_mask,
            is_coinbase_wallet,
            is_rabby,
        }
    }
}

/// Native-currency metadata for typed add-chain requests.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletNativeCurrency {
    /// Native currency name.
    pub name: String,
    /// Native currency symbol.
    pub symbol: String,
    /// Native currency decimals.
    pub decimals: u8,
}

impl WalletNativeCurrency {
    /// Creates validated native-currency metadata for `wallet_addEthereumChain`.
    ///
    /// # Errors
    ///
    /// Returns an error when the name or symbol is empty after trimming.
    pub fn new(
        name: impl Into<String>,
        symbol: impl Into<String>,
        decimals: u8,
    ) -> Result<Self, BrowserWalletError> {
        let name = name.into();
        let symbol = symbol.into();
        Ok(Self {
            name: validate_wallet_text(&name, "native currency name", None)?,
            symbol: validate_wallet_text(&symbol, "native currency symbol", None)?,
            decimals,
        })
    }
}

/// Typed chain parameters for `wallet_addEthereumChain`.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletChainParameters {
    /// Target supported chain id.
    pub chain_id: SupportedChainId,
    /// Human-readable chain name.
    pub chain_name: String,
    /// Native-currency metadata for the chain.
    pub native_currency: WalletNativeCurrency,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// RPC URLs supplied to the wallet.
    pub rpc_urls: Vec<Redacted<String>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Block explorer URLs supplied to the wallet.
    pub block_explorer_urls: Vec<Redacted<String>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Icon URLs supplied to the wallet.
    pub icon_urls: Vec<Redacted<String>>,
}

impl WalletChainParameters {
    /// Creates validated chain parameters with no RPC or explorer URLs yet attached.
    ///
    /// # Errors
    ///
    /// Returns an error when the supplied chain name is empty after trimming.
    pub fn new(
        chain_id: SupportedChainId,
        chain_name: impl Into<String>,
        native_currency: WalletNativeCurrency,
    ) -> Result<Self, BrowserWalletError> {
        let chain_name = chain_name.into();
        Ok(Self {
            chain_id,
            chain_name: validate_wallet_text(&chain_name, "chain name", Some(u64::from(chain_id)))?,
            native_currency,
            rpc_urls: Vec::new(),
            block_explorer_urls: Vec::new(),
            icon_urls: Vec::new(),
        })
    }

    /// Returns the built-in metadata for one supported chain.
    ///
    /// # Panics
    ///
    /// Panics only if the crate's built-in chain metadata stops satisfying the
    /// same validation rules enforced for user-supplied chain parameters.
    #[must_use]
    pub fn for_supported_chain(chain_id: SupportedChainId) -> Self {
        let (chain_name, native_currency) = known_chain_metadata(chain_id);
        Self::new(chain_id, chain_name, native_currency)
            .expect("built-in chain metadata must stay valid")
    }

    /// Adds one validated RPC URL to the chain parameters.
    ///
    /// # Errors
    ///
    /// Returns an error when the URL is empty or does not use `http` or `https`.
    pub fn try_with_rpc_url(
        mut self,
        rpc_url: impl Into<String>,
    ) -> Result<Self, BrowserWalletError> {
        let rpc_url = rpc_url.into();
        self.rpc_urls.push(Redacted::new(validate_wallet_url(
            &rpc_url,
            "RPC URL",
            u64::from(self.chain_id),
        )?));
        Ok(self)
    }

    /// Adds one validated block-explorer URL to the chain parameters.
    ///
    /// # Errors
    ///
    /// Returns an error when the URL is empty or does not use `http` or `https`.
    pub fn try_with_block_explorer_url(
        mut self,
        block_explorer_url: impl Into<String>,
    ) -> Result<Self, BrowserWalletError> {
        let block_explorer_url = block_explorer_url.into();
        self.block_explorer_urls
            .push(Redacted::new(validate_wallet_url(
                &block_explorer_url,
                "block explorer URL",
                u64::from(self.chain_id),
            )?));
        Ok(self)
    }

    /// Adds one validated icon URL to the chain parameters.
    ///
    /// # Errors
    ///
    /// Returns an error when the URL is empty or does not use `http` or `https`.
    pub fn try_with_icon_url(
        mut self,
        icon_url: impl Into<String>,
    ) -> Result<Self, BrowserWalletError> {
        let icon_url = icon_url.into();
        self.icon_urls.push(Redacted::new(validate_wallet_url(
            &icon_url,
            "icon URL",
            u64::from(self.chain_id),
        )?));
        Ok(self)
    }

    /// Validates the typed chain parameters before any wallet request is attempted.
    ///
    /// # Errors
    ///
    /// Returns an error when required fields are empty or when no RPC URL is configured.
    pub fn validate(&self) -> Result<(), BrowserWalletError> {
        let chain_id = u64::from(self.chain_id);
        let _ = validate_wallet_text(&self.chain_name, "chain name", Some(chain_id))?;
        let _ = validate_wallet_text(
            &self.native_currency.name,
            "native currency name",
            Some(chain_id),
        )?;
        let _ = validate_wallet_text(
            &self.native_currency.symbol,
            "native currency symbol",
            Some(chain_id),
        )?;
        if self.rpc_urls.is_empty() {
            return Err(BrowserWalletError::invalid_chain_configuration(
                chain_id,
                "wallet add-chain requires at least one RPC URL",
            ));
        }
        for url in &self.rpc_urls {
            let _ = validate_wallet_url(url.as_inner(), "RPC URL", chain_id)?;
        }
        for url in &self.block_explorer_urls {
            let _ = validate_wallet_url(url.as_inner(), "block explorer URL", chain_id)?;
        }
        for url in &self.icon_urls {
            let _ = validate_wallet_url(url.as_inner(), "icon URL", chain_id)?;
        }
        Ok(())
    }

    pub(crate) fn for_wallet_payload(
        &self,
    ) -> Result<WalletChainParametersPayload<'_>, BrowserWalletError> {
        self.validate()?;
        Ok(WalletChainParametersPayload {
            chain_id: hex_quantity(&u64::from(self.chain_id).to_string())?,
            chain_name: &self.chain_name,
            native_currency: &self.native_currency,
            rpc_urls: self
                .rpc_urls
                .iter()
                .map(|url| url.as_inner().as_str())
                .collect(),
            block_explorer_urls: self
                .block_explorer_urls
                .iter()
                .map(|url| url.as_inner().as_str())
                .collect(),
            icon_urls: self
                .icon_urls
                .iter()
                .map(|url| url.as_inner().as_str())
                .collect(),
        })
    }

    fn rpc_payload(&self) -> Result<Value, BrowserWalletError> {
        serde_json::to_value(self.for_wallet_payload()?)
            .map_err(|error| BrowserWalletError::serialization(error.to_string()))
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WalletChainParametersPayload<'a> {
    chain_id: String,
    chain_name: &'a str,
    native_currency: &'a WalletNativeCurrency,
    rpc_urls: Vec<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    block_explorer_urls: Vec<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    icon_urls: Vec<&'a str>,
}

/// Result kind returned by typed chain-management helpers.
///
/// The enum is `#[non_exhaustive]` so additional wallet-side chain-management
/// outcomes can land additively without breaking downstream consumers. In-crate
/// matches may stay exhaustive; external matches must include a wildcard arm.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WalletChainChangeKind {
    /// The chain was added successfully.
    Added,
    /// The wallet switched directly to an already-known chain.
    Switched,
    /// The chain was added first and then switched to.
    AddedThenSwitched,
}

/// Result returned by typed add-chain and switch-chain helpers.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletChainChange {
    /// Chain id requested by the helper.
    pub requested_chain_id: SupportedChainId,
    /// Chain-management path taken by the helper.
    pub kind: WalletChainChangeKind,
    /// Session snapshot after the helper completed.
    pub session: WalletSession,
}

impl WalletChainChange {
    /// Creates a typed chain-management result from the current public field set.
    #[must_use]
    pub const fn new(
        requested_chain_id: SupportedChainId,
        kind: WalletChainChangeKind,
        session: WalletSession,
    ) -> Self {
        Self {
            requested_chain_id,
            kind,
            session,
        }
    }
}

#[derive(Clone)]
struct DiscoveredInjectedWallet {
    transport: Rc<dyn Eip1193Transport>,
    info: InjectedWalletInfo,
}

/// Result of one injected-wallet discovery attempt.
#[derive(Clone)]
pub struct InjectedWalletDiscovery {
    timeout_ms: u32,
    used_legacy_fallback: bool,
    wallets: Vec<DiscoveredInjectedWallet>,
}

impl fmt::Debug for InjectedWalletDiscovery {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InjectedWalletDiscovery")
            .field("timeout_ms", &self.timeout_ms)
            .field("used_legacy_fallback", &self.used_legacy_fallback)
            .field(
                "wallets",
                &self
                    .wallets
                    .iter()
                    .map(|wallet| wallet.info.clone())
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl InjectedWalletDiscovery {
    /// Returns the discovered wallet metadata in discovery order.
    #[must_use]
    pub fn wallets(&self) -> Vec<InjectedWalletInfo> {
        self.wallets
            .iter()
            .map(|wallet| wallet.info.clone())
            .collect()
    }

    /// Returns the number of discovered wallet candidates.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.wallets.len()
    }

    /// Returns `true` when discovery produced no wallet candidates.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.wallets.is_empty()
    }

    /// Returns the bounded discovery wait time, in milliseconds.
    #[must_use]
    pub const fn timeout_ms(&self) -> u32 {
        self.timeout_ms
    }

    /// Returns `true` when discovery fell back to direct `window.ethereum` lookup.
    #[must_use]
    pub const fn used_legacy_fallback(&self) -> bool {
        self.used_legacy_fallback
    }

    /// Returns `true` when explicit wallet selection is required before use.
    #[must_use]
    pub const fn requires_explicit_selection(&self) -> bool {
        self.wallets.len() > 1
    }

    /// Returns the wallet at one discovery index.
    ///
    /// # Errors
    ///
    /// Returns an error when `index` is outside the available discovery range.
    pub fn wallet_at(&self, index: usize) -> Result<BrowserWallet, BrowserWalletError> {
        let wallet = self.wallets.get(index).ok_or_else(|| {
            BrowserWalletError::discovery_selection_out_of_range(index, self.wallets.len())
        })?;
        BrowserWallet::from_parts(wallet.transport.clone(), Some(wallet.info.clone()))
    }

    /// Returns the only discovered wallet when exactly one candidate exists.
    ///
    /// # Errors
    ///
    /// Returns an error when more than one candidate was discovered and explicit selection is
    /// required.
    pub fn single_wallet(&self) -> Result<Option<BrowserWallet>, BrowserWalletError> {
        match self.wallets.len() {
            0 => Ok(None),
            1 => self.wallet_at(0).map(Some),
            candidates => Err(BrowserWalletError::discovery_selection_required(candidates)),
        }
    }

    fn from_detected_wallets(
        options: InjectedWalletDetectionOptions,
        used_legacy_fallback: bool,
        wallets: Vec<(Rc<dyn Eip1193Transport>, InjectedWalletInfo)>,
    ) -> Self {
        Self {
            timeout_ms: options.timeout_ms(),
            used_legacy_fallback,
            wallets: wallets
                .into_iter()
                .map(|(transport, info)| DiscoveredInjectedWallet { transport, info })
                .collect(),
        }
    }
}

/// Typed browser-wallet handle that owns session state, events, and provider/signer helpers.
#[derive(Debug, Clone)]
pub struct BrowserWallet {
    provider: Eip1193Provider,
    injected_info: Option<InjectedWalletInfo>,
}

impl BrowserWallet {
    /// Creates a browser-wallet handle from one typed EIP-1193 transport.
    ///
    /// This compatibility constructor is intended for deterministic test and
    /// review transports supplied directly by Rust code. Browser-injected
    /// providers should use discovery or [`Eip1193ProviderBuilder`] so the
    /// provider origin is explicit.
    ///
    /// # Panics
    ///
    /// Panics if the transport label cannot be represented as a local origin
    /// label, or if the explicitly trusted transport cannot construct a
    /// provider. Use [`Self::from_trusted_transport`] to handle construction
    /// errors explicitly.
    #[must_use]
    pub fn from_transport<T>(transport: T) -> Self
    where
        T: Eip1193Transport + 'static,
    {
        let origin = Origin::new(format!("transport:{}", transport.label()))
            .expect("transport label must produce a valid local origin label");
        Self::from_trusted_transport(transport, origin)
            .expect("explicitly trusted Rust transport must build")
    }

    /// Creates a browser-wallet handle from a non-discovered provider origin
    /// that has been reviewed by the caller.
    ///
    /// # Errors
    ///
    /// Returns [`BrowserWalletError`] when provider construction fails.
    pub fn from_trusted_transport<T>(
        transport: T,
        origin: Origin,
    ) -> Result<Self, BrowserWalletError>
    where
        T: Eip1193Transport + 'static,
    {
        Self::from_provider_builder(
            Eip1193ProviderBuilder::new(transport).with_trusted_origin(origin),
            None,
        )
    }

    /// Returns a trust-aware provider builder for custom EIP-1193 transports.
    #[must_use]
    pub fn provider_builder<T>(transport: T) -> Eip1193ProviderBuilder
    where
        T: Eip1193Transport + 'static,
    {
        Eip1193ProviderBuilder::new(transport)
    }

    /// Returns injected-wallet metadata when this wallet originated from discovery or detection.
    #[must_use]
    pub fn injected_info(&self) -> Option<InjectedWalletInfo> {
        self.injected_info.clone()
    }

    /// Returns the typed provider associated with this wallet.
    #[must_use]
    pub fn provider(&self) -> Eip1193Provider {
        self.provider.clone()
    }

    /// Returns a typed signer bound to this wallet without fixing an expected
    /// chain.
    ///
    /// Use [`Self::signer_for_chain`] when the workflow already owns an
    /// explicit chain authority and live browser-wallet actions must fail fast
    /// if the wallet session drifts onto a different chain.
    #[must_use]
    pub fn signer(&self) -> Eip1193Signer {
        Eip1193Signer::new(self.provider.clone(), None)
    }

    /// Returns a typed signer bound to one expected chain.
    ///
    /// The wallet session chain is validated before the signer is returned, and
    /// the signer revalidates that same chain before address, signature, gas,
    /// and transaction operations.
    ///
    /// # Errors
    ///
    /// Returns an error when the wallet rejects `eth_chainId`, reports a
    /// malformed chain id, or is currently connected to a different chain than
    /// `chain_id`.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?chain_id,
                method = "browser_wallet.signer_for_chain",
            ),
        ),
    )]
    pub async fn signer_for_chain(
        &self,
        chain_id: SupportedChainId,
    ) -> Result<Eip1193Signer, BrowserWalletError> {
        let _ = self.ensure_chain(chain_id).await?;
        Ok(Eip1193Signer::new(self.provider.clone(), None).with_expected_chain(chain_id))
    }

    /// Returns the current normalized wallet session snapshot.
    #[must_use]
    pub fn session(&self) -> WalletSession {
        self.provider.session()
    }

    /// Returns the currently selected account, when one is available.
    #[must_use]
    pub fn account(&self) -> Option<Address> {
        self.session().selected_account
    }

    /// Returns the currently known chain id, when one is available.
    #[must_use]
    pub fn chain_id(&self) -> Option<ChainId> {
        self.session().chain_id
    }

    /// Clears cached session state while preserving the wallet label.
    #[must_use]
    pub fn reset_session(&self) -> WalletSession {
        self.provider.reset_session()
    }

    /// Drains and returns the buffered wallet event log.
    #[must_use]
    pub fn take_events(&self) -> Vec<crate::WalletEvent> {
        self.provider.events().take()
    }

    /// Returns a cloned snapshot of the buffered wallet event log.
    #[must_use]
    pub fn events(&self) -> Vec<crate::WalletEvent> {
        self.provider.events().snapshot()
    }

    /// Requests accounts and chain id, then returns the updated session.
    ///
    /// This path may trigger wallet authorization prompts.
    ///
    /// # Errors
    ///
    /// Returns an error when the wallet rejects account or chain requests or returns malformed
    /// responses.
    pub async fn connect(&self) -> Result<WalletSession, BrowserWalletError> {
        self.provider.query_accounts(true).await?;
        self.provider.query_chain_id().await?;
        Ok(self.session())
    }

    /// Requests accounts interactively and returns the normalized account list.
    ///
    /// # Errors
    ///
    /// Returns an error when the wallet rejects the request or returns malformed accounts.
    pub async fn request_accounts(&self) -> Result<Vec<Address>, BrowserWalletError> {
        self.provider.query_accounts(true).await
    }

    /// Refreshes the cached wallet session from passive account and chain queries.
    ///
    /// # Errors
    ///
    /// Returns an error when the wallet rejects `eth_accounts` or `eth_chainId`, or when either
    /// response is malformed.
    pub async fn refresh_session(&self) -> Result<WalletSession, BrowserWalletError> {
        let _ = self.provider.query_accounts(false).await?;
        let _ = self.provider.query_chain_id().await?;
        Ok(self.session())
    }

    /// Ensures the wallet currently reports one expected chain id.
    ///
    /// # Errors
    ///
    /// Returns an error when the wallet rejects `eth_chainId`, reports a
    /// malformed chain id, or is connected to a different chain than
    /// `chain_id`.
    pub async fn ensure_chain(
        &self,
        chain_id: SupportedChainId,
    ) -> Result<WalletSession, BrowserWalletError> {
        let session_chain_id = self.provider.query_chain_id().await?;
        let expected_chain_id = u64::from(chain_id);
        if session_chain_id != expected_chain_id {
            return Err(BrowserWalletError::SessionChainMismatch {
                expected_chain_id,
                session_chain_id,
            });
        }
        Ok(self.session())
    }

    /// Switches to a supported chain and returns the refreshed session snapshot.
    ///
    /// The returned session must report the requested chain after the switch
    /// request completes.
    ///
    /// # Errors
    ///
    /// Returns an error when the wallet rejects the switch request, does not support the method,
    /// or reports that the chain has not been added.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?chain_id,
                method = "browser_wallet.switch_chain",
            ),
        ),
    )]
    pub async fn switch_chain(
        &self,
        chain_id: SupportedChainId,
    ) -> Result<WalletSession, BrowserWalletError> {
        self.switch_chain_request(chain_id).await?;
        self.refresh_session_and_ensure_chain(chain_id).await
    }

    /// Adds one typed chain configuration through `wallet_addEthereumChain`.
    ///
    /// # Errors
    ///
    /// Returns an error when the chain parameters are invalid, when the wallet rejects the add
    /// request, or when the refreshed session cannot be loaded afterwards.
    pub async fn add_chain(
        &self,
        parameters: &WalletChainParameters,
    ) -> Result<WalletChainChange, BrowserWalletError> {
        self.add_chain_request(parameters).await?;
        let session = self.refresh_session().await?;
        Ok(WalletChainChange::new(
            parameters.chain_id,
            WalletChainChangeKind::Added,
            session,
        ))
    }

    /// Switches to a chain, or adds it first when the wallet reports it is not present.
    ///
    /// Successful switch results are returned only after the refreshed session
    /// reports the requested chain.
    ///
    /// # Errors
    ///
    /// Returns an error when the switch request fails for reasons other than chain absence, when
    /// the typed add-chain request is invalid, when the wallet rejects either request, or when the
    /// refreshed session cannot be loaded afterwards.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?parameters.chain_id,
                method = "browser_wallet.switch_or_add_chain",
            ),
        ),
    )]
    pub async fn switch_or_add_chain(
        &self,
        parameters: &WalletChainParameters,
    ) -> Result<WalletChainChange, BrowserWalletError> {
        match self.switch_chain_request(parameters.chain_id).await {
            Ok(()) => {
                let session = self
                    .refresh_session_and_ensure_chain(parameters.chain_id)
                    .await?;
                Ok(WalletChainChange::new(
                    parameters.chain_id,
                    WalletChainChangeKind::Switched,
                    session,
                ))
            }
            Err(BrowserWalletError::ChainNotAdded { chain_id, .. })
                if chain_id == u64::from(parameters.chain_id) =>
            {
                self.add_chain_request(parameters).await?;
                self.switch_chain_request(parameters.chain_id).await?;
                let session = self
                    .refresh_session_and_ensure_chain(parameters.chain_id)
                    .await?;
                Ok(WalletChainChange::new(
                    parameters.chain_id,
                    WalletChainChangeKind::AddedThenSwitched,
                    session,
                ))
            }
            Err(error) => Err(error),
        }
    }

    async fn refresh_session_and_ensure_chain(
        &self,
        chain_id: SupportedChainId,
    ) -> Result<WalletSession, BrowserWalletError> {
        let _ = self.refresh_session().await?;
        self.ensure_chain(chain_id).await
    }

    async fn switch_chain_request(
        &self,
        chain_id: SupportedChainId,
    ) -> Result<(), BrowserWalletError> {
        self.provider
            .request(
                "wallet_switchEthereumChain",
                Some(json!([{ "chainId": hex_quantity(&u64::from(chain_id).to_string())? }])),
            )
            .await
            .map(|_| ())
    }

    async fn add_chain_request(
        &self,
        parameters: &WalletChainParameters,
    ) -> Result<(), BrowserWalletError> {
        self.provider
            .request(
                "wallet_addEthereumChain",
                Some(json!([parameters.rpc_payload()?])),
            )
            .await
            .map(|_| ())
    }

    #[cfg(target_arch = "wasm32")]
    /// Discovers injected wallets with the default bounded timeout.
    ///
    /// On `wasm32`, this uses EIP-6963 discovery first and falls back to direct
    /// `window.ethereum` lookup when needed.
    ///
    /// # Errors
    ///
    /// Returns an error when the browser runtime cannot perform discovery.
    pub async fn discover() -> Result<InjectedWalletDiscovery, BrowserWalletError> {
        Self::discover_with(InjectedWalletDetectionOptions::default()).await
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Discovers injected wallets with the default bounded timeout.
    ///
    /// On non-WASM targets, discovery is a no-op and returns an empty result set.
    ///
    /// # Errors
    ///
    /// This helper does not return an error on non-WASM targets.
    pub async fn discover() -> Result<InjectedWalletDiscovery, BrowserWalletError> {
        Self::discover_with(InjectedWalletDetectionOptions::default()).await
    }

    #[cfg(target_arch = "wasm32")]
    /// Discovers injected wallets with explicit options.
    ///
    /// The result preserves discovery metadata and indicates whether direct `window.ethereum`
    /// fallback was used.
    ///
    /// # Errors
    ///
    /// Returns an error when the browser runtime cannot perform discovery.
    pub async fn discover_with(
        options: InjectedWalletDetectionOptions,
    ) -> Result<InjectedWalletDiscovery, BrowserWalletError> {
        let discovery = crate::js::discover_injected_wallets(options).await?;
        let wallets = discovery
            .wallets
            .into_iter()
            .map(|wallet| {
                let transport: Rc<dyn Eip1193Transport> = Rc::new(wallet.transport);
                (transport, wallet.info)
            })
            .collect();
        Ok(InjectedWalletDiscovery::from_detected_wallets(
            options,
            discovery.used_legacy_fallback,
            wallets,
        ))
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Discovers injected wallets with explicit options.
    ///
    /// On non-WASM targets, discovery is a no-op and returns an empty result set.
    ///
    /// # Errors
    ///
    /// This helper does not return an error on non-WASM targets.
    pub async fn discover_with(
        options: InjectedWalletDetectionOptions,
    ) -> Result<InjectedWalletDiscovery, BrowserWalletError> {
        std::future::ready(()).await;
        Ok(InjectedWalletDiscovery::from_detected_wallets(
            options,
            false,
            Vec::new(),
        ))
    }

    #[cfg(target_arch = "wasm32")]
    /// Detects the legacy `window.ethereum` provider directly.
    ///
    /// This is a compatibility helper and is not the preferred multi-wallet discovery contract.
    /// Legacy direct detection does not provide EIP-6963 origin metadata, so callers that accept
    /// the result should use [`Self::detect_with_trusted_origin`] unless their runtime has been
    /// upgraded to EIP-6963 discovery.
    ///
    /// # Errors
    ///
    /// Returns an error when the browser runtime cannot read the provider binding or when the
    /// detected provider lacks EIP-6963 trust metadata.
    pub fn detect() -> Result<Option<Self>, BrowserWalletError> {
        let Some(transport) = crate::js::InjectedProviderTransport::detect_legacy()? else {
            return Ok(None);
        };
        let info = transport.info();
        Self::from_parts(Rc::new(transport), Some(info)).map(Some)
    }

    #[cfg(target_arch = "wasm32")]
    /// Detects the legacy `window.ethereum` provider with an explicitly reviewed origin.
    ///
    /// # Errors
    ///
    /// Returns an error when the browser runtime cannot read the provider binding or provider
    /// construction fails.
    pub fn detect_with_trusted_origin(origin: Origin) -> Result<Option<Self>, BrowserWalletError> {
        let Some(transport) = crate::js::InjectedProviderTransport::detect_legacy()? else {
            return Ok(None);
        };
        let info = transport.info();
        Self::from_provider_builder(
            Eip1193ProviderBuilder::from_shared(Rc::new(transport)).with_trusted_origin(origin),
            Some(info),
        )
        .map(Some)
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Detects the legacy `window.ethereum` provider directly.
    ///
    /// On non-WASM targets, this always returns `Ok(None)`.
    ///
    /// # Errors
    ///
    /// Returns an error when the runtime transport probe fails unexpectedly.
    pub const fn detect() -> Result<Option<Self>, BrowserWalletError> {
        let _ = crate::js::InjectedProviderTransport::detect_legacy();
        Ok(None)
    }

    fn from_parts(
        transport: Rc<dyn Eip1193Transport>,
        injected_info: Option<InjectedWalletInfo>,
    ) -> Result<Self, BrowserWalletError> {
        let builder = provider_builder_for_injected_info(transport, injected_info.as_ref())?;
        Self::from_provider_builder(builder, injected_info)
    }

    fn from_provider_builder(
        builder: Eip1193ProviderBuilder,
        injected_info: Option<InjectedWalletInfo>,
    ) -> Result<Self, BrowserWalletError> {
        let events = EventLog::default();
        let session = Rc::new(RefCell::new(WalletSession::new(
            false,
            None,
            Vec::new(),
            None,
            "unknown wallet".to_owned(),
        )));
        let provider = builder.build_with_session(session, events)?;
        Ok(Self {
            provider,
            injected_info,
        })
    }
}

fn provider_builder_for_injected_info(
    transport: Rc<dyn Eip1193Transport>,
    injected_info: Option<&InjectedWalletInfo>,
) -> Result<Eip1193ProviderBuilder, BrowserWalletError> {
    let mut builder = Eip1193ProviderBuilder::from_shared(transport);
    if let Some(info) = injected_info
        && info.discovery_source == InjectedWalletDiscoverySource::Eip6963
        && let Some(rdns) = &info.provider_rdns
    {
        builder = builder.with_detected_origin(Origin::new(rdns.clone())?);
    }
    Ok(builder)
}

fn validate_wallet_text(
    value: &str,
    label: &str,
    chain_id: Option<ChainId>,
) -> Result<String, BrowserWalletError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(BrowserWalletError::invalid_chain_configuration(
            chain_id.unwrap_or_default(),
            format!("{label} must not be empty"),
        ));
    }
    Ok(trimmed.to_owned())
}

fn validate_wallet_url(
    value: &str,
    label: &str,
    chain_id: ChainId,
) -> Result<String, BrowserWalletError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(BrowserWalletError::invalid_chain_configuration(
            chain_id,
            format!("{label} must not be empty"),
        ));
    }
    let lower = trimmed.to_ascii_lowercase();
    if !(lower.starts_with("https://") || lower.starts_with("http://")) {
        return Err(BrowserWalletError::invalid_chain_configuration(
            chain_id,
            format!("{label} must use an http or https URL"),
        ));
    }
    Ok(trimmed.to_owned())
}

fn known_wallet_native_currency(name: &str, symbol: &str, decimals: u8) -> WalletNativeCurrency {
    WalletNativeCurrency::new(name, symbol, decimals)
        .expect("built-in native-currency metadata must stay valid")
}

fn known_chain_metadata(chain_id: SupportedChainId) -> (&'static str, WalletNativeCurrency) {
    match chain_id {
        SupportedChainId::Mainnet => (
            "Ethereum Mainnet",
            known_wallet_native_currency("Ether", "ETH", 18),
        ),
        SupportedChainId::Bnb => (
            "BNB Smart Chain",
            known_wallet_native_currency("BNB", "BNB", 18),
        ),
        SupportedChainId::GnosisChain => (
            "Gnosis Chain",
            known_wallet_native_currency("xDAI", "xDAI", 18),
        ),
        SupportedChainId::Polygon => ("Polygon", known_wallet_native_currency("POL", "POL", 18)),
        SupportedChainId::Base => ("Base", known_wallet_native_currency("Ether", "ETH", 18)),
        SupportedChainId::Plasma => ("Plasma", known_wallet_native_currency("Plasma", "XPL", 18)),
        SupportedChainId::ArbitrumOne => (
            "Arbitrum One",
            known_wallet_native_currency("Ether", "ETH", 18),
        ),
        SupportedChainId::Avalanche => (
            "Avalanche C-Chain",
            known_wallet_native_currency("Avalanche", "AVAX", 18),
        ),
        SupportedChainId::Ink => ("Ink", known_wallet_native_currency("Ether", "ETH", 18)),
        SupportedChainId::Linea => ("Linea", known_wallet_native_currency("Ether", "ETH", 18)),
        SupportedChainId::Sepolia => ("Sepolia", known_wallet_native_currency("Ether", "ETH", 18)),
        _ => (
            "Supported CoW Chain",
            known_wallet_native_currency("Native Currency", "NATIVE", 18),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MockEip1193Transport;

    fn discovery_with(
        entries: &[(&str, InjectedWalletDiscoverySource)],
        timeout_ms: u32,
        used_legacy_fallback: bool,
    ) -> InjectedWalletDiscovery {
        let wallets = entries
            .iter()
            .map(|(label, source)| {
                let transport: Rc<dyn Eip1193Transport> =
                    Rc::new(MockEip1193Transport::sepolia().with_label(*label));
                let provider_rdns =
                    (*source == InjectedWalletDiscoverySource::Eip6963).then(|| {
                        format!(
                            "{}.wallet.test",
                            label.to_ascii_lowercase().replace(' ', "-")
                        )
                    });
                (
                    transport,
                    InjectedWalletInfo::new(
                        (*label).to_owned(),
                        *source,
                        None,
                        provider_rdns,
                        None,
                        false,
                        false,
                        false,
                    ),
                )
            })
            .collect();
        InjectedWalletDiscovery::from_detected_wallets(
            InjectedWalletDetectionOptions::new(timeout_ms),
            used_legacy_fallback,
            wallets,
        )
    }

    #[test]
    fn detection_options_default_timeout_is_bounded() {
        let options = InjectedWalletDetectionOptions::default();
        assert_eq!(
            options.timeout_ms(),
            InjectedWalletDetectionOptions::DEFAULT_TIMEOUT_MS
        );
        assert!(options.timeout_ms() > 0);
    }

    #[test]
    fn multi_wallet_discovery_requires_explicit_selection() {
        let discovery = discovery_with(
            &[
                ("MetaMask", InjectedWalletDiscoverySource::Eip6963),
                ("Rabby", InjectedWalletDiscoverySource::Eip6963),
            ],
            750,
            false,
        );

        assert!(discovery.requires_explicit_selection());
        assert_eq!(
            discovery.single_wallet().err().unwrap(),
            BrowserWalletError::DiscoverySelectionRequired { candidates: 2 }
        );
    }

    #[test]
    fn wallet_selection_is_indexed_and_preserves_discovery_metadata() {
        let discovery = discovery_with(
            &[
                ("MetaMask", InjectedWalletDiscoverySource::Eip6963),
                ("Rabby", InjectedWalletDiscoverySource::Eip6963),
            ],
            500,
            false,
        );

        let wallet = discovery.wallet_at(1).unwrap();
        let info = wallet.injected_info().unwrap();

        assert_eq!(info.provider_label, "Rabby");
        assert_eq!(
            info.discovery_source,
            InjectedWalletDiscoverySource::Eip6963
        );
        assert_eq!(wallet.session().wallet_label, "Rabby");
    }

    #[test]
    fn legacy_fallback_metadata_stays_visible() {
        let discovery = discovery_with(
            &[(
                "Injected Wallet",
                InjectedWalletDiscoverySource::LegacyWindowEthereum,
            )],
            250,
            true,
        );

        assert_eq!(discovery.timeout_ms(), 250);
        assert!(discovery.used_legacy_fallback());
        assert_eq!(
            discovery.wallets()[0].discovery_source,
            InjectedWalletDiscoverySource::LegacyWindowEthereum
        );
    }

    #[test]
    fn discovery_cardinality_state_machine_never_auto_selects_a_provider() {
        let empty = discovery_with(&[], 250, false);
        assert!(!empty.requires_explicit_selection());
        assert!(empty.single_wallet().unwrap().is_none());

        let single = discovery_with(
            &[("MetaMask", InjectedWalletDiscoverySource::Eip6963)],
            250,
            false,
        );
        assert!(!single.requires_explicit_selection());
        let single_wallet = single
            .single_wallet()
            .unwrap()
            .expect("one candidate should be returned explicitly");
        assert_eq!(single_wallet.session().wallet_label, "MetaMask");

        let many = discovery_with(
            &[
                ("MetaMask", InjectedWalletDiscoverySource::Eip6963),
                ("Rabby", InjectedWalletDiscoverySource::Eip6963),
            ],
            250,
            false,
        );
        assert!(many.requires_explicit_selection());
        assert_eq!(
            many.single_wallet().unwrap_err(),
            BrowserWalletError::DiscoverySelectionRequired { candidates: 2 }
        );
        assert_eq!(
            many.wallet_at(0).unwrap().session().wallet_label,
            "MetaMask"
        );
        assert_eq!(many.wallet_at(1).unwrap().session().wallet_label, "Rabby");
        assert_eq!(
            many.wallet_at(2).unwrap_err(),
            BrowserWalletError::DiscoverySelectionOutOfRange {
                index: 2,
                candidates: 2,
            }
        );
    }
}
