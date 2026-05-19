use std::marker::PhantomData;
use std::sync::Arc;

use cow_sdk_core::{Address, CowEnv, SupportedChainId};

use super::{AppCodeSet, AppCodeUnset, ChainIdSet, ChainIdUnset, HelperOnlySdk, TradingSdk};
use crate::{
    AppCode, AppCodeError, OrderbookClient, PartialTraderParameters, TraderParameters,
    TradingError, TradingSdkOptions, types::validate_orderbook_context,
};

/// Builder for [`TradingSdk`].
///
/// The builder carries two typestate markers that track whether the required
/// [`chain_id`](TradingSdkBuilder::with_chain_id) and
/// [`app_code`](TradingSdkBuilder::with_app_code) prerequisites have been
/// supplied. When both are set, [`TradingSdkBuilder::build_ready`] is
/// available and returns a fully-configured [`TradingSdk`] with only a
/// runtime orderbook-binding check remaining. When only a chain id is set,
/// [`TradingSdkBuilder::build_helper_only`] returns a [`HelperOnlySdk`] with
/// no quote, post, order-lookup, or off-chain cancellation methods.
///
/// On `wasm32`, the SDK keeps a documented runtime terminal for ready-state
/// orderbook injection: [`TradingSdkBuilder::build_ready`] requires
/// [`TradingSdkBuilder::with_orderbook_client`] or
/// [`TradingSdkBuilder::with_options`] with an injected orderbook client, and
/// returns [`TradingError::MissingInjectedOrderbookClient`] when that runtime
/// requirement is not satisfied.
#[derive(Debug, Clone)]
pub struct TradingSdkBuilder<C = ChainIdUnset, A = AppCodeUnset> {
    trader_defaults: PartialTraderParameters,
    options: TradingSdkOptions,
    app_code_error: Option<AppCodeError>,
    order_bounds: crate::validation::OrderValidityBounds,
    _state: PhantomData<(C, A)>,
}

impl Default for TradingSdkBuilder<ChainIdUnset, AppCodeUnset> {
    fn default() -> Self {
        Self {
            trader_defaults: PartialTraderParameters::default(),
            options: TradingSdkOptions::default(),
            app_code_error: None,
            order_bounds: crate::validation::OrderValidityBounds::SERVICES_DEFAULT,
            _state: PhantomData,
        }
    }
}

impl TradingSdkBuilder<ChainIdUnset, AppCodeUnset> {
    /// Creates a new builder with empty defaults.
    ///
    /// The returned builder is in the typestate `<ChainIdUnset, AppCodeUnset>`
    /// so the compile-time-checked [`TradingSdkBuilder::build_ready`] and
    /// [`TradingSdkBuilder::build_helper_only`] terminals are only unlocked
    /// after the corresponding [`TradingSdkBuilder::with_chain_id`] and
    /// [`TradingSdkBuilder::with_app_code`] prerequisites are supplied.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds a ready-state [`TradingSdk`] from total trader parameters.
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
        options: TradingSdkOptions,
    ) -> Result<TradingSdk, TradingError> {
        let TraderParameters {
            chain_id,
            app_code,
            env,
            settlement_contract_override,
            eth_flow_contract_override,
        } = params;

        let mut builder = Self::new()
            .with_options(options)
            .with_chain_id(chain_id)
            .with_app_code(app_code);

        if let Some(env) = env {
            builder = builder.with_env(env);
        }
        if let Some(overrides) = settlement_contract_override {
            builder = builder.with_settlement_contract_override(overrides);
        }
        if let Some(overrides) = eth_flow_contract_override {
            builder = builder.with_eth_flow_contract_override(overrides);
        }

        builder.build_ready()
    }

    /// Builds a [`HelperOnlySdk`] from total chain authority.
    ///
    /// This one-call terminal is for chain-bound helper workflows that need no
    /// quote or submission attribution. The returned type does not expose
    /// quote, post, order-lookup, or off-chain cancellation methods.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError::InjectedOrderbookContextConflict`] when the
    /// chain id conflicts with an injected orderbook client.
    pub fn helper_only(
        chain_id: SupportedChainId,
        options: TradingSdkOptions,
    ) -> Result<HelperOnlySdk, TradingError> {
        Self::new()
            .with_options(options)
            .with_chain_id(chain_id)
            .build_helper_only()
    }
}

