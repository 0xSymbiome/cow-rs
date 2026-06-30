//! `impl Trading` operation methods for the high-level trading facade.
//!
//! Each method resolves trader and orderbook context through the helpers in
//! [`super`] and delegates to the corresponding crate-level free function.

use cow_sdk_core::{
    Address, Amount, CowEnv, Hash32, HexData, OrderUid, ProtocolOptions, Provider, Signer,
    TransactionHash,
};

use super::Trading;
use crate::{
    AllowanceParams, ApprovalParams, Authorization, LimitTradeParams, OrderPlacement,
    OrderTraderParams, PreparedTransaction, QuoteResults, SafeActivation, TradeAdvancedSettings,
    TradeParams, TradingError, build_presign_activation, cow_protocol_allowance,
    offchain_cancel_order, onchain::protocol_options_for_partial_order, onchain_cancel_order,
    place_limit, place_swap, pre_sign_transaction, preflight_eip1271, quote_only, quote_results,
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
    /// bridge produces a [`LimitTradeParamsFromQuote`](crate::LimitTradeParamsFromQuote)
    /// value that guarantees the quote identifier is present, and the
    /// `EthFlow` native-currency submission seam accepts only that newtype.
    /// A `LimitTradeParams` value constructed without a quote id surfaces
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
        params: TradeParams,
        signer: &S,
        advanced_settings: Option<&TradeAdvancedSettings>,
    ) -> Result<crate::OrderPostingResult, TradingError>
    where
        S: Signer,
        S::Error: std::fmt::Display + cow_sdk_core::UserRejection,
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
    /// bridge produces a [`LimitTradeParamsFromQuote`](crate::LimitTradeParamsFromQuote)
    /// value that guarantees the quote identifier is present, and the
    /// `EthFlow` native-currency submission seam accepts only that newtype.
    /// A `LimitTradeParams` value constructed without a quote id surfaces
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
        S::Error: std::fmt::Display + cow_sdk_core::UserRejection,
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
    /// bridge produces a [`LimitTradeParamsFromQuote`](crate::LimitTradeParamsFromQuote)
    /// value that guarantees the quote identifier is present, and the
    /// `EthFlow` native-currency submission seam accepts only that newtype.
    /// A `LimitTradeParams` value constructed without a quote id surfaces
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
        params: LimitTradeParams,
        signer: &S,
        advanced_settings: Option<&TradeAdvancedSettings>,
    ) -> Result<crate::OrderPostingResult, TradingError>
    where
        S: Signer,
        S::Error: std::fmt::Display + cow_sdk_core::UserRejection,
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

    /// Posts a limit order under the pre-sign scheme without consulting a
    /// signer.
    ///
    /// Pre-sign placements carry no cryptographic signature: the order is
    /// submitted with [`cow_sdk_orderbook::SigningScheme::PreSign`] (the wire
    /// `signature` field carries the owner address, mirroring the reviewed
    /// upstream SDK) and only becomes fillable once the owner sets the
    /// on-chain pre-signature flag via `setPreSignature` on the settlement
    /// contract — for example by submitting the transaction built by
    /// [`Trading::pre_sign_transaction`]. This is the smart-contract-owner
    /// path: Safes and other smart accounts place the order off-chain first
    /// and approve it on-chain from the contract itself.
    ///
    /// Because no signer participates, the owner must be explicit:
    /// [`LimitTradeParams::owner`] (or an advanced-settings
    /// `quote_request.from` override) is required. Any signing-scheme
    /// override carried in `advanced_settings` is superseded by the pre-sign
    /// scheme.
    ///
    /// Callers that need cooperative cancellation wrap this future through
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site;
    /// cancellation only affects pre-broadcast work, because once the order
    /// payload has been accepted by the orderbook, the order cannot be
    /// un-submitted.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError::MissingSubmissionOwner`] when no explicit
    /// owner is supplied, [`TradingError::InvalidInput`] for native-currency
    /// sell orders (`EthFlow` orders are created on-chain and need a
    /// signer-backed entry), and otherwise [`TradingError`] when required
    /// defaults are missing, app-data generation fails, or submission fails.
    #[cfg_attr(
        feature = "tracing",
        tracing::instrument(
            skip_all,
            fields(
                chain = ?self.trader_defaults.chain_id,
                env = ?self.trader_defaults.env,
                endpoint = "trading.post_limit_order_presign",
            ),
        ),
    )]
    pub async fn post_limit_order_presign(
        &self,
        params: LimitTradeParams,
        advanced_settings: Option<&TradeAdvancedSettings>,
    ) -> Result<crate::OrderPostingResult, TradingError> {
        let (trader, orderbook) = self.resolve_orderbook_trader(None, params.env)?;

        crate::post::post_limit_order_presign(
            &params,
            &trader,
            advanced_settings,
            orderbook.client.as_ref(),
        )
        .await
    }
}

