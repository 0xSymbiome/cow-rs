use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;

use cow_sdk_core::{AppCode, AppCodeError, CowEnv, SupportedChainId};

use super::{AppCodeSet, AppCodeUnset, ChainIdSet, ChainIdUnset, Trading};
use crate::{
    OrderbookClient, PartialTraderParams, TraderParams, TradingError,
    types::validate_orderbook_context,
};

/// Builder for [`Trading`].
///
/// The builder carries two typestate markers that track whether the required
/// [`chain_id`](TradingBuilder::chain_id) and
/// [`app_code`](TradingBuilder::app_code) prerequisites have been
/// supplied. When both are set, [`TradingBuilder::build`] is
/// available and returns a fully-configured [`Trading`] with only a
/// runtime orderbook-binding check remaining.
///
/// The terminal behaves identically on native and `wasm32` targets: when no
/// orderbook client is injected, the default orderbook factory constructs
/// one lazily through `OrderbookApi::builder()`, whose default-transport
/// terminal exists on both targets (native `ReqwestTransport`, browser
/// `FetchTransport`).
#[derive(Clone)]
pub struct TradingBuilder<C = ChainIdUnset, A = AppCodeUnset> {
    trader_defaults: PartialTraderParams,
    orderbook: Option<Arc<dyn OrderbookClient>>,
    app_code_error: Option<AppCodeError>,
    _state: PhantomData<(C, A)>,
}

impl<C, A> fmt::Debug for TradingBuilder<C, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TradingBuilder")
            .field("trader_defaults", &self.trader_defaults)
            .field("orderbook", &self.orderbook.is_some())
            .field("app_code_error", &self.app_code_error)
            .finish()
    }
}

impl Default for TradingBuilder<ChainIdUnset, AppCodeUnset> {
    fn default() -> Self {
        Self {
            trader_defaults: PartialTraderParams::default(),
            orderbook: None,
            app_code_error: None,
            _state: PhantomData,
        }
    }
}

impl TradingBuilder<ChainIdUnset, AppCodeUnset> {
    /// Creates a new builder with empty defaults.
    ///
    /// The returned builder is in the typestate `<ChainIdUnset, AppCodeUnset>`
    /// so the compile-time-checked [`TradingBuilder::build`] terminal is
    /// only unlocked after the [`TradingBuilder::chain_id`] and
    /// [`TradingBuilder::app_code`] prerequisites are supplied.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds a ready-state [`Trading`] from total trader parameters.
    ///
    /// For callers that already hold a complete [`TraderParams`] — chain id and
    /// a validated `appCode` are present by construction, so this terminal is
    /// infallible. The orderbook client is the default per-chain factory; to
    /// inject a custom client, use [`Trading::builder`] with
    /// [`TradingBuilder::orderbook`].
    #[must_use]
    pub fn ready(params: TraderParams) -> Trading {
        let TraderParams {
            chain_id,
            app_code,
            env,
            settlement_contract_override,
            eth_flow_contract_override,
        } = params;

        Trading {
            trader_defaults: PartialTraderParams {
                chain_id: Some(chain_id),
                app_code: Some(app_code),
                env,
                settlement_contract_override,
                eth_flow_contract_override,
            },
            orderbook: None,
        }
    }
}

impl<C, A> TradingBuilder<C, A> {
    /// Returns a copy of this builder with a default chain id.
    ///
    /// Transitions the builder's chain-id typestate to [`ChainIdSet`];
    /// [`TradingBuilder::build`] unlocks once app code is also set.
    #[must_use]
    pub fn chain_id(self, chain_id: SupportedChainId) -> TradingBuilder<ChainIdSet, A> {
        TradingBuilder {
            trader_defaults: PartialTraderParams {
                chain_id: Some(chain_id),
                ..self.trader_defaults
            },
            orderbook: self.orderbook,
            app_code_error: self.app_code_error,
            _state: PhantomData,
        }
    }

    /// Returns a copy of this builder with a validated default app code.
    ///
    /// Transitions the builder's app-code typestate to [`AppCodeSet`], which
    /// completes the typestate for [`TradingBuilder::build`] once
    /// chain id is also set.
    ///
    /// Invalid input is recorded and surfaced by the builder terminal as
    /// [`TradingError::AppCode`]. Deferring the error to the terminal keeps the
    /// fluent construction chain ergonomic while preserving typed validation.
    #[must_use]
    pub fn app_code<T>(self, app_code: T) -> TradingBuilder<C, AppCodeSet>
    where
        T: TryInto<AppCode>,
        T::Error: Into<AppCodeError>,
    {
        let (app_code, app_code_error) = match app_code.try_into() {
            Ok(app_code) => (Some(app_code), None),
            Err(error) => (None, Some(error.into())),
        };

        TradingBuilder {
            trader_defaults: PartialTraderParams {
                app_code,
                ..self.trader_defaults
            },
            orderbook: self.orderbook,
            app_code_error,
            _state: PhantomData,
        }
    }

