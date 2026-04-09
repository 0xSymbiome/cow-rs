use std::sync::Arc;

use cow_sdk_core::{
    ApiContext, AsyncProvider, AsyncSigner, CowEnv, Provider, Signer, SupportedChainId,
};
use cow_sdk_orderbook::OrderBookApi;

use crate::{
    AllowanceParameters, ApprovalParameters, OrderTraderParameters, OrderbookClient,
    PartialTraderParameters, QuoteResults, SwapAdvancedSettings, TradeParameters, TraderParameters,
    TradingError, TradingSdkOptions, cancel_order_onchain_async, get_cow_protocol_allowance,
    get_cow_protocol_allowance_async, get_pre_sign_transaction, get_pre_sign_transaction_async,
    get_quote_only, get_quote_results_async, off_chain_cancel_order_async, post_limit_order_async,
    post_swap_order_async, protocol_options_for_order,
};

#[derive(Clone, Default)]
pub struct TradingSdk {
    pub trader_params: PartialTraderParameters,
    pub options: TradingSdkOptions,
}

impl TradingSdk {
    pub fn new(trader_params: PartialTraderParameters, options: TradingSdkOptions) -> Self {
        Self {
            trader_params,
            options,
        }
    }

    pub fn set_trader_params(&mut self, params: PartialTraderParameters) -> &mut Self {
        if params.chain_id.is_some() {
            self.trader_params.chain_id = params.chain_id;
        }
        if params.app_code.is_some() {
            self.trader_params.app_code = params.app_code;
        }
        if params.owner.is_some() {
            self.trader_params.owner = params.owner;
        }
        if params.env.is_some() {
            self.trader_params.env = params.env;
        }
        if params.settlement_contract_override.is_some() {
            self.trader_params.settlement_contract_override = params.settlement_contract_override;
        }
        if params.eth_flow_contract_override.is_some() {
            self.trader_params.eth_flow_contract_override = params.eth_flow_contract_override;
        }

        self
    }

    pub async fn get_quote_only(
        &self,
        mut params: TradeParameters,
        advanced_settings: Option<&SwapAdvancedSettings>,
    ) -> Result<QuoteResults, TradingError> {
        params.owner = params.owner.or_else(|| self.trader_params.owner.clone());
        let owner = params.owner.clone().ok_or(TradingError::MissingOwner)?;
        let quoter = self.resolve_quoter(owner)?;
        let orderbook = self.resolve_orderbook(quoter.chain_id, quoter.env.unwrap_or(CowEnv::Prod));

        get_quote_only(&params, &quoter, advanced_settings, orderbook.as_ref()).await
    }

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

    pub async fn get_quote_results_async<S>(
        &self,
        params: TradeParameters,
        signer: &S,
        advanced_settings: Option<&SwapAdvancedSettings>,
    ) -> Result<QuoteResults, TradingError>
    where
        S: AsyncSigner,
        S::Error: std::fmt::Display,
    {
        let trader = self.resolve_trader()?;
        let orderbook = self.resolve_orderbook(trader.chain_id, trader.env.unwrap_or(CowEnv::Prod));

        get_quote_results_async(
            &params,
            &trader,
            signer,
            advanced_settings,
            orderbook.as_ref(),
        )
        .await
    }

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

    pub async fn post_swap_order_async<S>(
        &self,
        params: TradeParameters,
        signer: &S,
        advanced_settings: Option<&SwapAdvancedSettings>,
    ) -> Result<crate::OrderPostingResult, TradingError>
    where
        S: AsyncSigner,
        S::Error: std::fmt::Display,
    {
        let trader = self.resolve_trader()?;
        let orderbook = self.resolve_orderbook(trader.chain_id, trader.env.unwrap_or(CowEnv::Prod));

        post_swap_order_async(
            &params,
            &trader,
            signer,
            advanced_settings,
            orderbook.as_ref(),
        )
        .await
    }

