//! Browser-wallet discovery, session, and typed chain-management entrypoints.
//!
//! This module keeps injected-wallet behavior explicit. Discovery is bounded, multi-wallet
//! selection is visible, and typed add-chain or switch-chain helpers do not imply universal wallet
//! support across browser extensions or broaden the crate into a raw JS bridge.

mod chain;
mod chain_mgmt;
mod detect;
mod discovery;

pub use self::{
    chain::{
        WalletChainChange, WalletChainChangeKind, WalletChainParameters, WalletNativeCurrency,
    },
    discovery::{
        InjectedWalletDetectionOptions, InjectedWalletDiscovery, InjectedWalletDiscoverySource,
        InjectedWalletInfo,
    },
};

use std::{cell::RefCell, rc::Rc};

use cow_sdk_core::{Address, ChainId, SupportedChainId};

use crate::{
    BrowserWalletError, Eip1193Provider, Eip1193ProviderBuilder, Eip1193Signer, Eip1193Transport,
    EventLog, Origin, WalletSession,
};

/// Typed browser-wallet handle that owns session state, events, and provider/signer helpers.
#[derive(Debug, Clone)]
pub struct BrowserWallet {
    provider: Eip1193Provider,
    injected_info: Option<InjectedWalletInfo>,
}

impl BrowserWallet {
    /// Creates a browser-wallet handle from one typed EIP-1193 transport and
    /// panics if the trusted local-origin wrapper cannot be built.
    ///
    /// This compatibility constructor is intended for deterministic test and
    /// review transports supplied directly by Rust code. Prefer
    /// [`Self::from_trusted_transport`] when construction errors should be
    /// handled explicitly.
    ///
    /// # Panics
    ///
    /// Panics if the transport label cannot be represented as a local origin
    /// label, or if the explicitly trusted transport cannot construct a
    /// provider. Use [`Self::from_trusted_transport`] to handle construction
    /// errors explicitly.
    #[must_use]
    pub fn from_transport_or_panic<T>(transport: T) -> Self
    where
        T: Eip1193Transport + 'static,
    {
        // SAFETY: the origin is derived from a trusted Rust transport label for
        // this explicitly panic-named compatibility constructor.
        let origin = Origin::new(format!("transport:{}", transport.label()))
            .expect("transport label must produce a valid local origin label");
        // SAFETY: the caller selected the panic-on-invalid constructor for a
        // trusted in-process transport.
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

    fn from_parts(
        transport: Rc<dyn Eip1193Transport>,
        injected_info: Option<InjectedWalletInfo>,
    ) -> Result<Self, BrowserWalletError> {
        let builder =
            discovery::provider_builder_for_injected_info(transport, injected_info.as_ref())?;
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
