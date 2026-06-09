//! `impl Trading` operation methods for the high-level trading facade.
//!
//! Each method resolves trader and orderbook context through the helpers in
//! [`super`] and delegates to the corresponding crate-level free function.

use cow_sdk_core::{Amount, CowEnv, Provider, Signer, TransactionHash};

use super::Trading;
use crate::{
    AllowanceParameters, ApprovalParameters, LimitTradeParameters, OrderTraderParameters,
    QuoteResults, TradeAdvancedSettings, TradeParameters, TradingError, cancel_order_onchain,
    cow_protocol_allowance, off_chain_cancel_order, onchain::protocol_options_for_partial_order,
    pre_sign_transaction, quote_only, quote_results,
};

impl Trading {
    /// Quotes and posts a swap order.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site;
    /// cancellation only affects pre-broadcast work, because once the signed
    /// order payload has been accepted by the orderbook, the order cannot be
    /// un-submitted.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when quoting, signing, app-data upload, or
    /// order submission fails.
    ///
    /// `EthFlow` sell orders require a quote identifier and are routed to the
    /// native-currency transaction path. The
    /// [`swap_params_to_limit_order_params`](crate::swap_params_to_limit_order_params)
    /// bridge produces a [`LimitTradeParametersFromQuote`](crate::LimitTradeParametersFromQuote)
    /// value that guarantees the quote identifier is present, and the
    /// `EthFlow` native-currency submission seam accepts only that newtype.
    /// A `LimitTradeParameters` value constructed without a quote id surfaces
    /// [`TradingError::MissingQuoteId`] at the typed boundary before the
    /// transaction is built.
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
        advanced_settings: Option<&TradeAdvancedSettings>,
    ) -> Result<crate::OrderPostingResult, TradingError>
    where
        S: Signer,
        S::Error: std::fmt::Display + cow_sdk_core::SignerError,
    {
        let (trader, orderbook) = self.resolve_orderbook_trader(None, params.env)?;

        crate::post::post_swap_order(
            &params,
            &trader,
            signer,
            advanced_settings,
            orderbook.client.as_ref(),
        )
        .await
    }

    /// Posts a swap order from previously computed quote results.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site;
    /// cancellation only affects pre-broadcast work, because once the signed
    /// order payload has been accepted by the orderbook, the order cannot be
    /// un-submitted.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when the stored orderbook binding no longer
    /// matches the SDK's active orderbook, when app-data merging fails, when
    /// signing fails, or when the orderbook rejects the submission.
    ///
    /// `EthFlow` sell orders require a quote identifier and are routed to the
    /// native-currency transaction path. The
    /// [`swap_params_to_limit_order_params`](crate::swap_params_to_limit_order_params)
    /// bridge produces a [`LimitTradeParametersFromQuote`](crate::LimitTradeParametersFromQuote)
    /// value that guarantees the quote identifier is present, and the
    /// `EthFlow` native-currency submission seam accepts only that newtype.
    /// A `LimitTradeParameters` value constructed without a quote id surfaces
    /// [`TradingError::MissingQuoteId`] at the typed boundary before the
    /// transaction is built.
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
        advanced_settings: Option<&TradeAdvancedSettings>,
    ) -> Result<crate::OrderPostingResult, TradingError>
    where
        S: Signer,
        S::Error: std::fmt::Display + cow_sdk_core::SignerError,
    {
        let (trader, orderbook) =
            self.resolve_orderbook_trader(None, quote_results.trade_parameters.env)?;

        crate::post::post_swap_order_from_quote(
            quote_results,
            &trader,
            signer,
            advanced_settings,
            orderbook.client.as_ref(),
        )
        .await
    }

    /// Posts a limit order.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site;
    /// cancellation only affects pre-broadcast work, because once the signed
    /// order payload has been accepted by the orderbook, the order cannot be
    /// un-submitted.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when required defaults are missing, app-data
    /// generation fails, or downstream signing/submission fails.
    ///
    /// `EthFlow` sell orders require a quote identifier and are routed to the
    /// native-currency transaction path. The
    /// [`swap_params_to_limit_order_params`](crate::swap_params_to_limit_order_params)
    /// bridge produces a [`LimitTradeParametersFromQuote`](crate::LimitTradeParametersFromQuote)
    /// value that guarantees the quote identifier is present, and the
    /// `EthFlow` native-currency submission seam accepts only that newtype.
    /// A `LimitTradeParameters` value constructed without a quote id surfaces
    /// [`TradingError::MissingQuoteId`] at the typed boundary before the
    /// transaction is built.
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
        advanced_settings: Option<&TradeAdvancedSettings>,
    ) -> Result<crate::OrderPostingResult, TradingError>
    where
        S: Signer,
        S::Error: std::fmt::Display + cow_sdk_core::SignerError,
    {
        let (trader, orderbook) = self.resolve_orderbook_trader(None, params.env)?;

        crate::post::post_limit_order(
            &params,
            &trader,
            signer,
            advanced_settings,
            orderbook.client.as_ref(),
        )
        .await
    }
}

impl Trading {
    /// Fetches quote-only results using SDK defaults plus optional advanced settings.
    ///
    /// Owner precedence: advanced-settings `quote_request.from`, then
    /// call-level [`TradeParameters::owner`]. The SDK does not store a
    /// default owner; missing owner surfaces as
    /// [`TradingError::MissingOwner`].
    ///
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
    pub async fn quote_only(
        &self,
        params: TradeParameters,
        advanced_settings: Option<&TradeAdvancedSettings>,
    ) -> Result<QuoteResults, TradingError> {
        let owner = Self::resolve_quote_owner(&params, advanced_settings)?;
        let (quoter, orderbook) = self.resolve_quoter(owner, params.env)?;

        quote_only(
            &params,
            &quoter,
            advanced_settings,
            orderbook.client.as_ref(),
        )
        .await
    }