    pub async fn post_limit_order<S>(
        &self,
        params: crate::LimitTradeParameters,
        signer: &S,
        advanced_settings: Option<&crate::LimitOrderAdvancedSettings>,
    ) -> Result<crate::OrderPostingResult, TradingError>
    where
        S: Signer,
        S::Error: std::fmt::Display,
    {
        self.post_limit_order_async(params, signer, advanced_settings)
            .await
    }

    pub async fn post_limit_order_async<S>(
        &self,
        params: crate::LimitTradeParameters,
        signer: &S,
        advanced_settings: Option<&crate::LimitOrderAdvancedSettings>,
    ) -> Result<crate::OrderPostingResult, TradingError>
    where
        S: AsyncSigner,
        S::Error: std::fmt::Display,
    {
        let trader = self.resolve_trader()?;
        let orderbook = self.resolve_orderbook(trader.chain_id, trader.env.unwrap_or(CowEnv::Prod));

        post_limit_order_async(
            &params,
            &trader,
            signer,
            advanced_settings,
            orderbook.as_ref(),
        )
        .await
    }

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

    pub async fn get_order(
        &self,
        params: &OrderTraderParameters,
    ) -> Result<cow_sdk_orderbook::Order, TradingError> {
        let trader = self.resolve_trader_without_appcode()?;
        let chain_id = params
            .chain_id
            .or(trader.chain_id)
            .ok_or_else(|| TradingError::MissingTraderParameters("chainId".to_owned()))?;
        let env = params.env.or(trader.env).unwrap_or(CowEnv::Prod);
        let orderbook = self.resolve_orderbook(chain_id, env);

        orderbook
            .get_order(&params.order_uid)
            .await
            .map_err(Into::into)
    }

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

    pub async fn off_chain_cancel_order_async<S>(
        &self,
        params: &OrderTraderParameters,
        signer: &S,
    ) -> Result<bool, TradingError>
    where
        S: AsyncSigner,
        S::Error: std::fmt::Display,
    {
        let trader = self.resolve_trader()?;
        let chain_id = params.chain_id.unwrap_or(trader.chain_id);
        let env = params.env.or(trader.env).unwrap_or(CowEnv::Prod);
        let orderbook = self.resolve_orderbook(chain_id, env);
        let effective_params = OrderTraderParameters {
            chain_id: Some(chain_id),
            env: Some(env),
            ..params.clone()
        };

        off_chain_cancel_order_async(orderbook.as_ref(), &effective_params, &trader, signer).await
    }

    pub async fn on_chain_cancel_order<S>(
        &self,
        params: &OrderTraderParameters,
        signer: &S,
    ) -> Result<String, TradingError>
    where
        S: Signer,
        S::Error: std::fmt::Display,
    {
        self.on_chain_cancel_order_async(params, signer).await
    }

    pub async fn on_chain_cancel_order_async<S>(
        &self,
        params: &OrderTraderParameters,
        signer: &S,
    ) -> Result<String, TradingError>
    where
        S: AsyncSigner,
        S::Error: std::fmt::Display,
    {
        let trader = self.resolve_trader_without_appcode()?;
        let chain_id = params
            .chain_id
            .or(trader.chain_id)
            .ok_or_else(|| TradingError::MissingTraderParameters("chainId".to_owned()))?;
        let env = params.env.or(trader.env).unwrap_or(CowEnv::Prod);
        let orderbook = self.resolve_orderbook(chain_id, env);
        let order = orderbook.get_order(&params.order_uid).await?;
        let effective_params = OrderTraderParameters {
            chain_id: Some(chain_id),
            env: Some(env),
            ..params.clone()
        };
        let options = protocol_options_for_order(
            &effective_params,
            &TraderParameters {
                chain_id,
                app_code: self.trader_params.app_code.clone().unwrap_or_default(),
                env: Some(env),
                settlement_contract_override: self
                    .trader_params
                    .settlement_contract_override
                    .clone(),
                eth_flow_contract_override: self.trader_params.eth_flow_contract_override.clone(),
            },
        );

        cancel_order_onchain_async(signer, chain_id, &order, Some(&options)).await
    }

