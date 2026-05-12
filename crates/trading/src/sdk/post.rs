use cow_sdk_core::{AsyncSigner, Signer};

use super::TradingSdk;
use crate::{
    LimitOrderAdvancedSettings, LimitTradeParameters, QuoteResults, SwapAdvancedSettings,
    TradeParameters, TradingError,
};

impl TradingSdk {
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
    ///
    /// `EthFlow` sell orders require a quote identifier and are routed to the
    /// native-currency transaction path. Propagate the orderbook quote id with
    /// `with_quote_id(quote.id)`; otherwise the method returns
    /// [`TradingError::MissingQuoteId`] before building the transaction.
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
    ///
    /// `EthFlow` sell orders require a quote identifier and are routed to the
    /// native-currency transaction path. Propagate the orderbook quote id with
    /// `with_quote_id(quote.id)`; otherwise the method returns
    /// [`TradingError::MissingQuoteId`] before building the transaction.
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
    ///
    /// `EthFlow` sell orders require a quote identifier and are routed to the
    /// native-currency transaction path. Propagate the orderbook quote id with
    /// `with_quote_id(quote.id)`; otherwise the method returns
    /// [`TradingError::MissingQuoteId`] before building the transaction.
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
}
