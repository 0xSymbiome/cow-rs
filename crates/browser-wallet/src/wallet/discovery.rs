use std::{fmt, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::{BrowserWalletError, Eip1193ProviderBuilder, Eip1193Transport, Origin};

use super::BrowserWallet;

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

    pub(super) fn from_detected_wallets(
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

pub(super) fn provider_builder_for_injected_info(
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

#[cfg(test)]
mod tests {
    use std::rc::Rc;

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
