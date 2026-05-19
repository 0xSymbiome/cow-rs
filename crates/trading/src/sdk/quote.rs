use cow_sdk_core::{AsyncSigner, Signer};

use super::TradingSdk;
use crate::{
    QuoteResults, SwapAdvancedSettings, TradeParameters, TradingError, get_quote_only,
    get_quote_results_async,
};

impl TradingSdk {
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
        params.owner = params.owner.or(self.trader_defaults.owner);
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
        S::Error: std::fmt::Display + cow_sdk_core::SignerError,
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
        S::Error: std::fmt::Display + cow_sdk_core::SignerError,
    {
        params.owner = params.owner.or(self.trader_defaults.owner);
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
}
