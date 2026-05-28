use serde::{Deserialize, Serialize};

use cow_sdk_core::{
    Address, AddressPerChain, AppCode, AppCodeError, CowEnv, OrderUid, SupportedChainId,
};

/// Fully resolved trader configuration used by order-posting and on-chain flows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TraderParameters {
    /// Active chain id for the workflow.
    pub chain_id: SupportedChainId,
    /// App code written into generated app-data documents.
    pub app_code: AppCode,
    /// Optional environment override for endpoint and contract resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<CowEnv>,
    /// Optional settlement contract overrides keyed by chain id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settlement_contract_override: Option<AddressPerChain>,
    /// Optional `EthFlow` contract overrides keyed by chain id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eth_flow_contract_override: Option<AddressPerChain>,
}

impl TraderParameters {
    /// Creates trader parameters with the required chain and app-code fields.
    ///
    /// # Errors
    ///
    /// Returns [`AppCodeError`] when `app_code` is empty or contains
    /// forbidden control characters.
    pub fn new<T>(chain_id: SupportedChainId, app_code: T) -> Result<Self, AppCodeError>
    where
        T: TryInto<AppCode>,
        T::Error: Into<AppCodeError>,
    {
        Ok(Self {
            chain_id,
            app_code: app_code.try_into().map_err(Into::into)?,
            env: None,
            settlement_contract_override: None,
            eth_flow_contract_override: None,
        })
    }

    /// Returns a copy with an explicit environment override.
    #[must_use]
    pub const fn with_env(mut self, env: CowEnv) -> Self {
        self.env = Some(env);
        self
    }

    /// Returns a copy with settlement-contract overrides keyed by chain id.
    #[must_use]
    pub fn with_settlement_contract_override(mut self, overrides: AddressPerChain) -> Self {
        self.settlement_contract_override = Some(overrides);
        self
    }

    /// Returns a copy with `EthFlow`-contract overrides keyed by chain id.
    #[must_use]
    pub fn with_eth_flow_contract_override(mut self, overrides: AddressPerChain) -> Self {
        self.eth_flow_contract_override = Some(overrides);
        self
    }
}

/// Partial trader defaults stored on [`crate::TradingSdk`] and its builder.
///
/// Carries the protocol-resolution defaults a `TradingSdk` instance
/// applies when call-level parameters omit them: chain id, app code,
/// environment, settlement-contract overrides, and `EthFlow`-contract
/// overrides. The SDK does not store a default owner; per-call
/// [`crate::TradeParameters::owner`] (with the signer's address as the
/// implicit fallback for signer-backed flows, or
/// `advanced_settings.quote_request.from` for quote-only flows) is the
/// sole owner source.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PartialTraderParameters {
    /// Default chain id when call-level params omit it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<SupportedChainId>,
    /// Default app code written into generated app-data documents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_code: Option<AppCode>,
    /// Default environment for endpoint and contract resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<CowEnv>,
    /// Optional settlement contract overrides keyed by chain id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settlement_contract_override: Option<AddressPerChain>,
    /// Optional `EthFlow` contract overrides keyed by chain id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eth_flow_contract_override: Option<AddressPerChain>,
}

impl PartialTraderParameters {
    /// Creates an empty partial-trader-parameters bundle; attach values through the `with_*` setters.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a copy with an explicit default chain id.
    #[must_use]
    pub const fn with_chain_id(mut self, chain_id: SupportedChainId) -> Self {
        self.chain_id = Some(chain_id);
        self
    }

    /// Returns a copy with an explicit default app code.
    ///
    /// # Errors
    ///
    /// Returns [`AppCodeError`] when `app_code` is empty or contains
    /// forbidden control characters.
    pub fn with_app_code<T>(mut self, app_code: T) -> Result<Self, AppCodeError>
    where
        T: TryInto<AppCode>,
        T::Error: Into<AppCodeError>,
    {
        self.app_code = Some(app_code.try_into().map_err(Into::into)?);
        Ok(self)
    }

    /// Returns a copy with an explicit default environment.
    #[must_use]
    pub const fn with_env(mut self, env: CowEnv) -> Self {
        self.env = Some(env);
        self
    }

