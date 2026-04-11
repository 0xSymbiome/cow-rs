use std::sync::Arc;

use cow_sdk_core::{
    Address, Amount, ApiContext, AsyncProvider, AsyncSigner, CowEnv, Provider, Signer,
    SupportedChainId, TransactionHash,
};
use cow_sdk_orderbook::OrderBookApi;

use crate::{
    AllowanceParameters, ApprovalParameters, LimitOrderAdvancedSettings, LimitTradeParameters,
    OrderTraderParameters, OrderbookClient, PartialTraderParameters, QuoteResults,
    QuoterParameters, SwapAdvancedSettings, TradeParameters, TraderParameters, TradingError,
    TradingSdkOptions, cancel_order_onchain_async, get_cow_protocol_allowance,
    get_cow_protocol_allowance_async, get_pre_sign_transaction, get_pre_sign_transaction_async,
    get_quote_only, get_quote_results_async, off_chain_cancel_order_async, post_limit_order_async,
    post_swap_order_async, protocol_options_for_order,
};

/// High-level trading facade that stores trader defaults plus optional injected services.
#[derive(Debug, Clone, Default)]
pub struct TradingSdk {
    trader_defaults: PartialTraderParameters,
    options: TradingSdkOptions,
}

/// Builder for [`TradingSdk`].
#[derive(Debug, Clone, Default)]
pub struct TradingSdkBuilder {
    trader_defaults: PartialTraderParameters,
    options: TradingSdkOptions,
}

#[derive(Clone)]
struct ResolvedOrderbookBinding {
    client: Arc<dyn OrderbookClient>,
    chain_id: SupportedChainId,
    env: CowEnv,
}

impl TradingSdkBuilder {
    /// Creates a new builder with empty defaults.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a copy of this builder with trader defaults replaced.
    #[must_use]
    pub fn with_trader_defaults(mut self, trader_defaults: PartialTraderParameters) -> Self {
        self.trader_defaults = trader_defaults;
        self
    }

    /// Returns a copy of this builder with a default chain id.
    #[must_use]
    pub fn with_chain_id(mut self, chain_id: SupportedChainId) -> Self {
        self.trader_defaults.chain_id = Some(chain_id);
        self
    }

    /// Returns a copy of this builder with a default app code.
    #[must_use]
    pub fn with_app_code(mut self, app_code: impl Into<String>) -> Self {
        self.trader_defaults.app_code = Some(app_code.into());
        self
    }

    /// Returns a copy of this builder with a default owner.
    #[must_use]
    pub fn with_owner(mut self, owner: Address) -> Self {
        self.trader_defaults.owner = Some(owner);
        self
    }

    /// Returns a copy of this builder with a default environment.
    #[must_use]
    pub fn with_env(mut self, env: CowEnv) -> Self {
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

    /// Returns a copy of this builder with EthFlow contract overrides.
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

    /// Builds a [`TradingSdk`] and validates any injected orderbook binding.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError::InjectedOrderbookContextConflict`] when the
    /// builder's default chain or environment conflicts with an injected
    /// orderbook client.
    pub fn build(self) -> Result<TradingSdk, TradingError> {
        if let Some(orderbook_client) = self.options.orderbook_client() {
            validate_injected_orderbook_context(
                orderbook_client.as_ref(),
                self.trader_defaults.chain_id,
                self.trader_defaults.env,
            )?;
        }

        Ok(TradingSdk {
            trader_defaults: self.trader_defaults,
            options: self.options,
        })
    }
}

impl TradingSdk {
    /// Returns a new [`TradingSdkBuilder`].
    #[must_use]
    pub fn builder() -> TradingSdkBuilder {
        TradingSdkBuilder::new()
    }

    /// Creates an SDK directly from defaults and options.
    #[must_use]
    pub fn new(trader_defaults: PartialTraderParameters, options: TradingSdkOptions) -> Self {
        Self {
            trader_defaults,
            options,
        }
    }

    /// Returns the stored trader defaults.
    #[must_use]
    pub fn trader_defaults(&self) -> &PartialTraderParameters {
        &self.trader_defaults
    }

    /// Returns the stored SDK options.
    #[must_use]
    pub fn options(&self) -> &TradingSdkOptions {
        &self.options
    }