    /// Fetches quote results.
    ///
    /// Owner precedence: call-level [`TradeParameters::owner`], then the
    /// signer address resolved through
    /// [`cow_sdk_core::Signer::address`]. The SDK does not store a
    /// default owner.
    ///
    /// Callers that need cooperative cancellation wrap this future
    /// through [`cow_sdk_core::Cancellable::cancel_with`] at the call site.
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
                endpoint = "trading.quote_results",
            ),
        ),
    )]
    pub async fn quote_results<S>(
        &self,
        params: TradeParameters,
        signer: &S,
        advanced_settings: Option<&TradeAdvancedSettings>,
    ) -> Result<QuoteResults, TradingError>
    where
        S: Signer,
        S::Error: std::fmt::Display + cow_sdk_core::SignerError,
    {
        let (trader, orderbook) = self.resolve_orderbook_trader(None, params.env)?;

        quote_results(
            &params,
            &trader,
            signer,
            advanced_settings,
            orderbook.client.as_ref(),
        )
        .await
    }
}

impl Trading {
    /// Signs and submits an off-chain cancellation.
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
                chain = ?params.chain_id.or(self.trader_defaults.chain_id),
                env = ?params.env.or(self.trader_defaults.env),
                endpoint = "trading.off_chain_cancel_order",
                order_uid = %params.order_uid,
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
        S::Error: std::fmt::Display + cow_sdk_core::SignerError,
    {
        let (trader, orderbook) = self.resolve_orderbook_trader(params.chain_id, params.env)?;
        let effective_params = OrderTraderParameters {
            chain_id: Some(orderbook.chain_id),
            env: Some(orderbook.env),
            ..params.clone()
        };

        off_chain_cancel_order(
            orderbook.client.as_ref(),
            &effective_params,
            &trader,
            signer,
        )
        .await
    }

    /// Cancels an order on-chain.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site;
    /// cancellation only affects pre-broadcast work, because once the
    /// on-chain cancellation transaction has been broadcast, it cannot be
    /// withdrawn.
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
                chain = ?params.chain_id.or(self.trader_defaults.chain_id),
                env = ?params.env.or(self.trader_defaults.env),
                endpoint = "trading.on_chain_cancel_order",
                order_uid = %params.order_uid,
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
        S::Error: std::fmt::Display + cow_sdk_core::SignerError,
    {
        let (trader, orderbook) = self.resolve_chain_partial_trader(params.chain_id, params.env)?;

        let order = orderbook.client.order(&params.order_uid).await?;

        let effective_params = OrderTraderParameters {
            chain_id: Some(orderbook.chain_id),
            env: Some(orderbook.env),
            ..params.clone()
        };
        let options = protocol_options_for_partial_order(&effective_params, &trader);

        cancel_order_onchain(signer, orderbook.chain_id, &order, Some(&options)).await
    }
}

impl Trading {
    /// Builds the pre-sign transaction for an order.
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
                endpoint = "trading.pre_sign_transaction",
                order_uid = %params.order_uid,
            ),
        ),
    )]
    pub async fn pre_sign_transaction<S>(
        &self,
        params: &OrderTraderParameters,
        signer: &S,
    ) -> Result<cow_sdk_core::TransactionRequest, TradingError>
    where
        S: Signer,
        S::Error: std::fmt::Display + cow_sdk_core::SignerError,
    {
        let (trader, _) = self.resolve_chain_partial_trader(params.chain_id, params.env)?;
        let chain_id = trader
            .chain_id
            .ok_or(TradingError::MissingTraderParameters("chainId"))?;
        let options = protocol_options_for_partial_order(params, &trader);

        pre_sign_transaction(signer, chain_id, &params.order_uid, Some(&options)).await
    }
}

impl Trading {
    /// Reads the `CoW` Protocol allowance.
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
                endpoint = "trading.cow_protocol_allowance",
            ),
        ),
    )]
    pub async fn cow_protocol_allowance<P>(
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

        cow_protocol_allowance(
            provider,
            &params.token_address,
            &params.owner,
            chain_id,
            env,
            params.vault_relayer_override.as_ref(),
        )
        .await
    }

    /// Sends an approval transaction.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site;
    /// cancellation only affects pre-broadcast work, because once the
    /// approval transaction has been broadcast, it cannot be withdrawn.
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
                endpoint = "trading.approve_cow_protocol",
            ),
        ),
    )]
    pub async fn approve_cow_protocol<S>(
        &self,
        signer: &S,
        params: &ApprovalParameters,
    ) -> Result<TransactionHash, TradingError>
    where
        S: Signer,
        S::Error: std::fmt::Display + cow_sdk_core::SignerError,
    {
        let (trader, _) = self.resolve_chain_partial_trader(params.chain_id, params.env)?;
        let chain_id = trader
            .chain_id
            .ok_or(TradingError::MissingTraderParameters("chainId"))?;
        let env = trader.env.unwrap_or(CowEnv::Prod);

        crate::approve_cow_protocol(signer, params, chain_id, env).await
    }
}

impl Trading {
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
                endpoint = "trading.order",
                order_uid = %params.order_uid,
            ),
        ),
    )]
    pub async fn order(
        &self,
        params: &OrderTraderParameters,
    ) -> Result<cow_sdk_orderbook::Order, TradingError> {
        let (_, orderbook) = self.resolve_chain_partial_trader(params.chain_id, params.env)?;

        orderbook
            .client
            .order(&params.order_uid)
            .await
            .map_err(Into::into)
    }
}