impl Trading {
    /// Posts a swap order from quote results, selecting the signing path from
    /// `auth` and returning the typed [`OrderPlacement`] sum (ADR 0073).
    ///
    /// [`Authorization::Ecdsa`] and [`Authorization::Eip1271`] resolve to
    /// [`OrderPlacement::Live`]; [`Authorization::PreSign`] resolves to
    /// [`OrderPlacement::PendingActivation`] carrying the on-chain
    /// approve-then-set-pre-signature bundle the owner must send or propose from
    /// the smart account.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when the stored orderbook binding no longer
    /// matches, when signing fails, when the orderbook rejects the submission,
    /// or when the pre-sign activation cannot be built.
    pub async fn place_swap<S>(
        &self,
        quote_results: &QuoteResults,
        owner: Address,
        auth: Authorization<'_, S>,
        advanced_settings: Option<&TradeAdvancedSettings>,
    ) -> Result<OrderPlacement, TradingError>
    where
        S: Signer,
        S::Error: std::fmt::Display + cow_sdk_core::UserRejection,
    {
        let (trader, orderbook) =
            self.resolve_orderbook_trader(None, quote_results.trade_parameters.env)?;

        place_swap(
            quote_results,
            owner,
            auth,
            &trader,
            advanced_settings,
            orderbook.client.as_ref(),
        )
        .await
    }

    /// Posts a limit order, selecting the signing path from `auth` and returning
    /// the typed [`OrderPlacement`] sum (ADR 0073).
    ///
    /// [`Authorization::Ecdsa`] and [`Authorization::Eip1271`] resolve to
    /// [`OrderPlacement::Live`]; [`Authorization::PreSign`] resolves to
    /// [`OrderPlacement::PendingActivation`] carrying the on-chain
    /// approve-then-set-pre-signature bundle the owner must send or propose from
    /// the smart account.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when required defaults are missing, when app-data
    /// generation or signing fails, when the orderbook rejects the submission,
    /// or when the pre-sign activation cannot be built.
    pub async fn place_limit<S>(
        &self,
        params: LimitTradeParams,
        owner: Address,
        auth: Authorization<'_, S>,
        advanced_settings: Option<&TradeAdvancedSettings>,
    ) -> Result<OrderPlacement, TradingError>
    where
        S: Signer,
        S::Error: std::fmt::Display + cow_sdk_core::UserRejection,
    {
        let (trader, orderbook) = self.resolve_orderbook_trader(None, params.env)?;

        place_limit(
            params,
            owner,
            auth,
            &trader,
            advanced_settings,
            orderbook.client.as_ref(),
        )
        .await
    }

    /// Builds the unsigned limit order (`order_to_sign`) and its app-data the
    /// posting path would produce for these inputs, without contacting a signer
    /// or the orderbook.
    ///
    /// Resolves trader defaults the same way [`Trading::place_limit`] does, then
    /// delegates to [`build_limit_order_to_sign`](crate::build_limit_order_to_sign).
    /// The returned [`OrderData`](cow_sdk_core::OrderData) is the digest a smart
    /// account signs for an EIP-1271 limit order, so a caller can resolve a
    /// contract signature against it before handing the order to
    /// [`Trading::place_limit`] with
    /// [`Authorization::Eip1271`](crate::Authorization::Eip1271), which rebuilds
    /// the same order and echoes the resolved signature.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when trader defaults are missing or when app-data
    /// generation or order construction fails.
    pub async fn build_limit_order_to_sign(
        &self,
        params: &LimitTradeParams,
        owner: Address,
        advanced_settings: Option<&TradeAdvancedSettings>,
    ) -> Result<(cow_sdk_core::OrderData, crate::TradingAppDataInfo), TradingError> {
        let (trader, _orderbook) = self.resolve_orderbook_trader(None, params.env)?;

        crate::build_limit_order_to_sign(params, owner, &trader, advanced_settings).await
    }

    /// Builds the on-chain activation bundle for an already-posted pre-sign
    /// order (ADR 0073).
    ///
    /// Resolves the chain and environment from trader defaults, then composes
    /// the ordered approve-then-set-pre-signature pair via
    /// [`build_presign_activation`]. Pure: it reads no on-chain allowance and
    /// always emits both calls.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError::MissingTraderParams`] when no chain default is
    /// configured, or [`TradingError::Contracts`] when no settlement deployment
    /// is registered for the chain/environment.
    pub fn build_presign_activation(
        &self,
        order_uid: &OrderUid,
        sell_token: Address,
        amount: Amount,
    ) -> Result<SafeActivation, TradingError> {
        let chain_id = self
            .trader_defaults
            .chain_id
            .ok_or(TradingError::MissingTraderParams("chainId"))?;
        let mut options =
            ProtocolOptions::new().with_env(self.trader_defaults.env.unwrap_or(CowEnv::Prod));
        if let Some(overrides) = self.trader_defaults.settlement_contract_override.as_ref() {
            options = options.with_settlement_contract_override(overrides.clone());
        }

        build_presign_activation(order_uid, sell_token, amount, chain_id, Some(&options))
    }