    pub fn get_cow_protocol_allowance<P>(
        &self,
        provider: &P,
        params: &AllowanceParameters,
    ) -> Result<String, TradingError>
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

    pub async fn get_cow_protocol_allowance_async<P>(
        &self,
        provider: &P,
        params: &AllowanceParameters,
    ) -> Result<String, TradingError>
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

    pub fn approve_cow_protocol<S>(
        &self,
        signer: &S,
        params: &ApprovalParameters,
    ) -> Result<String, TradingError>
    where
        S: Signer,
        S::Error: std::fmt::Display,
    {
        let trader = self.resolve_trader()?;
        let chain_id = params.chain_id.unwrap_or(trader.chain_id);
        let env = params.env.or(trader.env).unwrap_or(CowEnv::Prod);

        crate::approve_cow_protocol(signer, params, chain_id, env)
    }

    pub async fn approve_cow_protocol_async<S>(
        &self,
        signer: &S,
        params: &ApprovalParameters,
    ) -> Result<String, TradingError>
    where
        S: AsyncSigner,
        S::Error: std::fmt::Display,
    {
        let trader = self.resolve_trader()?;
        let chain_id = params.chain_id.unwrap_or(trader.chain_id);
        let env = params.env.or(trader.env).unwrap_or(CowEnv::Prod);

        crate::approve_cow_protocol_async(signer, params, chain_id, env).await
    }

    fn resolve_trader(&self) -> Result<TraderParameters, TradingError> {
        let chain_id = self
            .trader_params
            .chain_id
            .ok_or_else(|| TradingError::MissingTraderParameters("chainId, appCode".to_owned()))?;
        let app_code =
            self.trader_params.app_code.clone().ok_or_else(|| {
                TradingError::MissingTraderParameters("chainId, appCode".to_owned())
            })?;

        Ok(TraderParameters {
            chain_id,
            app_code,
            env: self.trader_params.env,
            settlement_contract_override: self.trader_params.settlement_contract_override.clone(),
            eth_flow_contract_override: self.trader_params.eth_flow_contract_override.clone(),
        })
    }

    fn resolve_trader_without_appcode(&self) -> Result<PartialTraderParameters, TradingError> {
        let chain_id = self
            .trader_params
            .chain_id
            .ok_or_else(|| TradingError::MissingTraderParameters("chainId".to_owned()))?;

        Ok(PartialTraderParameters {
            chain_id: Some(chain_id),
            app_code: self.trader_params.app_code.clone(),
            owner: self.trader_params.owner.clone(),
            env: self.trader_params.env,
            settlement_contract_override: self.trader_params.settlement_contract_override.clone(),
            eth_flow_contract_override: self.trader_params.eth_flow_contract_override.clone(),
        })
    }

    fn resolve_quoter(
        &self,
        owner: cow_sdk_core::Address,
    ) -> Result<crate::QuoterParameters, TradingError> {
        let chain_id = self
            .trader_params
            .chain_id
            .ok_or_else(|| TradingError::MissingQuoterParameters("chainId".to_owned()))?;
        let app_code = self
            .trader_params
            .app_code
            .clone()
            .ok_or_else(|| TradingError::MissingQuoterParameters("appCode".to_owned()))?;

        Ok(crate::QuoterParameters {
            chain_id,
            app_code,
            account: owner,
            env: self.trader_params.env,
            settlement_contract_override: self.trader_params.settlement_contract_override.clone(),
            eth_flow_contract_override: self.trader_params.eth_flow_contract_override.clone(),
        })
    }

    fn resolve_orderbook(
        &self,
        chain_id: SupportedChainId,
        env: CowEnv,
    ) -> Arc<dyn OrderbookClient> {
        self.options.order_book_api.clone().unwrap_or_else(|| {
            Arc::new(OrderBookApi::new(ApiContext {
                chain_id,
                env,
                base_urls: None,
                api_key: None,
            }))
        })
    }
}
