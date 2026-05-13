#[cfg(target_arch = "wasm32")]
use std::rc::Rc;

use crate::BrowserWalletError;
#[cfg(target_arch = "wasm32")]
use crate::{Eip1193ProviderBuilder, Eip1193Transport, Origin};

use super::{BrowserWallet, InjectedWalletDetectionOptions, InjectedWalletDiscovery};

impl BrowserWallet {
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
}