impl<C, A> TradingSdkBuilder<C, A> {
    /// Returns a copy of this builder with trader defaults replaced.
    ///
    /// Replacing the defaults does not transition the typestate markers;
    /// callers that want the compile-time-checked terminals must still reach
    /// the chain-id and app-code states through the explicit
    /// [`TradingSdkBuilder::with_chain_id`] and
    /// [`TradingSdkBuilder::with_app_code`] setters.
    #[must_use]
    pub fn with_trader_defaults(mut self, trader_defaults: PartialTraderParameters) -> Self {
        self.trader_defaults = trader_defaults;
        self
    }

    /// Returns a copy of this builder with a default chain id.
    ///
    /// Transitions the builder's chain-id typestate to
    /// [`ChainIdSet`], which unlocks
    /// [`TradingSdkBuilder::build_helper_only`] for any app-code state and
    /// [`TradingSdkBuilder::build_ready`] once app code is also set.
    #[must_use]
    pub fn with_chain_id(self, chain_id: SupportedChainId) -> TradingSdkBuilder<ChainIdSet, A> {
        TradingSdkBuilder {
            trader_defaults: PartialTraderParameters {
                chain_id: Some(chain_id),
                ..self.trader_defaults
            },
            options: self.options,
            app_code_error: self.app_code_error,
            order_bounds: self.order_bounds,
            _state: PhantomData,
        }
    }

    /// Returns a copy of this builder with a validated default app code.
    ///
    /// Transitions the builder's app-code typestate to [`AppCodeSet`], which
    /// completes the typestate for [`TradingSdkBuilder::build_ready`] once
    /// chain id is also set.
    ///
    /// Invalid input is recorded and surfaced by the builder terminal as
    /// [`TradingError::AppCode`]. Deferring the error to the terminal keeps the
    /// fluent construction chain ergonomic while preserving typed validation.
    #[must_use]
    pub fn with_app_code<T>(self, app_code: T) -> TradingSdkBuilder<C, AppCodeSet>
    where
        T: TryInto<AppCode>,
        T::Error: Into<AppCodeError>,
    {
        let (app_code, app_code_error) = match app_code.try_into() {
            Ok(app_code) => (Some(app_code), None),
            Err(error) => (None, Some(error.into())),
        };

        TradingSdkBuilder {
            trader_defaults: PartialTraderParameters {
                app_code,
                ..self.trader_defaults
            },
            options: self.options,
            app_code_error,
            order_bounds: self.order_bounds,
            _state: PhantomData,
        }
    }

    /// Returns a copy of this builder with a default owner.
    #[must_use]
    pub const fn with_owner(mut self, owner: Address) -> Self {
        self.trader_defaults.owner = Some(owner);
        self
    }

    /// Returns a copy of this builder with a default environment.
    #[must_use]
    pub const fn with_env(mut self, env: CowEnv) -> Self {
        self.trader_defaults.env = Some(env);
        self
    }

    /// Returns a copy of this builder with settlement contract overrides.
    #[must_use]
    pub fn with_settlement_contract_override(
        mut self,
        settlement_contract_override: cow_sdk_core::AddressPerChain,
    ) -> Self {
        self.trader_defaults.settlement_contract_override = Some(settlement_contract_override);
        self
    }

    /// Returns a copy of this builder with `EthFlow` contract overrides.
    #[must_use]
    pub fn with_eth_flow_contract_override(
        mut self,
        eth_flow_contract_override: cow_sdk_core::AddressPerChain,
    ) -> Self {
        self.trader_defaults.eth_flow_contract_override = Some(eth_flow_contract_override);
        self
    }

    /// Returns a copy of this builder with explicit SDK options.
    #[must_use]
    pub fn with_options(mut self, options: TradingSdkOptions) -> Self {
        self.options = options;
        self
    }

    /// Returns a copy of this builder with an injected orderbook client.
    ///
    /// The injected client fixes the effective orderbook chain and environment
    /// for orderbook-bound flows.
    #[must_use]
    pub fn with_orderbook_client(mut self, orderbook_client: Arc<dyn OrderbookClient>) -> Self {
        self.options = self.options.with_orderbook_client(orderbook_client);
        self
    }