    /// Verifies a smart-contract-wallet EIP-1271 signature on-chain and returns
    /// the real verdict (ADR 0073).
    ///
    /// # Errors
    ///
    /// Returns [`TradingError::Contracts`] when the verifier has no code, the
    /// provider call fails, or the response does not match the EIP-1271 magic
    /// value.
    pub async fn preflight_eip1271<P>(
        &self,
        provider: &P,
        owner: Address,
        digest: Hash32,
        signature: HexData,
    ) -> Result<(), TradingError>
    where
        P: Provider,
        P::Error: std::fmt::Display,
    {
        preflight_eip1271(provider, owner, digest, signature).await
    }
}

impl Trading {
    /// Fetches quote-only results using SDK defaults plus optional advanced settings.
    ///
    /// Owner precedence: advanced-settings `quote_request.from`, then
    /// call-level [`TradeParams::owner`]. The SDK does not store a
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
        params: TradeParams,
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
    /// Owner precedence: call-level [`TradeParams::owner`], then the
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
        params: TradeParams,
        signer: &S,
        advanced_settings: Option<&TradeAdvancedSettings>,
    ) -> Result<QuoteResults, TradingError>
    where
        S: Signer,
        S::Error: std::fmt::Display + cow_sdk_core::UserRejection,
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
                endpoint = "trading.offchain_cancel_order",
                order_uid = %params.order_uid,
            ),
        ),
    )]
    pub async fn offchain_cancel_order<S>(
        &self,
        params: &OrderTraderParams,
        signer: &S,
    ) -> Result<bool, TradingError>
    where
        S: Signer,
        S::Error: std::fmt::Display + cow_sdk_core::UserRejection,
    {
        let (trader, orderbook) = self.resolve_orderbook_trader(params.chain_id, params.env)?;
        let effective_params = OrderTraderParams {
            chain_id: Some(orderbook.chain_id),
            env: Some(orderbook.env),
            ..params.clone()
        };

        offchain_cancel_order(
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
                endpoint = "trading.onchain_cancel_order",
                order_uid = %params.order_uid,
            ),
        ),
    )]
    pub async fn onchain_cancel_order<S>(
        &self,
        params: &OrderTraderParams,
        signer: &S,
    ) -> Result<TransactionHash, TradingError>
    where
        S: Signer,
        S::Error: std::fmt::Display + cow_sdk_core::UserRejection,
    {
        let (trader, orderbook) = self.resolve_chain_partial_trader(params.chain_id, params.env)?;

        let order = orderbook.client.order(&params.order_uid).await?;

        let effective_params = OrderTraderParams {
            chain_id: Some(orderbook.chain_id),
            env: Some(orderbook.env),
            ..params.clone()
        };
        let options = protocol_options_for_partial_order(&effective_params, &trader);

        onchain_cancel_order(signer, orderbook.chain_id, &order, Some(&options)).await
    }
}

impl Trading {
    /// Builds the pre-sign transaction for an order.
    ///
    /// The returned [`PreparedTransaction`] carries every field set; convert
    /// it with `.into()` when a submission seam expects a
    /// [`cow_sdk_core::TransactionRequest`].
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
        params: &OrderTraderParams,
        signer: &S,
    ) -> Result<PreparedTransaction, TradingError>
    where
        S: Signer,
        S::Error: std::fmt::Display + cow_sdk_core::UserRejection,
    {
        let (trader, _) = self.resolve_chain_partial_trader(params.chain_id, params.env)?;
        let chain_id = trader
            .chain_id
            .ok_or(TradingError::MissingTraderParams("chainId"))?;
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
        params: &AllowanceParams,
    ) -> Result<Amount, TradingError>
    where
        P: Provider,
        P::Error: std::fmt::Display,
    {
        let (trader, _) = self.resolve_chain_partial_trader(params.chain_id, params.env)?;
        let chain_id = trader
            .chain_id
            .ok_or(TradingError::MissingTraderParams("chainId"))?;
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
        params: &ApprovalParams,
    ) -> Result<TransactionHash, TradingError>
    where
        S: Signer,
        S::Error: std::fmt::Display + cow_sdk_core::UserRejection,
    {
        let (trader, _) = self.resolve_chain_partial_trader(params.chain_id, params.env)?;
        let chain_id = trader
            .chain_id
            .ok_or(TradingError::MissingTraderParams("chainId"))?;
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
        params: &OrderTraderParams,
    ) -> Result<cow_sdk_orderbook::Order, TradingError> {
        let (_, orderbook) = self.resolve_chain_partial_trader(params.chain_id, params.env)?;

        orderbook
            .client
            .order(&params.order_uid)
            .await
            .map_err(Into::into)
    }
}