    /// Fetches quote-only results using SDK defaults plus optional advanced settings.
    ///
    /// Owner precedence is: quote override `from`, call-level `owner`, SDK default `owner`.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when required defaults are missing, the quote
    /// request is invalid, or downstream quote construction fails.
    pub async fn get_quote_only(
        &self,
        mut params: TradeParameters,
        advanced_settings: Option<&SwapAdvancedSettings>,
    ) -> Result<QuoteResults, TradingError> {
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
    /// # Errors
    ///
    /// Returns any error from [`Self::get_quote_results_async`].
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
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when required defaults are missing, signer
    /// address resolution fails, or downstream quote construction fails.
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
    /// # Errors
    ///
    /// Returns any error from [`Self::post_swap_order_async`].
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
    /// # Errors
    ///
    /// Returns [`TradingError`] when quoting, signing, app-data upload, or
    /// order submission fails.
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

        post_swap_order_async(
            &params,
            &trader,
            signer,
            advanced_settings,
            orderbook.client.as_ref(),
        )
        .await
    }

    /// Posts a limit order using a sync signer.
    ///
    /// # Errors
    ///
    /// Returns any error from [`Self::post_limit_order_async`].
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
    /// # Errors
    ///
    /// Returns [`TradingError`] when required defaults are missing, app-data
    /// generation fails, or downstream signing/submission fails.
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
        let trader = self.resolve_trader()?;
        let chain_id = params
            .chain_id
            .or(Some(trader.chain_id))
            .ok_or_else(|| TradingError::MissingTraderParameters("chainId".to_owned()))?;
        let options = protocol_options_for_order(params, &trader);

        get_pre_sign_transaction(signer, chain_id, &params.order_uid, Some(&options))
    }

    /// Builds the pre-sign transaction for an order using an async signer.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when trader defaults are incomplete or gas
    /// estimation / transaction construction fails.
    pub async fn get_pre_sign_transaction_async<S>(
        &self,
        params: &OrderTraderParameters,
        signer: &S,
    ) -> Result<cow_sdk_core::TransactionRequest, TradingError>
    where
        S: AsyncSigner,
        S::Error: std::fmt::Display,
    {
        let trader = self.resolve_trader()?;
        let chain_id = params
            .chain_id
            .or(Some(trader.chain_id))
            .ok_or_else(|| TradingError::MissingTraderParameters("chainId".to_owned()))?;
        let options = protocol_options_for_order(params, &trader);

        get_pre_sign_transaction_async(signer, chain_id, &params.order_uid, Some(&options)).await
    }

    /// Fetches an order from the active orderbook binding.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when chain resolution fails or the orderbook
    /// request fails.
    pub async fn get_order(
        &self,
        params: &OrderTraderParameters,
    ) -> Result<cow_sdk_orderbook::Order, TradingError> {
        let (_, orderbook) = self.resolve_orderbook_partial_trader(params.chain_id, params.env)?;

        orderbook
            .client
            .get_order(&params.order_uid)
            .await
            .map_err(Into::into)
    }

    /// Signs and submits an off-chain cancellation using a sync signer.
    ///
    /// # Errors
    ///
    /// Returns any error from [`Self::off_chain_cancel_order_async`].
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
    /// # Errors
    ///
    /// Returns [`TradingError`] when orderbook context resolution, signing, or
    /// orderbook submission fails.
    pub async fn off_chain_cancel_order_async<S>(
        &self,
        params: &OrderTraderParameters,
        signer: &S,
    ) -> Result<bool, TradingError>
    where
        S: AsyncSigner,
        S::Error: std::fmt::Display,
    {
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
    /// # Errors
    ///
    /// Returns any error from [`Self::on_chain_cancel_order_async`].
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
    /// # Errors
    ///
    /// Returns [`TradingError`] when order lookup, transaction construction, or
    /// transaction submission fails.
    pub async fn on_chain_cancel_order_async<S>(
        &self,
        params: &OrderTraderParameters,
        signer: &S,
    ) -> Result<TransactionHash, TradingError>
    where
        S: AsyncSigner,
        S::Error: std::fmt::Display,
    {
        let (_trader, orderbook) =
            self.resolve_orderbook_partial_trader(params.chain_id, params.env)?;
        let order = orderbook.client.get_order(&params.order_uid).await?;
        let effective_params = OrderTraderParameters {
            chain_id: Some(orderbook.chain_id),
            env: Some(orderbook.env),
            ..params.clone()
        };
        let options = protocol_options_for_order(
            &effective_params,
            &TraderParameters {
                chain_id: orderbook.chain_id,
                app_code: self.trader_defaults.app_code.clone().unwrap_or_default(),
                env: Some(orderbook.env),
                settlement_contract_override: self
                    .trader_defaults
                    .settlement_contract_override
                    .clone(),
                eth_flow_contract_override: self.trader_defaults.eth_flow_contract_override.clone(),
            },
        );

        cancel_order_onchain_async(signer, orderbook.chain_id, &order, Some(&options)).await
    }

    /// Reads the CoW Protocol allowance using a sync provider.
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
        let trader = self.resolve_trader()?;
        let chain_id = params.chain_id.unwrap_or(trader.chain_id);
        let env = params.env.or(trader.env).unwrap_or(CowEnv::Prod);

        get_cow_protocol_allowance(
            provider,
            &params.token_address,
            &params.owner,
            chain_id,
            env,
            params.vault_relayer_address.as_ref(),
        )
    }