    /// Returns a copy of this builder with a default environment.
    #[must_use]
    pub const fn env(mut self, env: CowEnv) -> Self {
        self.trader_defaults.env = Some(env);
        self
    }

    /// Returns a copy of this builder with settlement contract overrides.
    #[must_use]
    pub fn settlement_contract_override(
        mut self,
        settlement_contract_override: cow_sdk_core::AddressPerChain,
    ) -> Self {
        self.trader_defaults.settlement_contract_override = Some(settlement_contract_override);
        self
    }

    /// Returns a copy of this builder with `EthFlow` contract overrides.
    #[must_use]
    pub fn eth_flow_contract_override(
        mut self,
        eth_flow_contract_override: cow_sdk_core::AddressPerChain,
    ) -> Self {
        self.trader_defaults.eth_flow_contract_override = Some(eth_flow_contract_override);
        self
    }

    /// Returns a copy of this builder with an injected orderbook client.
    ///
    /// Accepts the client by value and shares it internally, so callers do not
    /// wrap it in [`Arc`]. Use [`TradingBuilder::orderbook_shared`] when an
    /// `Arc<dyn OrderbookClient>` is already held and is shared elsewhere.
    ///
    /// The injected client fixes the effective orderbook chain and environment
    /// for orderbook-bound flows and carries its own [`TransportPolicy`] (retry,
    /// rate-limit, and HTTP-client tuning). Configure that resilience on the
    /// client before injecting it — build it through
    /// [`OrderbookApi::builder().transport_policy(...)`] — rather than on the
    /// trading builder. On the default construction path (no client injected),
    /// the SDK builds an orderbook client with the standard
    /// [`TransportPolicy::default_orderbook`] policy.
    ///
    /// [`TransportPolicy`]: cow_sdk_core::transport::policy::TransportPolicy
    /// [`OrderbookApi::builder().transport_policy(...)`]: cow_sdk_orderbook::OrderbookApiBuilder::transport_policy
    /// [`TransportPolicy::default_orderbook`]: cow_sdk_core::transport::policy::TransportPolicy::default_orderbook
    #[must_use]
    pub fn orderbook(self, orderbook: impl OrderbookClient + 'static) -> Self {
        self.orderbook_shared(Arc::new(orderbook))
    }

    /// Returns a copy of this builder with a shared orderbook client.
    ///
    /// The injected client fixes the effective orderbook chain and environment
    /// for orderbook-bound flows. Prefer [`TradingBuilder::orderbook`] to inject
    /// a client by value; this variant accepts an existing shared handle.
    #[must_use]
    pub fn orderbook_shared(mut self, orderbook: Arc<dyn OrderbookClient>) -> Self {
        self.orderbook = Some(orderbook);
        self
    }

    fn validate_injected_orderbook_binding(&self) -> Result<(), TradingError> {
        if let Some(orderbook_client) = self.orderbook.as_ref() {
            validate_orderbook_context(
                orderbook_client.as_ref(),
                self.trader_defaults.chain_id,
                self.trader_defaults.env,
            )?;
        }

        Ok(())
    }
}

impl TradingBuilder<ChainIdSet, AppCodeSet> {
    /// Builds a fully-configured ready-state [`Trading`].
    ///
    /// The compile-time typestate guarantees that both chain id and app code
    /// have been supplied before this terminal runs. On every target the
    /// default orderbook factory resolves the remaining runtime prerequisite
    /// for quote and post flows lazily through `OrderbookApi::builder()`,
    /// whose default-transport terminal exists on native and `wasm32` alike
    /// (see ADR 0013). Attempting to call `build` on a builder that does not
    /// own the typestate prerequisites is a compile error.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError::InjectedOrderbookContextConflict`] when the
    /// builder's default chain or environment conflicts with an injected
    /// orderbook client.
    ///
    /// `build` is reachable only once both the chain id and application code
    /// have been supplied; the builder typestate makes calling it earlier a
    /// compile error rather than a runtime failure.
    ///
    /// ```compile_fail
    /// use cow_sdk_trading::TradingBuilder;
    /// // Missing chain id: `build` is not callable.
    /// let _ = TradingBuilder::new().app_code("test").build();
    /// ```
    ///
    /// ```compile_fail
    /// use cow_sdk_core::SupportedChainId;
    /// use cow_sdk_trading::TradingBuilder;
    /// // Missing app code: `build` is not callable.
    /// let _ = TradingBuilder::new().chain_id(SupportedChainId::Mainnet).build();
    /// ```
    pub fn build(self) -> Result<Trading, TradingError> {
        if let Some(error) = self.app_code_error {
            return Err(error.into());
        }
        self.validate_injected_orderbook_binding()?;

        Ok(Trading {
            trader_defaults: self.trader_defaults,
            orderbook: self.orderbook,
        })
    }
}
