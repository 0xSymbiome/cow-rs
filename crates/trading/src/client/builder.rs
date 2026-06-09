use std::marker::PhantomData;
use std::sync::Arc;

use cow_sdk_core::{AppCode, AppCodeError, CowEnv, SupportedChainId};

use super::{AppCodeSet, AppCodeUnset, ChainIdSet, ChainIdUnset, Trading};
use crate::{
    OrderbookClient, PartialTraderParameters, TraderParameters, TradingError, TradingOptions,
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
/// On `wasm32`, the SDK keeps a documented runtime terminal for ready-state
/// orderbook injection: [`TradingBuilder::build`] requires
/// [`TradingBuilder::orderbook_client`] or
/// [`TradingBuilder::options`] with an injected orderbook client, and
/// returns [`TradingError::MissingInjectedOrderbookClient`] when that runtime
/// requirement is not satisfied.
#[derive(Debug, Clone)]
pub struct TradingBuilder<C = ChainIdUnset, A = AppCodeUnset> {
    trader_defaults: PartialTraderParameters,
    options: TradingOptions,
    app_code_error: Option<AppCodeError>,
    _state: PhantomData<(C, A)>,
}

impl Default for TradingBuilder<ChainIdUnset, AppCodeUnset> {
    fn default() -> Self {
        Self {
            trader_defaults: PartialTraderParameters::default(),
            options: TradingOptions::default(),
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
    /// This one-call terminal is for callers that already hold the complete
    /// [`TraderParameters`] shape. It intentionally does not accept
    /// [`PartialTraderParameters`], so chain id and `appCode` stay present
    /// before construction reaches the ready-state terminal.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError::InjectedOrderbookContextConflict`] when the
    /// trader parameters conflict with an injected orderbook client. On
    /// `wasm32`, also returns [`TradingError::MissingInjectedOrderbookClient`]
    /// when no orderbook client has been supplied.
    pub fn ready(
        params: TraderParameters,
        options: TradingOptions,
    ) -> Result<Trading, TradingError> {
        let TraderParameters {
            chain_id,
            app_code,
            env,
            settlement_contract_override,
            eth_flow_contract_override,
        } = params;

        let mut builder = Self::new()
            .options(options)
            .chain_id(chain_id)
            .app_code(app_code);

        if let Some(env) = env {
            builder = builder.env(env);
        }
        if let Some(overrides) = settlement_contract_override {
            builder = builder.settlement_contract_override(overrides);
        }
        if let Some(overrides) = eth_flow_contract_override {
            builder = builder.eth_flow_contract_override(overrides);
        }

        builder.build()
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
            trader_defaults: PartialTraderParameters {
                chain_id: Some(chain_id),
                ..self.trader_defaults
            },
            options: self.options,
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
            trader_defaults: PartialTraderParameters {
                app_code,
                ..self.trader_defaults
            },
            options: self.options,
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

    /// Returns a copy of this builder with explicit SDK options.
    #[must_use]
    pub fn options(mut self, options: TradingOptions) -> Self {
        self.options = options;
        self
    }

    /// Returns a copy of this builder with an injected orderbook client.
    ///
    /// Accepts the client by value and shares it internally, so callers do not
    /// wrap it in [`Arc`]. Use [`TradingBuilder::orderbook_client`] when an
    /// `Arc<dyn OrderbookClient>` is already held and is shared elsewhere.
    ///
    /// The injected client fixes the effective orderbook chain and environment
    /// for orderbook-bound flows.
    #[must_use]
    pub fn orderbook(self, orderbook: impl OrderbookClient + 'static) -> Self {
        self.orderbook_client(Arc::new(orderbook))
    }

    /// Returns a copy of this builder with an injected orderbook client.
    ///
    /// The injected client fixes the effective orderbook chain and environment
    /// for orderbook-bound flows. Prefer [`TradingBuilder::orderbook`] to inject
    /// a client by value; this variant accepts an existing shared handle.
    #[must_use]
    pub fn orderbook_client(mut self, orderbook_client: Arc<dyn OrderbookClient>) -> Self {
        self.options = self.options.with_orderbook_client(orderbook_client);
        self
    }

    fn validate_injected_orderbook_binding(&self) -> Result<(), TradingError> {
        if let Some(orderbook_client) = self.options.orderbook_client() {
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
    /// have been supplied before this terminal runs. On native targets the
    /// default orderbook factory resolves the remaining runtime prerequisite
    /// for quote and post flows. On `wasm32` targets, the builder requires an
    /// injected orderbook client through
    /// [`crate::TradingOptions::with_orderbook_client`] because the browser
    /// runtime does not ship a default HTTP transport; see ADR 0013.
    /// This is the chosen `wasm32` posture for the ready terminal: the
    /// requirement remains a documented runtime terminal check rather than a
    /// third typestate axis, keeping the public builder state readable while
    /// still failing before any quote or post method can run.
    /// Attempting to call `build` on a builder that does not own the
    /// typestate prerequisites is a compile error.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError::InjectedOrderbookContextConflict`] when the
    /// builder's default chain or environment conflicts with an injected
    /// orderbook client. On `wasm32`, also returns
    /// [`TradingError::MissingInjectedOrderbookClient`] when no orderbook
    /// client has been supplied.
    ///
    /// `build` is reachable only once both the chain id and application code
    /// have been supplied; the builder typestate makes calling it earlier a
    /// compile error rather than a runtime failure.
    pub fn build(self) -> Result<Trading, TradingError> {
        if let Some(error) = self.app_code_error {
            return Err(error.into());
        }
        self.validate_injected_orderbook_binding()?;

        // On wasm32 targets the default orderbook factory cannot run because
        // ADR 0013 requires an explicit HTTP transport. Fail at the terminal
        // instead of deferring the missing-client error to the first
        // orderbook-bound call.
        #[cfg(target_arch = "wasm32")]
        if self.options.orderbook_client().is_none() {
            return Err(TradingError::MissingInjectedOrderbookClient);
        }

        Ok(Trading {
            trader_defaults: self.trader_defaults,
            options: self.options,
        })
    }
}
