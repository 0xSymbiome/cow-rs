use std::marker::PhantomData;
use std::sync::Arc;

use cow_sdk_core::{
    Address, Amount, ApiContext, AsyncProvider, AsyncSigner, CowEnv, Provider, Signer,
    SupportedChainId, TransactionHash,
};
use cow_sdk_orderbook::OrderBookApi;

use crate::onchain::protocol_options_for_partial_order;
use crate::{
    AllowanceParameters, ApprovalParameters, LimitOrderAdvancedSettings, LimitTradeParameters,
    OrderTraderParameters, OrderbookClient, PartialTraderParameters, QuoteResults,
    QuoterParameters, SwapAdvancedSettings, TradeParameters, TraderParameters, TradingError,
    TradingSdkOptions, cancel_order_onchain_async, get_cow_protocol_allowance,
    get_cow_protocol_allowance_async, get_pre_sign_transaction, get_pre_sign_transaction_async,
    get_quote_only, get_quote_results_async, off_chain_cancel_order_async, post_limit_order_async,
    post_swap_order_async, post_swap_order_from_quote_async, types::validate_orderbook_context,
};

/// Runtime readiness of a constructed [`TradingSdk`].
///
/// The default `Ready` mode exposes quote, post, and off-chain cancellation
/// flows. `HelperOnly` mode is produced by
/// [`TradingSdkBuilder::build_helper_only`] and restricts those flows so the
/// sdk can only drive chain-bound helpers such as pre-sign transaction
/// construction, allowance reads, approval submission, and on-chain
/// cancellation.
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
#[derive(Debug, Clone, Copy, Default)]
pub struct ChainIdUnset;

/// Typestate marker for a builder that has been given a chain id.
#[derive(Debug, Clone, Copy, Default)]
pub struct ChainIdSet;

/// Typestate marker for a builder that has not yet been given an `appCode`.
#[derive(Debug, Clone, Copy, Default)]
pub struct AppCodeUnset;

/// Typestate marker for a builder that has been given an `appCode`.
#[derive(Debug, Clone, Copy, Default)]
pub struct AppCodeSet;

/// High-level trading facade that stores trader defaults plus optional injected services.
#[derive(Debug, Clone, Default)]
pub struct TradingSdk {
    trader_defaults: PartialTraderParameters,
    options: TradingSdkOptions,
    mode: TradingSdkMode,
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
///
/// The permissive [`TradingSdkBuilder::build`] and
/// [`TradingSdkBuilder::build_partial`] methods remain available on every
/// state and preserve the runtime-validated construction path for the
/// migration window.
#[derive(Debug, Clone)]
pub struct TradingSdkBuilder<C = ChainIdUnset, A = AppCodeUnset> {
    trader_defaults: PartialTraderParameters,
    options: TradingSdkOptions,
    _state: PhantomData<(C, A)>,
}

impl Default for TradingSdkBuilder<ChainIdUnset, AppCodeUnset> {
    fn default() -> Self {
        Self {
            trader_defaults: PartialTraderParameters::default(),
            options: TradingSdkOptions::default(),
            _state: PhantomData,
        }
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

    fn validate_ready_defaults(&self) -> Result<(), TradingError> {
        let mut missing = Vec::new();

        if self.options.orderbook_client().is_none() && self.trader_defaults.chain_id.is_none() {
            missing.push("chainId");
        }
        if self.trader_defaults.app_code.is_none() {
            missing.push("appCode");
        }

        if missing.is_empty() {
            Ok(())
        } else {
            Err(TradingError::MissingTraderParameters(missing.join(", ")))
        }
    }

    /// Builds a partially configured [`TradingSdk`] and validates any injected
    /// orderbook binding.
    ///
    /// Use this when the SDK is only being prepared for chain-bound helper
    /// flows such as allowance, approval, pre-sign, or on-chain cancellation.
    /// Quote, post, and off-chain cancellation helpers still validate
    /// `appCode` when those workflows are used. The returned SDK reports
    /// [`TradingSdkMode::Ready`] so runtime gating stays opt-in through
    /// [`TradingSdkBuilder::build_helper_only`].
    ///
    /// # Errors
    ///
    /// Returns [`TradingError::InjectedOrderbookContextConflict`] when the
    /// builder's default chain or environment conflicts with an injected
    /// orderbook client.
    pub fn build_partial(self) -> Result<TradingSdk, TradingError> {
        self.validate_injected_orderbook_binding()?;

        Ok(TradingSdk {
            trader_defaults: self.trader_defaults,
            options: self.options,
            mode: TradingSdkMode::Ready,
        })
    }

    /// Builds a ready-state [`TradingSdk`] with runtime validation of the
    /// trader defaults.
    ///
    /// This is the permissive construction path preserved for the migration
    /// window and is available on every builder state. New code should prefer
    /// the compile-time-checked [`TradingSdkBuilder::build_ready`] instead.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError::InjectedOrderbookContextConflict`] when the
    /// builder's default chain or environment conflicts with an injected
    /// orderbook client, or [`TradingError::MissingTraderParameters`] when the
    /// defaults do not provide `appCode` plus either a default `chainId` or an
    /// injected orderbook client that fixes chain authority.
    pub fn build(self) -> Result<TradingSdk, TradingError> {
        self.validate_injected_orderbook_binding()?;
        self.validate_ready_defaults()?;
        self.build_partial()
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
        })
    }
}

