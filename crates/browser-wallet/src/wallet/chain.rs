use serde::{Deserialize, Serialize};
use serde_json::Value;

use cow_sdk_core::{ChainId, Redacted, SupportedChainId};

use crate::{BrowserWalletError, WalletSession, provider::hex_quantity};

/// Native-currency metadata for typed add-chain requests.
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
        // SAFETY: known_chain_metadata returns crate-owned literals that share
        // the same validators as user-supplied chain metadata.
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

    fn for_wallet_payload(&self) -> Result<WalletChainParametersPayload<'_>, BrowserWalletError> {
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

    pub(super) fn rpc_payload(&self) -> Result<Value, BrowserWalletError> {
        serde_json::to_value(self.for_wallet_payload()?)
            .map_err(|error| BrowserWalletError::serialization(error.to_string()))
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WalletChainParametersPayload<'a> {
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

/// Builds built-in native-currency metadata for wallet chain registration.
///
/// # Panics
///
/// Panics only if crate-owned chain metadata literals stop satisfying the same
/// validation rules enforced for caller-supplied wallet metadata.
fn known_wallet_native_currency(name: &str, symbol: &str, decimals: u8) -> WalletNativeCurrency {
    // SAFETY: all call sites pass reviewed static metadata from
    // known_chain_metadata.
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