    /// Reads the CoW Protocol allowance using an async provider.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when trader defaults are incomplete or provider
    /// reads fail.
    pub async fn get_cow_protocol_allowance_async<P>(
        &self,
        provider: &P,
        params: &AllowanceParameters,
    ) -> Result<Amount, TradingError>
    where
        P: AsyncProvider,
        P::Error: std::fmt::Display,
    {
        let trader = self.resolve_trader()?;
        let chain_id = params.chain_id.unwrap_or(trader.chain_id);
        let env = params.env.or(trader.env).unwrap_or(CowEnv::Prod);

        get_cow_protocol_allowance_async(
            provider,
            &params.token_address,
            &params.owner,
            chain_id,
            env,
            params.vault_relayer_address.as_ref(),
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
        let trader = self.resolve_trader()?;
        let chain_id = params.chain_id.unwrap_or(trader.chain_id);
        let env = params.env.or(trader.env).unwrap_or(CowEnv::Prod);

        crate::approve_cow_protocol(signer, params, chain_id, env)
    }

    /// Sends an approval transaction using an async signer.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when trader defaults are incomplete or
    /// transaction submission fails.
    pub async fn approve_cow_protocol_async<S>(
        &self,
        signer: &S,
        params: &ApprovalParameters,
    ) -> Result<TransactionHash, TradingError>
    where
        S: AsyncSigner,
        S::Error: std::fmt::Display,
    {
        let trader = self.resolve_trader()?;
        let chain_id = params.chain_id.unwrap_or(trader.chain_id);
        let env = params.env.or(trader.env).unwrap_or(CowEnv::Prod);

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

    fn resolve_trader(&self) -> Result<TraderParameters, TradingError> {
        let chain_id = self
            .trader_defaults
            .chain_id
            .ok_or_else(|| TradingError::MissingTraderParameters("chainId, appCode".to_owned()))?;
        let app_code =
            self.trader_defaults.app_code.clone().ok_or_else(|| {
                TradingError::MissingTraderParameters("chainId, appCode".to_owned())
            })?;

        Ok(TraderParameters {
            chain_id,
            app_code,
            env: self.trader_defaults.env,
            settlement_contract_override: self.trader_defaults.settlement_contract_override.clone(),
            eth_flow_contract_override: self.trader_defaults.eth_flow_contract_override.clone(),
        })
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

    fn resolve_orderbook_partial_trader(
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
            validate_injected_orderbook_context(
                orderbook_client.as_ref(),
                requested_chain,
                requested_env,
            )?;
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
            client: Arc::new(OrderBookApi::new(ApiContext {
                chain_id,
                env,
                base_urls: None,
                api_key: None,
            })),
            chain_id,
            env,
        })
    }
}

fn validate_injected_orderbook_context(
    orderbook_client: &dyn OrderbookClient,
    requested_chain: Option<SupportedChainId>,
    requested_env: Option<CowEnv>,
) -> Result<(), TradingError> {
    let context = orderbook_client.context();

    if let Some(chain_id) = requested_chain
        && chain_id != context.chain_id
    {
        return Err(TradingError::InjectedOrderbookContextConflict {
            field: "chainId",
            requested: u64::from(chain_id).to_string(),
            configured: u64::from(context.chain_id).to_string(),
        });
    }

    if let Some(env) = requested_env
        && env != context.env
    {
        return Err(TradingError::InjectedOrderbookContextConflict {
            field: "env",
            requested: env.as_str().to_owned(),
            configured: context.env.as_str().to_owned(),
        });
    }

    Ok(())
}