impl TradingSdkBuilder<ChainIdSet, AppCodeSet> {
    /// Builds a fully-configured ready-state [`TradingSdk`].
    ///
    /// The compile-time typestate guarantees that both chain id and app code
    /// have been supplied before this terminal runs, so the only remaining
    /// runtime validation is the injected orderbook binding. Attempting to
    /// call `build_ready` on a builder that does not own those prerequisites
    /// is a compile error. Use [`TradingSdkBuilder::build`] for the permissive
    /// runtime-validated alternative.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError::InjectedOrderbookContextConflict`] when the
    /// builder's default chain or environment conflicts with an injected
    /// orderbook client.
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

        Ok(TradingSdk {
            trader_defaults: self.trader_defaults,
            options: self.options,
            mode: TradingSdkMode::Ready,
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

    /// Creates a ready-state SDK directly from defaults and options.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError::InjectedOrderbookContextConflict`] when the
    /// supplied defaults conflict with an injected orderbook client, or
    /// [`TradingError::MissingTraderParameters`] when the defaults do not
    /// provide `appCode` plus either a default `chainId` or an injected
    /// orderbook client that fixes chain authority.
    pub fn new(
        trader_defaults: PartialTraderParameters,
        options: TradingSdkOptions,
    ) -> Result<Self, TradingError> {
        TradingSdkBuilder::new()
            .with_trader_defaults(trader_defaults)
            .with_options(options)
            .build()
    }

    /// Creates a partially configured SDK directly from defaults and options.
    ///
    /// This constructor is intended for chain-bound helper flows that do not
    /// require quote or submission attribution, such as allowance reads,
    /// approval submission, pre-sign transaction construction, or on-chain
    /// cancellation.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError::InjectedOrderbookContextConflict`] when the
    /// supplied defaults conflict with an injected orderbook client.
    pub fn new_partial(
        trader_defaults: PartialTraderParameters,
        options: TradingSdkOptions,
    ) -> Result<Self, TradingError> {
        TradingSdkBuilder::new()
            .with_trader_defaults(trader_defaults)
            .with_options(options)
            .build_partial()
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

        post_swap_order_async(
            &params,
            &trader,
            signer,
            advanced_settings,
            orderbook.client.as_ref(),
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

        post_swap_order_from_quote_async(
            quote_results,
            &trader,
            signer,
            advanced_settings,
            orderbook.client.as_ref(),
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

        post_limit_order_async(
            &params,
            &trader,
            signer,
            advanced_settings,
            orderbook.client.as_ref(),
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
            .ok_or_else(|| TradingError::MissingTraderParameters("chainId".to_owned()))?;
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
            .ok_or_else(|| TradingError::MissingTraderParameters("chainId".to_owned()))?;
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
            .ok_or_else(|| TradingError::MissingTraderParameters("chainId".to_owned()))?;
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
            .ok_or_else(|| TradingError::MissingTraderParameters("chainId".to_owned()))?;
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
            .ok_or_else(|| TradingError::MissingTraderParameters("chainId".to_owned()))?;
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
            .ok_or_else(|| TradingError::MissingTraderParameters("chainId".to_owned()))?;
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
            .ok_or_else(|| TradingError::MissingQuoterParameters("appCode".to_owned()))?;
        let orderbook = self.resolve_orderbook_binding(
            self.trader_defaults.chain_id,
            requested_env.or(self.trader_defaults.env),
            TradingError::MissingQuoterParameters("chainId".to_owned()),
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
        let app_code =
            self.trader_defaults.app_code.clone().ok_or_else(|| {
                TradingError::MissingTraderParameters("chainId, appCode".to_owned())
            })?;
        let orderbook = self.resolve_orderbook_binding(
            requested_chain.or(self.trader_defaults.chain_id),
            requested_env.or(self.trader_defaults.env),
            TradingError::MissingTraderParameters("chainId, appCode".to_owned()),
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
            TradingError::MissingTraderParameters("chainId".to_owned()),
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

        let chain_id = requested_chain.ok_or(missing_chain_error)?;
        let env = requested_env.unwrap_or(CowEnv::Prod);

        Ok(ResolvedOrderbookBinding {
            client: Arc::new(OrderBookApi::new(ApiContext::new(chain_id, env))),
            chain_id,
            env,
        })
    }
}