    /// Returns a copy with settlement-contract overrides keyed by chain id.
    #[must_use]
    pub fn with_settlement_contract_override(mut self, overrides: AddressPerChain) -> Self {
        self.settlement_contract_override = Some(overrides);
        self
    }

    /// Returns a copy with `EthFlow`-contract overrides keyed by chain id.
    #[must_use]
    pub fn with_eth_flow_contract_override(mut self, overrides: AddressPerChain) -> Self {
        self.eth_flow_contract_override = Some(overrides);
        self
    }
}

/// Quoter configuration used by quote-only and quote-and-sign flows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct QuoterParameters {
    /// Active chain id for the workflow.
    pub chain_id: SupportedChainId,
    /// App code written into generated app-data documents.
    pub app_code: AppCode,
    /// Effective account used for quote ownership.
    pub account: Address,
    /// Optional environment override for endpoint and contract resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<CowEnv>,
    /// Optional settlement contract overrides keyed by chain id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settlement_contract_override: Option<AddressPerChain>,
    /// Optional `EthFlow` contract overrides keyed by chain id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eth_flow_contract_override: Option<AddressPerChain>,
}

impl QuoterParameters {
    /// Creates quoter parameters with the required chain, app-code, and account fields.
    ///
    /// # Errors
    ///
    /// Returns [`AppCodeError`] when `app_code` is empty or contains
    /// forbidden control characters.
    pub fn new<T>(
        chain_id: SupportedChainId,
        app_code: T,
        account: Address,
    ) -> Result<Self, AppCodeError>
    where
        T: TryInto<AppCode>,
        T::Error: Into<AppCodeError>,
    {
        Ok(Self {
            chain_id,
            app_code: app_code.try_into().map_err(Into::into)?,
            account,
            env: None,
            settlement_contract_override: None,
            eth_flow_contract_override: None,
        })
    }

    /// Returns a copy with an explicit environment override.
    #[must_use]
    pub const fn with_env(mut self, env: CowEnv) -> Self {
        self.env = Some(env);
        self
    }

    /// Returns a copy with settlement-contract overrides keyed by chain id.
    #[must_use]
    pub fn with_settlement_contract_override(mut self, overrides: AddressPerChain) -> Self {
        self.settlement_contract_override = Some(overrides);
        self
    }

    /// Returns a copy with `EthFlow`-contract overrides keyed by chain id.
    #[must_use]
    pub fn with_eth_flow_contract_override(mut self, overrides: AddressPerChain) -> Self {
        self.eth_flow_contract_override = Some(overrides);
        self
    }
}

/// Parameters for order lookup, cancellation, and on-chain helper flows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct OrderTraderParameters {
    /// Target order UID.
    pub order_uid: OrderUid,
    /// Optional chain-id override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<SupportedChainId>,
    /// Optional environment override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<CowEnv>,
    /// Optional settlement contract overrides keyed by chain id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settlement_contract_override: Option<AddressPerChain>,
    /// Optional `EthFlow` contract overrides keyed by chain id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eth_flow_contract_override: Option<AddressPerChain>,
}

impl OrderTraderParameters {
    /// Creates order-trader parameters with the required order UID.
    #[must_use]
    pub const fn new(order_uid: OrderUid) -> Self {
        Self {
            order_uid,
            chain_id: None,
            env: None,
            settlement_contract_override: None,
            eth_flow_contract_override: None,
        }
    }

    /// Returns a copy with an explicit chain-id override.
    #[must_use]
    pub const fn with_chain_id(mut self, chain_id: SupportedChainId) -> Self {
        self.chain_id = Some(chain_id);
        self
    }

    /// Returns a copy with an explicit environment override.
    #[must_use]
    pub const fn with_env(mut self, env: CowEnv) -> Self {
        self.env = Some(env);
        self
    }

    /// Returns a copy with settlement-contract overrides keyed by chain id.
    #[must_use]
    pub fn with_settlement_contract_override(mut self, overrides: AddressPerChain) -> Self {
        self.settlement_contract_override = Some(overrides);
        self
    }

    /// Returns a copy with `EthFlow`-contract overrides keyed by chain id.
    #[must_use]
    pub fn with_eth_flow_contract_override(mut self, overrides: AddressPerChain) -> Self {
        self.eth_flow_contract_override = Some(overrides);
        self
    }
}
