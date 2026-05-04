use std::marker::PhantomData;
use std::sync::Arc;

use cow_sdk_core::{
    Address, Amount, AsyncProvider, AsyncSigner, CowEnv, Provider, Signer, SupportedChainId,
    TransactionHash,
};
#[cfg(not(target_arch = "wasm32"))]
use cow_sdk_orderbook::OrderBookApi;

use crate::onchain::protocol_options_for_partial_order;
use crate::{
    AllowanceParameters, ApprovalParameters, LimitOrderAdvancedSettings, LimitTradeParameters,
    OrderTraderParameters, OrderbookClient, PartialTraderParameters, QuoteResults,
    QuoterParameters, SwapAdvancedSettings, TradeParameters, TraderParameters, TradingError,
    TradingSdkOptions, cancel_order_onchain_async, get_cow_protocol_allowance,
    get_cow_protocol_allowance_async, get_pre_sign_transaction, get_pre_sign_transaction_async,
    get_quote_only, get_quote_results_async, off_chain_cancel_order_async,
    types::validate_orderbook_context,
};

/// Runtime readiness of a constructed [`TradingSdk`].
///
/// The default `Ready` mode exposes quote, post, and off-chain cancellation
/// flows. `HelperOnly` mode is produced by
/// [`TradingSdkBuilder::build_helper_only`] and restricts those flows so the
/// sdk can only drive chain-bound helpers such as pre-sign transaction
/// construction, allowance reads, approval submission, and on-chain
/// cancellation.
///
/// Classified as `sdk-local-state` in the workspace enum policy manifest: the
/// SDK owns this closed discriminator and the only valid runtime modes are
/// ready-state execution and helper-only execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TradingSdkMode {
    /// Full quote, post, and off-chain cancellation flows are enabled.
    #[default]
    Ready,
    /// Quote, post, and off-chain cancellation flows return
    /// [`TradingError::HelperOnlyMode`].
    HelperOnly,
}

/// Typestate marker for a builder that has not yet been given a chain id.
#[derive(Debug, Clone, Copy)]
pub struct ChainIdUnset(());

/// Typestate marker for a builder that has been given a chain id.
#[derive(Debug, Clone, Copy)]
pub struct ChainIdSet(());

/// Typestate marker for a builder that has not yet been given an `appCode`.
#[derive(Debug, Clone, Copy)]
pub struct AppCodeUnset(());

/// Typestate marker for a builder that has been given an `appCode`.
#[derive(Debug, Clone, Copy)]
pub struct AppCodeSet(());

/// High-level trading facade that stores trader defaults plus optional injected services.
#[derive(Debug, Clone, Default)]
pub struct TradingSdk {
    trader_defaults: PartialTraderParameters,
    options: TradingSdkOptions,
    mode: TradingSdkMode,
    order_bounds: crate::validation::OrderValidityBounds,
}

/// Builder for [`TradingSdk`].
///
/// The builder carries two typestate markers that track whether the required
/// [`chain_id`](TradingSdkBuilder::with_chain_id) and
/// [`app_code`](TradingSdkBuilder::with_app_code) prerequisites have been
/// supplied. When both are set, [`TradingSdkBuilder::build_ready`] is
/// available and returns a fully-configured [`TradingSdk`] with only a
/// runtime orderbook-binding check remaining. When only a chain id is set,
/// [`TradingSdkBuilder::build_helper_only`] returns a helper-mode sdk that
/// can still drive chain-bound helpers but fails closed on quote, post, and
/// off-chain cancellation flows with [`TradingError::HelperOnlyMode`].
#[derive(Debug, Clone)]
pub struct TradingSdkBuilder<C = ChainIdUnset, A = AppCodeUnset> {
    trader_defaults: PartialTraderParameters,
    options: TradingSdkOptions,
    order_bounds: crate::validation::OrderValidityBounds,
    _state: PhantomData<(C, A)>,
}

