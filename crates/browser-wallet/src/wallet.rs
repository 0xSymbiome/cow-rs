use std::{cell::RefCell, rc::Rc};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use cow_sdk_core::{Address, ChainId, SupportedChainId};

use crate::{
    BrowserWalletError, Eip1193Provider, Eip1193Signer, Eip1193Transport, EventLog, WalletSession,
    provider::{Eip1193Provider as ProviderImpl, hex_quantity},
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InjectedWalletDiscoverySource {
    Eip6963,
    #[default]
    LegacyWindowEthereum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InjectedWalletDetectionOptions {
    pub timeout_ms: u32,
}

impl InjectedWalletDetectionOptions {
    pub const DEFAULT_TIMEOUT_MS: u32 = 500;

    pub fn new(timeout_ms: u32) -> Self {
        Self { timeout_ms }
    }

    pub fn timeout_ms(self) -> u32 {
        self.timeout_ms
    }
}

impl Default for InjectedWalletDetectionOptions {
    fn default() -> Self {
        Self::new(Self::DEFAULT_TIMEOUT_MS)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InjectedWalletInfo {
    pub provider_label: String,
    #[serde(default)]
    pub discovery_source: InjectedWalletDiscoverySource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_rdns: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_icon: Option<String>,
    pub is_meta_mask: bool,
    pub is_coinbase_wallet: bool,
    pub is_rabby: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletNativeCurrency {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

impl WalletNativeCurrency {
    pub fn new(
        name: impl Into<String>,
        symbol: impl Into<String>,
        decimals: u8,
    ) -> Result<Self, BrowserWalletError> {
        Ok(Self {
            name: validate_wallet_text(name.into(), "native currency name", None)?,
            symbol: validate_wallet_text(symbol.into(), "native currency symbol", None)?,
            decimals,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletChainParameters {
    pub chain_id: SupportedChainId,
    pub chain_name: String,
    pub native_currency: WalletNativeCurrency,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rpc_urls: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub block_explorer_urls: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub icon_urls: Vec<String>,
}

impl WalletChainParameters {
    pub fn new(
        chain_id: SupportedChainId,
        chain_name: impl Into<String>,
        native_currency: WalletNativeCurrency,
    ) -> Result<Self, BrowserWalletError> {
        Ok(Self {
            chain_id,
            chain_name: validate_wallet_text(
                chain_name.into(),
                "chain name",
                Some(u64::from(chain_id)),
            )?,
            native_currency,
            rpc_urls: Vec::new(),
            block_explorer_urls: Vec::new(),
            icon_urls: Vec::new(),
        })
    }

    pub fn for_supported_chain(chain_id: SupportedChainId) -> Self {
        let (chain_name, native_currency) = known_chain_metadata(chain_id);
        Self {
            chain_id,
            chain_name: chain_name.to_owned(),
            native_currency,
            rpc_urls: Vec::new(),
            block_explorer_urls: Vec::new(),
            icon_urls: Vec::new(),
        }
    }

    pub fn try_with_rpc_url(
        mut self,
        rpc_url: impl Into<String>,
    ) -> Result<Self, BrowserWalletError> {
        self.rpc_urls.push(validate_wallet_url(
            rpc_url.into(),
            "RPC URL",
            u64::from(self.chain_id),
        )?);
        Ok(self)
    }

    pub fn try_with_block_explorer_url(
        mut self,
        block_explorer_url: impl Into<String>,
    ) -> Result<Self, BrowserWalletError> {
        self.block_explorer_urls.push(validate_wallet_url(
            block_explorer_url.into(),
            "block explorer URL",
            u64::from(self.chain_id),
        )?);
        Ok(self)
    }

    pub fn try_with_icon_url(
        mut self,
        icon_url: impl Into<String>,
    ) -> Result<Self, BrowserWalletError> {
        self.icon_urls.push(validate_wallet_url(
            icon_url.into(),
            "icon URL",
            u64::from(self.chain_id),
        )?);
        Ok(self)
    }

    pub fn validate(&self) -> Result<(), BrowserWalletError> {
        let chain_id = u64::from(self.chain_id);
        let _ = validate_wallet_text(self.chain_name.clone(), "chain name", Some(chain_id))?;
        let _ = validate_wallet_text(
            self.native_currency.name.clone(),
            "native currency name",
            Some(chain_id),
        )?;
        let _ = validate_wallet_text(
            self.native_currency.symbol.clone(),
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
            let _ = validate_wallet_url(url.clone(), "RPC URL", chain_id)?;
        }
        for url in &self.block_explorer_urls {
            let _ = validate_wallet_url(url.clone(), "block explorer URL", chain_id)?;
        }
        for url in &self.icon_urls {
            let _ = validate_wallet_url(url.clone(), "icon URL", chain_id)?;
        }
        Ok(())
    }

    fn rpc_payload(&self) -> Result<Value, BrowserWalletError> {
        self.validate()?;
        let mut payload = json!({
            "chainId": hex_quantity(&u64::from(self.chain_id).to_string())?,
            "chainName": self.chain_name,
            "nativeCurrency": {
                "name": self.native_currency.name,
                "symbol": self.native_currency.symbol,
                "decimals": self.native_currency.decimals,
            },
            "rpcUrls": self.rpc_urls,
        });
        if !self.block_explorer_urls.is_empty() {
            payload["blockExplorerUrls"] = serde_json::to_value(&self.block_explorer_urls)
                .map_err(|error| BrowserWalletError::serialization(error.to_string()))?;
        }
        if !self.icon_urls.is_empty() {
            payload["iconUrls"] = serde_json::to_value(&self.icon_urls)
                .map_err(|error| BrowserWalletError::serialization(error.to_string()))?;
        }
        Ok(payload)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WalletChainChangeKind {
    Added,
    Switched,
    AddedThenSwitched,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletChainChange {
    pub requested_chain_id: SupportedChainId,
    pub kind: WalletChainChangeKind,
    pub session: WalletSession,
}

#[derive(Clone)]
struct DiscoveredInjectedWallet {
    transport: Rc<dyn Eip1193Transport>,
    info: InjectedWalletInfo,
}

#[derive(Clone)]
pub struct InjectedWalletDiscovery {
    timeout_ms: u32,
    used_legacy_fallback: bool,
    wallets: Vec<DiscoveredInjectedWallet>,
}

impl InjectedWalletDiscovery {
    pub fn wallets(&self) -> Vec<InjectedWalletInfo> {
        self.wallets
            .iter()
            .map(|wallet| wallet.info.clone())
            .collect()
    }

    pub fn len(&self) -> usize {
        self.wallets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.wallets.is_empty()
    }

    pub fn timeout_ms(&self) -> u32 {
        self.timeout_ms
    }

    pub fn used_legacy_fallback(&self) -> bool {
        self.used_legacy_fallback
    }

    pub fn requires_explicit_selection(&self) -> bool {
        self.wallets.len() > 1
    }

    pub fn wallet_at(&self, index: usize) -> Result<BrowserWallet, BrowserWalletError> {
        let wallet = self.wallets.get(index).ok_or_else(|| {
            BrowserWalletError::discovery_selection_out_of_range(index, self.wallets.len())
        })?;
        Ok(BrowserWallet::from_parts(
            wallet.transport.clone(),
            Some(wallet.info.clone()),
        ))
    }

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

#[derive(Clone)]
pub struct BrowserWallet {
    provider: Eip1193Provider,
    injected_info: Option<InjectedWalletInfo>,
}

impl BrowserWallet {
    pub fn from_transport<T>(transport: T) -> Self
    where
        T: Eip1193Transport + 'static,
    {
        Self::from_parts(Rc::new(transport), None)
    }

    pub fn injected_info(&self) -> Option<InjectedWalletInfo> {
        self.injected_info.clone()
    }

    pub fn provider(&self) -> Eip1193Provider {
        self.provider.clone()
    }

    pub fn signer(&self) -> Eip1193Signer {
        Eip1193Signer::new(self.provider.clone(), None)
    }

    pub fn session(&self) -> WalletSession {
        self.provider.session()
    }

    pub fn account(&self) -> Option<Address> {
        self.session().selected_account
    }

    pub fn chain_id(&self) -> Option<ChainId> {
        self.session().chain_id
    }

    pub fn reset_session(&self) -> WalletSession {
        self.provider.reset_session()
    }

    pub fn take_events(&self) -> Vec<crate::WalletEvent> {
        self.provider.events().take()
    }

    pub fn events(&self) -> Vec<crate::WalletEvent> {
        self.provider.events().snapshot()
    }

    pub async fn connect(&self) -> Result<WalletSession, BrowserWalletError> {
        self.provider.query_accounts(true).await?;
        self.provider.query_chain_id().await?;
        Ok(self.session())
    }

    pub async fn request_accounts(&self) -> Result<Vec<Address>, BrowserWalletError> {
        self.provider.query_accounts(true).await
    }

    pub async fn refresh_session(&self) -> Result<WalletSession, BrowserWalletError> {
        let _ = self.provider.query_accounts(false).await?;
        let _ = self.provider.query_chain_id().await?;
        Ok(self.session())
    }

    pub async fn switch_chain(
        &self,
        chain_id: SupportedChainId,
    ) -> Result<WalletSession, BrowserWalletError> {
        self.switch_chain_request(chain_id).await?;
        self.refresh_session().await
    }

    pub async fn add_chain(
        &self,
        parameters: &WalletChainParameters,
    ) -> Result<WalletChainChange, BrowserWalletError> {
        self.add_chain_request(parameters).await?;
        let session = self.refresh_session().await?;
        Ok(WalletChainChange {
            requested_chain_id: parameters.chain_id,
            kind: WalletChainChangeKind::Added,
            session,
        })
    }

    pub async fn switch_or_add_chain(
        &self,
        parameters: &WalletChainParameters,
    ) -> Result<WalletChainChange, BrowserWalletError> {
        match self.switch_chain_request(parameters.chain_id).await {
            Ok(()) => {
                let session = self.refresh_session().await?;
                Ok(WalletChainChange {
                    requested_chain_id: parameters.chain_id,
                    kind: WalletChainChangeKind::Switched,
                    session,
                })
            }
            Err(BrowserWalletError::ChainNotAdded { chain_id, .. })
                if chain_id == u64::from(parameters.chain_id) =>
            {
                self.add_chain_request(parameters).await?;
                self.switch_chain_request(parameters.chain_id).await?;
                let session = self.refresh_session().await?;
                Ok(WalletChainChange {
                    requested_chain_id: parameters.chain_id,
                    kind: WalletChainChangeKind::AddedThenSwitched,
                    session,
                })
            }
            Err(error) => Err(error),
        }
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
    pub async fn discover() -> Result<InjectedWalletDiscovery, BrowserWalletError> {
        Self::discover_with(InjectedWalletDetectionOptions::default()).await
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn discover() -> Result<InjectedWalletDiscovery, BrowserWalletError> {
        Self::discover_with(InjectedWalletDetectionOptions::default()).await
    }

    #[cfg(target_arch = "wasm32")]
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
    pub async fn discover_with(
        options: InjectedWalletDetectionOptions,
    ) -> Result<InjectedWalletDiscovery, BrowserWalletError> {
        Ok(InjectedWalletDiscovery::from_detected_wallets(
            options,
            false,
            Vec::new(),
        ))
    }

    #[cfg(target_arch = "wasm32")]
    pub fn detect() -> Result<Option<Self>, BrowserWalletError> {
        let Some(transport) = crate::js::InjectedProviderTransport::detect_legacy()? else {
            return Ok(None);
        };
        let info = transport.info();
        Ok(Some(Self::from_parts(Rc::new(transport), Some(info))))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn detect() -> Result<Option<Self>, BrowserWalletError> {
        let _ = crate::js::InjectedProviderTransport::detect_legacy()?;
        Ok(None)
    }

    fn from_parts(
        transport: Rc<dyn Eip1193Transport>,
        injected_info: Option<InjectedWalletInfo>,
    ) -> Self {
        let events = EventLog::default();
        let session = Rc::new(RefCell::new(WalletSession {
            wallet_label: transport.label().to_owned(),
            ..WalletSession::default()
        }));
        let provider = ProviderImpl::new(transport, session, events);
        Self {
            provider,
            injected_info,
        }
    }
}

fn validate_wallet_text(
    value: String,
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
    value: String,
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

fn known_chain_metadata(chain_id: SupportedChainId) -> (&'static str, WalletNativeCurrency) {
    match chain_id {
        SupportedChainId::Mainnet => (
            "Ethereum Mainnet",
            WalletNativeCurrency {
                name: "Ether".to_owned(),
                symbol: "ETH".to_owned(),
                decimals: 18,
            },
        ),
        SupportedChainId::Bnb => (
            "BNB Smart Chain",
            WalletNativeCurrency {
                name: "BNB".to_owned(),
                symbol: "BNB".to_owned(),
                decimals: 18,
            },
        ),
        SupportedChainId::GnosisChain => (
            "Gnosis Chain",
            WalletNativeCurrency {
                name: "xDAI".to_owned(),
                symbol: "xDAI".to_owned(),
                decimals: 18,
            },
        ),
        SupportedChainId::Polygon => (
            "Polygon",
            WalletNativeCurrency {
                name: "POL".to_owned(),
                symbol: "POL".to_owned(),
                decimals: 18,
            },
        ),
        SupportedChainId::Base => (
            "Base",
            WalletNativeCurrency {
                name: "Ether".to_owned(),
                symbol: "ETH".to_owned(),
                decimals: 18,
            },
        ),
        SupportedChainId::Plasma => (
            "Plasma",
            WalletNativeCurrency {
                name: "Plasma".to_owned(),
                symbol: "XPL".to_owned(),
                decimals: 18,
            },
        ),
        SupportedChainId::ArbitrumOne => (
            "Arbitrum One",
            WalletNativeCurrency {
                name: "Ether".to_owned(),
                symbol: "ETH".to_owned(),
                decimals: 18,
            },
        ),
        SupportedChainId::Avalanche => (
            "Avalanche C-Chain",
            WalletNativeCurrency {
                name: "Avalanche".to_owned(),
                symbol: "AVAX".to_owned(),
                decimals: 18,
            },
        ),
        SupportedChainId::Ink => (
            "Ink",
            WalletNativeCurrency {
                name: "Ether".to_owned(),
                symbol: "ETH".to_owned(),
                decimals: 18,
            },
        ),
        SupportedChainId::Linea => (
            "Linea",
            WalletNativeCurrency {
                name: "Ether".to_owned(),
                symbol: "ETH".to_owned(),
                decimals: 18,
            },
        ),
        SupportedChainId::Sepolia => (
            "Sepolia",
            WalletNativeCurrency {
                name: "Ether".to_owned(),
                symbol: "ETH".to_owned(),
                decimals: 18,
            },
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
                (
                    transport,
                    InjectedWalletInfo {
                        provider_label: (*label).to_owned(),
                        discovery_source: *source,
                        ..InjectedWalletInfo::default()
                    },
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
}