    /// Returns a copy of this builder with an injected quote cache.
    ///
    /// The cache is instance-scoped and never registered globally on the
    /// caller's behalf. Omitting this call keeps the pass-through
    /// [`crate::NoopQuoteCache`] default.
    #[must_use]
    pub fn with_quote_cache(mut self, quote_cache: Arc<dyn crate::cache::QuoteCache>) -> Self {
        self.options = self.options.with_quote_cache(quote_cache);
        self
    }

    /// Returns a copy of this builder with a custom [`crate::validation::OrderValidityBounds`].
    ///
    /// The default is [`crate::validation::OrderValidityBounds::SERVICES_DEFAULT`],
    /// which matches the reviewed services production configuration
    /// (minimum 60 seconds, market-class maximum 3 hours, limit-class
    /// maximum 1 year). A tighter policy may be supplied to enforce
    /// stricter client-side lifetime bounds before any bytes cross the
    /// wire.
    #[must_use]
    pub const fn with_order_bounds(
        mut self,
        bounds: crate::validation::OrderValidityBounds,
    ) -> Self {
        self.order_bounds = bounds;
        self
    }

    /// Returns the configured [`crate::validation::OrderValidityBounds`] for this builder.
    #[must_use]
    pub const fn order_bounds(&self) -> crate::validation::OrderValidityBounds {
        self.order_bounds
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

impl<A> TradingSdkBuilder<ChainIdSet, A> {
    /// Builds a [`HelperOnlySdk`].
    ///
    /// The returned SDK exposes only chain-bound helpers: pre-sign
    /// transaction construction, allowance reads, approval submission, and
    /// on-chain cancellation. Quote, post, order-lookup, and off-chain
    /// cancellation methods are not part of this type.
    ///
    /// The compile-time typestate guarantees that a chain id has been
    /// supplied before this terminal runs, so the only remaining runtime
    /// validation is the injected orderbook binding.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError::InjectedOrderbookContextConflict`] when the
    /// builder's default chain or environment conflicts with an injected
    /// orderbook client.
    pub fn build_helper_only(self) -> Result<HelperOnlySdk, TradingError> {
        if let Some(error) = self.app_code_error {
            return Err(error.into());
        }
        self.validate_injected_orderbook_binding()?;

        Ok(HelperOnlySdk {
            trader_defaults: self.trader_defaults,
            options: self.options,
        })
    }
}

impl TradingSdkBuilder<ChainIdSet, AppCodeSet> {
    /// Builds a fully-configured ready-state [`TradingSdk`].
    ///
    /// The compile-time typestate guarantees that both chain id and app code
    /// have been supplied before this terminal runs. On native targets the
    /// default orderbook factory resolves the remaining runtime prerequisite
    /// for quote and post flows. On `wasm32` targets, the builder requires an
    /// injected orderbook client through
    /// [`crate::TradingSdkOptions::with_orderbook_client`] because the browser
    /// runtime does not ship a default HTTP transport; see ADR 0013.
    /// This is the chosen `wasm32` posture for the ready terminal: the
    /// requirement remains a documented runtime terminal check rather than a
    /// third typestate axis, keeping the public builder state readable while
    /// still failing before any quote or post method can run.
    /// Attempting to call `build_ready` on a builder that does not own the
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
    /// ```compile_fail
    /// use cow_sdk_trading::TradingSdkBuilder;
    /// let _ = TradingSdkBuilder::new()
    ///     .with_app_code("test")
    ///     .build_ready();
    /// ```
    ///
    /// ```compile_fail
    /// use cow_sdk_core::SupportedChainId;
    /// use cow_sdk_trading::TradingSdkBuilder;
    /// let _ = TradingSdkBuilder::new()
    ///     .with_chain_id(SupportedChainId::Mainnet)
    ///     .build_ready();
    /// ```
    pub fn build_ready(self) -> Result<TradingSdk, TradingError> {
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

        Ok(TradingSdk {
            trader_defaults: self.trader_defaults,
            options: self.options,
            order_bounds: self.order_bounds,
        })
    }
}