impl Default for TradingSdkBuilder<ChainIdUnset, AppCodeUnset> {
    fn default() -> Self {
        Self {
            trader_defaults: PartialTraderParameters::default(),
            options: TradingSdkOptions::default(),
            order_bounds: crate::validation::OrderValidityBounds::SERVICES_DEFAULT,
            _state: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typestate_markers_are_sealed_against_external_construction() {
        // These constructors are visible only inside this module because the
        // tuple field is private; external callers cannot write `Marker(())`.
        let _ = ChainIdUnset(());
        let _ = ChainIdSet(());
        let _ = AppCodeUnset(());
        let _ = AppCodeSet(());
    }
}

#[derive(Clone)]
struct ResolvedOrderbookBinding {
    client: Arc<dyn OrderbookClient>,
    chain_id: SupportedChainId,
    env: CowEnv,
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

    /// Builds a helper-only [`TradingSdk`] from total chain authority.
    ///
    /// This one-call terminal is for chain-bound helper workflows that need no
    /// quote or submission attribution. Quote, post, and off-chain
    /// cancellation methods on the returned SDK fail closed with
    /// [`TradingError::HelperOnlyMode`].
    ///
    /// # Errors
    ///
    /// Returns [`TradingError::InjectedOrderbookContextConflict`] when the
    /// chain id conflicts with an injected orderbook client.
    pub fn helper_only(
        chain_id: SupportedChainId,
        options: TradingSdkOptions,
    ) -> Result<TradingSdk, TradingError> {
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
            order_bounds: self.order_bounds,
            _state: PhantomData,
        }
    }

    /// Returns a copy of this builder with a default app code.
    ///
    /// Transitions the builder's app-code typestate to [`AppCodeSet`], which
    /// completes the typestate for [`TradingSdkBuilder::build_ready`] once
    /// chain id is also set.
    #[must_use]
    pub fn with_app_code(self, app_code: impl Into<String>) -> TradingSdkBuilder<C, AppCodeSet> {
        TradingSdkBuilder {
            trader_defaults: PartialTraderParameters {
                app_code: Some(app_code.into()),
                ..self.trader_defaults
            },
            options: self.options,
            order_bounds: self.order_bounds,
            _state: PhantomData,
        }
    }

    /// Returns a copy of this builder with a default owner.
    #[must_use]
    pub fn with_owner(mut self, owner: Address) -> Self {
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
    /// Builds a helper-only [`TradingSdk`].
    ///
    /// The returned SDK is in [`TradingSdkMode::HelperOnly`] so quote, post,
    /// and off-chain cancellation flows return
    /// [`TradingError::HelperOnlyMode`]. Chain-bound helpers (pre-sign
    /// transaction construction, allowance reads, approval submission, and
    /// on-chain cancellation) remain fully usable.
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
    pub fn build_helper_only(self) -> Result<TradingSdk, TradingError> {
        self.validate_injected_orderbook_binding()?;

        Ok(TradingSdk {
            trader_defaults: self.trader_defaults,
            options: self.options,
            mode: TradingSdkMode::HelperOnly,
            order_bounds: self.order_bounds,
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
            mode: TradingSdkMode::Ready,
            order_bounds: self.order_bounds,
        })
    }
}

impl TradingSdk {
    /// Returns a new [`TradingSdkBuilder`] in the `<ChainIdUnset, AppCodeUnset>` typestate.
    #[must_use]
    pub fn builder() -> TradingSdkBuilder<ChainIdUnset, AppCodeUnset> {
        TradingSdkBuilder::new()
    }

    /// Returns the runtime readiness mode selected by the builder.
    #[inline]
    #[must_use]
    pub const fn mode(&self) -> TradingSdkMode {
        self.mode
    }

    /// Returns an error when the SDK is restricted to helper-only flows.
    ///
    /// Quote, post, and off-chain cancellation methods call this helper
    /// before running so helper-mode SDKs fail closed with
    /// [`TradingError::HelperOnlyMode`] instead of invoking a flow that would
    /// depend on missing trader defaults.
    #[inline]
    const fn ensure_ready_mode(&self) -> Result<(), TradingError> {
        match self.mode {
            TradingSdkMode::Ready => Ok(()),
            TradingSdkMode::HelperOnly => Err(TradingError::HelperOnlyMode),
        }
    }

    /// Returns the stored trader defaults.
    #[must_use]
    pub const fn trader_defaults(&self) -> &PartialTraderParameters {
        &self.trader_defaults
    }

    /// Returns the stored SDK options.
    #[must_use]
    pub const fn options(&self) -> &TradingSdkOptions {
        &self.options
    }

    /// Fetches quote-only results using SDK defaults plus optional advanced settings.
    ///
    /// Owner precedence is: quote override `from`, call-level `owner`, SDK default `owner`.
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when required defaults are missing, the quote
    /// request is invalid, or downstream quote construction fails.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?self.trader_defaults.chain_id,
                env = ?self.trader_defaults.env,
                endpoint = "trading.quote_only",
            ),
        ),
    )]
    pub async fn get_quote_only(
        &self,
        mut params: TradeParameters,
        advanced_settings: Option<&SwapAdvancedSettings>,
    ) -> Result<QuoteResults, TradingError> {
        self.ensure_ready_mode()?;
        params.owner = params.owner.or_else(|| self.trader_defaults.owner.clone());
        let owner = self.resolve_quote_owner(&params, advanced_settings)?;
        let (quoter, orderbook) = self.resolve_quoter(owner, params.env)?;

        get_quote_only(
            &params,
            &quoter,
            advanced_settings,
            orderbook.client.as_ref(),
        )
        .await
    }

    /// Fetches quote results for a sync signer.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site.
    ///
    /// # Errors
    ///
    /// Returns any error from [`Self::get_quote_results_async`].
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?self.trader_defaults.chain_id,
                env = ?self.trader_defaults.env,
                endpoint = "trading.quote_results",
            ),
        ),
    )]
    pub async fn get_quote_results<S>(
        &self,
        params: TradeParameters,
        signer: &S,
        advanced_settings: Option<&SwapAdvancedSettings>,
    ) -> Result<QuoteResults, TradingError>
    where
        S: Signer,
        S::Error: std::fmt::Display,
    {
        self.get_quote_results_async(params, signer, advanced_settings)
            .await
    }

    /// Fetches quote results for an async signer.
    ///
    /// Owner precedence is: call-level `owner`, SDK default `owner`, signer address.
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when required defaults are missing, signer
    /// address resolution fails, or downstream quote construction fails.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?self.trader_defaults.chain_id,
                env = ?self.trader_defaults.env,
                endpoint = "trading.quote_results_async",
            ),
        ),
    )]
    pub async fn get_quote_results_async<S>(
        &self,
        mut params: TradeParameters,
        signer: &S,
        advanced_settings: Option<&SwapAdvancedSettings>,
    ) -> Result<QuoteResults, TradingError>
    where
        S: AsyncSigner,
        S::Error: std::fmt::Display,
    {
        self.ensure_ready_mode()?;
        params.owner = params.owner.or_else(|| self.trader_defaults.owner.clone());
        let (trader, orderbook) = self.resolve_orderbook_trader(None, params.env)?;

        get_quote_results_async(
            &params,
            &trader,
            signer,
            advanced_settings,
            orderbook.client.as_ref(),
        )
        .await
    }

    /// Quotes and posts a swap order using a sync signer.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site; cancellation
    /// only affects pre-broadcast work, because once the signed order payload
    /// has been accepted by the orderbook, the order cannot be un-submitted.
    ///
    /// # Errors
    ///
    /// Returns any error from [`Self::post_swap_order_async`].
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?self.trader_defaults.chain_id,
                env = ?self.trader_defaults.env,
                endpoint = "trading.post_swap_order",
            ),
        ),
    )]
    pub async fn post_swap_order<S>(
        &self,
        params: TradeParameters,
        signer: &S,
        advanced_settings: Option<&SwapAdvancedSettings>,
    ) -> Result<crate::OrderPostingResult, TradingError>
    where
        S: Signer,
        S::Error: std::fmt::Display,
    {
        self.post_swap_order_async(params, signer, advanced_settings)
            .await
    }

    /// Quotes and posts a swap order using an async signer.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site; cancellation
    /// only affects pre-broadcast work, because once the signed order payload
    /// has been accepted by the orderbook, the order cannot be un-submitted.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when quoting, signing, app-data upload, or
    /// order submission fails.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?self.trader_defaults.chain_id,
                env = ?self.trader_defaults.env,
                endpoint = "trading.post_swap_order_async",
            ),
        ),
    )]
    pub async fn post_swap_order_async<S>(
        &self,
        mut params: TradeParameters,
        signer: &S,
        advanced_settings: Option<&SwapAdvancedSettings>,
    ) -> Result<crate::OrderPostingResult, TradingError>
    where
        S: AsyncSigner,
        S::Error: std::fmt::Display,
    {
        self.ensure_ready_mode()?;
        params.owner = params.owner.or_else(|| self.trader_defaults.owner.clone());
        let (trader, orderbook) = self.resolve_orderbook_trader(None, params.env)?;

        crate::post::post_swap_order_async_with_bounds(
            &params,
            &trader,
            signer,
            advanced_settings,
            orderbook.client.as_ref(),
            self.order_bounds,
        )
        .await
    }

    /// Posts a swap order from previously computed quote results using a sync signer.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site; cancellation
    /// only affects pre-broadcast work, because once the signed order payload
    /// has been accepted by the orderbook, the order cannot be un-submitted.
    ///
    /// # Errors
    ///
    /// Returns any error from [`Self::post_swap_order_from_quote_async`].
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?self.trader_defaults.chain_id,
                env = ?self.trader_defaults.env,
                endpoint = "trading.post_swap_order_from_quote",
            ),
        ),
    )]
    pub async fn post_swap_order_from_quote<S>(
        &self,
        quote_results: &QuoteResults,
        signer: &S,
        advanced_settings: Option<&SwapAdvancedSettings>,
    ) -> Result<crate::OrderPostingResult, TradingError>
    where
        S: Signer,
        S::Error: std::fmt::Display,
    {
        self.post_swap_order_from_quote_async(quote_results, signer, advanced_settings)
            .await
    }

    /// Posts a swap order from previously computed quote results using an async signer.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site; cancellation
    /// only affects pre-broadcast work, because once the signed order payload
    /// has been accepted by the orderbook, the order cannot be un-submitted.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when the stored orderbook binding no longer
    /// matches the SDK's active orderbook, when app-data merging fails, when
    /// signing fails, or when the orderbook rejects the submission.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?self.trader_defaults.chain_id,
                env = ?self.trader_defaults.env,
                endpoint = "trading.post_swap_order_from_quote_async",
            ),
        ),
    )]
    pub async fn post_swap_order_from_quote_async<S>(
        &self,
        quote_results: &QuoteResults,
        signer: &S,
        advanced_settings: Option<&SwapAdvancedSettings>,
    ) -> Result<crate::OrderPostingResult, TradingError>
    where
        S: AsyncSigner,
        S::Error: std::fmt::Display,
    {
        self.ensure_ready_mode()?;
        let (trader, orderbook) =
            self.resolve_orderbook_trader(None, quote_results.trade_parameters.env)?;

        crate::post::post_swap_order_from_quote_async_with_bounds(
            quote_results,
            &trader,
            signer,
            advanced_settings,
            orderbook.client.as_ref(),
            self.order_bounds,
        )
        .await
    }

    /// Posts a limit order using a sync signer.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site; cancellation
    /// only affects pre-broadcast work, because once the signed order payload
    /// has been accepted by the orderbook, the order cannot be un-submitted.
    ///
    /// # Errors
    ///
    /// Returns any error from [`Self::post_limit_order_async`].
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?self.trader_defaults.chain_id,
                env = ?self.trader_defaults.env,
                endpoint = "trading.post_limit_order",
            ),
        ),
    )]
    pub async fn post_limit_order<S>(
        &self,
        params: LimitTradeParameters,
        signer: &S,
        advanced_settings: Option<&LimitOrderAdvancedSettings>,
    ) -> Result<crate::OrderPostingResult, TradingError>
    where
        S: Signer,
        S::Error: std::fmt::Display,
    {
        self.post_limit_order_async(params, signer, advanced_settings)
            .await
    }

    /// Posts a limit order using an async signer.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site; cancellation
    /// only affects pre-broadcast work, because once the signed order payload
    /// has been accepted by the orderbook, the order cannot be un-submitted.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when required defaults are missing, app-data
    /// generation fails, or downstream signing/submission fails.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?self.trader_defaults.chain_id,
                env = ?self.trader_defaults.env,
                endpoint = "trading.post_limit_order_async",
            ),
        ),
    )]
    pub async fn post_limit_order_async<S>(
        &self,
        mut params: LimitTradeParameters,
        signer: &S,
        advanced_settings: Option<&LimitOrderAdvancedSettings>,
    ) -> Result<crate::OrderPostingResult, TradingError>
    where
        S: AsyncSigner,
        S::Error: std::fmt::Display,
    {
        self.ensure_ready_mode()?;
        params.owner = params.owner.or_else(|| self.trader_defaults.owner.clone());
        let (trader, orderbook) = self.resolve_orderbook_trader(None, params.env)?;

        crate::post::post_limit_order_async_with_bounds(
            &params,
            &trader,
            signer,
            advanced_settings,
            orderbook.client.as_ref(),
            self.order_bounds,
        )
        .await
    }

    /// Builds the pre-sign transaction for an order using a sync signer.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when trader defaults are incomplete or gas
    /// estimation / transaction construction fails.
    pub fn get_pre_sign_transaction<S>(
        &self,
        params: &OrderTraderParameters,
        signer: &S,
    ) -> Result<cow_sdk_core::TransactionRequest, TradingError>
    where
        S: Signer,
        S::Error: std::fmt::Display,
    {
        let (trader, _) = self.resolve_chain_partial_trader(params.chain_id, params.env)?;
        let chain_id = trader
            .chain_id
            .ok_or(TradingError::MissingTraderParameters("chainId"))?;
        let options = protocol_options_for_partial_order(params, &trader);

        get_pre_sign_transaction(signer, chain_id, &params.order_uid, Some(&options))
    }

    /// Builds the pre-sign transaction for an order using an async signer.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when trader defaults are incomplete or gas
    /// estimation / transaction construction fails.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?params.chain_id,
                env = ?params.env,
                endpoint = "trading.get_pre_sign_transaction_async",
                order_uid = params.order_uid.as_str(),
            ),
        ),
    )]
    pub async fn get_pre_sign_transaction_async<S>(
        &self,
        params: &OrderTraderParameters,
        signer: &S,
    ) -> Result<cow_sdk_core::TransactionRequest, TradingError>
    where
        S: AsyncSigner,
        S::Error: std::fmt::Display,
    {
        let (trader, _) = self.resolve_chain_partial_trader(params.chain_id, params.env)?;
        let chain_id = trader
            .chain_id
            .ok_or(TradingError::MissingTraderParameters("chainId"))?;
        let options = protocol_options_for_partial_order(params, &trader);

        get_pre_sign_transaction_async(signer, chain_id, &params.order_uid, Some(&options)).await
    }

    /// Fetches an order from the active orderbook binding.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when chain resolution fails or the orderbook
    /// request fails.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?params.chain_id,
                env = ?params.env,
                endpoint = "trading.get_order",
                order_uid = params.order_uid.as_str(),
            ),
        ),
    )]
    pub async fn get_order(
        &self,
        params: &OrderTraderParameters,
    ) -> Result<cow_sdk_orderbook::Order, TradingError> {
        let (_, orderbook) = self.resolve_chain_partial_trader(params.chain_id, params.env)?;

        orderbook
            .client
            .get_order(&params.order_uid)
            .await
            .map_err(Into::into)
    }

    /// Signs and submits an off-chain cancellation using a sync signer.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site.
    ///
    /// # Errors
    ///
    /// Returns any error from [`Self::off_chain_cancel_order_async`].
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?params.chain_id,
                env = ?params.env,
                endpoint = "trading.off_chain_cancel_order",
                order_uid = params.order_uid.as_str(),
            ),
        ),
    )]
    pub async fn off_chain_cancel_order<S>(
        &self,
        params: &OrderTraderParameters,
        signer: &S,
    ) -> Result<bool, TradingError>
    where
        S: Signer,
        S::Error: std::fmt::Display,
    {
        self.off_chain_cancel_order_async(params, signer).await
    }

    /// Signs and submits an off-chain cancellation using an async signer.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when orderbook context resolution, signing, or
    /// orderbook submission fails.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?params.chain_id,
                env = ?params.env,
                endpoint = "trading.off_chain_cancel_order_async",
                order_uid = params.order_uid.as_str(),
            ),
        ),
    )]
    pub async fn off_chain_cancel_order_async<S>(
        &self,
        params: &OrderTraderParameters,
        signer: &S,
    ) -> Result<bool, TradingError>
    where
        S: AsyncSigner,
        S::Error: std::fmt::Display,
    {
        self.ensure_ready_mode()?;
        let (trader, orderbook) = self.resolve_orderbook_trader(params.chain_id, params.env)?;
        let effective_params = OrderTraderParameters {
            chain_id: Some(orderbook.chain_id),
            env: Some(orderbook.env),
            ..params.clone()
        };

        off_chain_cancel_order_async(
            orderbook.client.as_ref(),
            &effective_params,
            &trader,
            signer,
        )
        .await
    }

    /// Cancels an order on-chain using a sync signer.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site; cancellation
    /// only affects pre-broadcast work, because once a transaction has been
    /// signed and broadcast to the chain, it cannot be withdrawn.
    ///
    /// # Errors
    ///
    /// Returns any error from [`Self::on_chain_cancel_order_async`].
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?params.chain_id,
                env = ?params.env,
                endpoint = "trading.on_chain_cancel_order",
                order_uid = params.order_uid.as_str(),
            ),
        ),
    )]
    pub async fn on_chain_cancel_order<S>(
        &self,
        params: &OrderTraderParameters,
        signer: &S,
    ) -> Result<TransactionHash, TradingError>
    where
        S: Signer,
        S::Error: std::fmt::Display,
    {
        self.on_chain_cancel_order_async(params, signer).await
    }

    /// Cancels an order on-chain using an async signer.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site; cancellation
    /// only affects pre-broadcast work, because once the on-chain cancellation
    /// transaction has been broadcast, it cannot be withdrawn.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when order lookup, transaction construction, or
    /// transaction submission fails.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?params.chain_id,
                env = ?params.env,
                endpoint = "trading.on_chain_cancel_order_async",
                order_uid = params.order_uid.as_str(),
            ),
        ),
    )]
    pub async fn on_chain_cancel_order_async<S>(
        &self,
        params: &OrderTraderParameters,
        signer: &S,
    ) -> Result<TransactionHash, TradingError>
    where
        S: AsyncSigner,
        S::Error: std::fmt::Display,
    {
        let (trader, orderbook) = self.resolve_chain_partial_trader(params.chain_id, params.env)?;

        let order = orderbook.client.get_order(&params.order_uid).await?;

        let effective_params = OrderTraderParameters {
            chain_id: Some(orderbook.chain_id),
            env: Some(orderbook.env),
            ..params.clone()
        };
        let options = protocol_options_for_partial_order(&effective_params, &trader);

        cancel_order_onchain_async(signer, orderbook.chain_id, &order, Some(&options)).await
    }

    /// Reads the `CoW` Protocol allowance using a sync provider.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when trader defaults are incomplete or provider
    /// reads fail.
    pub fn get_cow_protocol_allowance<P>(
        &self,
        provider: &P,
        params: &AllowanceParameters,
    ) -> Result<Amount, TradingError>
    where
        P: Provider,
        P::Error: std::fmt::Display,
    {
        let (trader, _) = self.resolve_chain_partial_trader(params.chain_id, params.env)?;
        let chain_id = trader
            .chain_id
            .ok_or(TradingError::MissingTraderParameters("chainId"))?;
        let env = trader.env.unwrap_or(CowEnv::Prod);

        get_cow_protocol_allowance(
            provider,
            &params.token_address,
            &params.owner,
            chain_id,
            env,
            params.vault_relayer_override.as_ref(),
        )
    }

    /// Reads the `CoW` Protocol allowance using an async provider.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when trader defaults are incomplete or provider
    /// reads fail.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?params.chain_id,
                env = ?params.env,
                endpoint = "trading.get_cow_protocol_allowance_async",
            ),
        ),
    )]
    pub async fn get_cow_protocol_allowance_async<P>(
        &self,
        provider: &P,
        params: &AllowanceParameters,
    ) -> Result<Amount, TradingError>
    where
        P: AsyncProvider,
        P::Error: std::fmt::Display,
    {
        let (trader, _) = self.resolve_chain_partial_trader(params.chain_id, params.env)?;
        let chain_id = trader
            .chain_id
            .ok_or(TradingError::MissingTraderParameters("chainId"))?;
        let env = trader.env.unwrap_or(CowEnv::Prod);

        get_cow_protocol_allowance_async(
            provider,
            &params.token_address,
            &params.owner,
            chain_id,
            env,
            params.vault_relayer_override.as_ref(),
        )
        .await
    }

    /// Sends an approval transaction using a sync signer.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when trader defaults are incomplete or
    /// transaction submission fails.
    pub fn approve_cow_protocol<S>(
        &self,
        signer: &S,
        params: &ApprovalParameters,
    ) -> Result<TransactionHash, TradingError>
    where
        S: Signer,
        S::Error: std::fmt::Display,
    {
        let (trader, _) = self.resolve_chain_partial_trader(params.chain_id, params.env)?;
        let chain_id = trader
            .chain_id
            .ok_or(TradingError::MissingTraderParameters("chainId"))?;
        let env = trader.env.unwrap_or(CowEnv::Prod);

        crate::approve_cow_protocol(signer, params, chain_id, env)
    }

    /// Sends an approval transaction using an async signer.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site; cancellation
    /// only affects pre-broadcast work, because once the approval transaction
    /// has been broadcast, it cannot be withdrawn.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when trader defaults are incomplete or
    /// transaction submission fails.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?params.chain_id,
                env = ?params.env,
                endpoint = "trading.approve_cow_protocol_async",
            ),
        ),
    )]
    pub async fn approve_cow_protocol_async<S>(
        &self,
        signer: &S,
        params: &ApprovalParameters,
    ) -> Result<TransactionHash, TradingError>
    where
        S: AsyncSigner,
        S::Error: std::fmt::Display,
    {
        let (trader, _) = self.resolve_chain_partial_trader(params.chain_id, params.env)?;
        let chain_id = trader
            .chain_id
            .ok_or(TradingError::MissingTraderParameters("chainId"))?;
        let env = trader.env.unwrap_or(CowEnv::Prod);

        crate::approve_cow_protocol_async(signer, params, chain_id, env).await
    }

    fn resolve_quote_owner(
        &self,
        params: &TradeParameters,
        advanced_settings: Option<&SwapAdvancedSettings>,
    ) -> Result<Address, TradingError> {
        advanced_settings
            .and_then(|settings| settings.quote_request.as_ref())
            .and_then(|override_request| override_request.from.clone())
            .or_else(|| params.owner.clone())
            .or_else(|| self.trader_defaults.owner.clone())
            .ok_or(TradingError::MissingOwner)
    }

    fn resolve_quoter(
        &self,
        owner: Address,
        requested_env: Option<CowEnv>,
    ) -> Result<(QuoterParameters, ResolvedOrderbookBinding), TradingError> {
        let app_code = self
            .trader_defaults
            .app_code
            .clone()
            .ok_or(TradingError::MissingQuoterParameters("appCode"))?;
        let orderbook = self.resolve_orderbook_binding(
            self.trader_defaults.chain_id,
            requested_env.or(self.trader_defaults.env),
            TradingError::MissingQuoterParameters("chainId"),
        )?;

        Ok((
            QuoterParameters {
                chain_id: orderbook.chain_id,
                app_code,
                account: owner,
                env: Some(orderbook.env),
                settlement_contract_override: self
                    .trader_defaults
                    .settlement_contract_override
                    .clone(),
                eth_flow_contract_override: self.trader_defaults.eth_flow_contract_override.clone(),
            },
            orderbook,
        ))
    }

    fn resolve_orderbook_trader(
        &self,
        requested_chain: Option<SupportedChainId>,
        requested_env: Option<CowEnv>,
    ) -> Result<(TraderParameters, ResolvedOrderbookBinding), TradingError> {
        let app_code = self
            .trader_defaults
            .app_code
            .clone()
            .ok_or_else(|| TradingError::MissingTraderParameters("chainId, appCode"))?;
        let orderbook = self.resolve_orderbook_binding(
            requested_chain.or(self.trader_defaults.chain_id),
            requested_env.or(self.trader_defaults.env),
            TradingError::MissingTraderParameters("chainId, appCode"),
        )?;

        Ok((
            TraderParameters {
                chain_id: orderbook.chain_id,
                app_code,
                env: Some(orderbook.env),
                settlement_contract_override: self
                    .trader_defaults
                    .settlement_contract_override
                    .clone(),
                eth_flow_contract_override: self.trader_defaults.eth_flow_contract_override.clone(),
            },
            orderbook,
        ))
    }

    fn resolve_chain_partial_trader(
        &self,
        requested_chain: Option<SupportedChainId>,
        requested_env: Option<CowEnv>,
    ) -> Result<(PartialTraderParameters, ResolvedOrderbookBinding), TradingError> {
        let orderbook = self.resolve_orderbook_binding(
            requested_chain.or(self.trader_defaults.chain_id),
            requested_env.or(self.trader_defaults.env),
            TradingError::MissingTraderParameters("chainId"),
        )?;

        Ok((
            PartialTraderParameters {
                chain_id: Some(orderbook.chain_id),
                app_code: self.trader_defaults.app_code.clone(),
                owner: self.trader_defaults.owner.clone(),
                env: Some(orderbook.env),
                settlement_contract_override: self
                    .trader_defaults
                    .settlement_contract_override
                    .clone(),
                eth_flow_contract_override: self.trader_defaults.eth_flow_contract_override.clone(),
            },
            orderbook,
        ))
    }

    fn resolve_orderbook_binding(
        &self,
        requested_chain: Option<SupportedChainId>,
        requested_env: Option<CowEnv>,
        missing_chain_error: TradingError,
    ) -> Result<ResolvedOrderbookBinding, TradingError> {
        if let Some(orderbook_client) = self.options.orderbook_client() {
            validate_orderbook_context(orderbook_client.as_ref(), requested_chain, requested_env)?;
            let context = orderbook_client.context().clone();

            return Ok(ResolvedOrderbookBinding {
                client: orderbook_client,
                chain_id: context.chain_id,
                env: context.env,
            });
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let chain_id = requested_chain.ok_or(missing_chain_error)?;
            let env = requested_env.unwrap_or(CowEnv::Prod);
            let client = OrderBookApi::builder()
                .chain(chain_id)
                .environment(env)
                .build()?;
            Ok(ResolvedOrderbookBinding {
                client: Arc::new(client),
                chain_id,
                env,
            })
        }
        #[cfg(target_arch = "wasm32")]
        {
            // On `wasm32` the typestate builder requires an explicit
            // `HttpTransport`. Browser consumers compose a `FetchTransport`
            // from `cow-sdk-transport-wasm` and inject the resulting
            // [`OrderBookApi`] through
            // [`TradingSdkOptions::with_orderbook_client`].
            let _ = (requested_chain, requested_env);
            Err(missing_chain_error)
        }
    }
}
