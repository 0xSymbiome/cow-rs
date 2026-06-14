use std::sync::Arc;

use cow_sdk_core::{Address, CowEnv, SupportedChainId};
use cow_sdk_orderbook::OrderbookApi;

use super::Trading;
use crate::{
    OrderbookClient, PartialTraderParams, QuoterParams, TradeAdvancedSettings, TradeParams,
    TraderParams, TradingError, types::validate_orderbook_context,
};

#[derive(Clone)]
pub(super) struct ResolvedOrderbookBinding {
    pub(super) client: Arc<dyn OrderbookClient>,
    pub(super) chain_id: SupportedChainId,
    pub(super) env: CowEnv,
}

impl Trading {
    pub(super) fn resolve_quote_owner(
        params: &TradeParams,
        advanced_settings: Option<&TradeAdvancedSettings>,
    ) -> Result<Address, TradingError> {
        advanced_settings
            .and_then(|settings| settings.quote_request.as_ref())
            .and_then(|override_request| override_request.from)
            .or(params.owner)
            .ok_or(TradingError::MissingOwner)
    }

    pub(super) fn resolve_quoter(
        &self,
        owner: Address,
        requested_env: Option<CowEnv>,
    ) -> Result<(QuoterParams, ResolvedOrderbookBinding), TradingError> {
        let app_code = self
            .trader_defaults
            .app_code
            .clone()
            .ok_or(TradingError::MissingQuoterParams("appCode"))?;
        let orderbook = self.resolve_orderbook_binding(
            self.trader_defaults.chain_id,
            requested_env.or(self.trader_defaults.env),
            TradingError::MissingQuoterParams("chainId"),
        )?;

        Ok((
            QuoterParams {
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

    pub(super) fn resolve_orderbook_trader(
        &self,
        requested_chain: Option<SupportedChainId>,
        requested_env: Option<CowEnv>,
    ) -> Result<(TraderParams, ResolvedOrderbookBinding), TradingError> {
        let app_code = self
            .trader_defaults
            .app_code
            .clone()
            .ok_or_else(|| TradingError::MissingTraderParams("chainId, appCode"))?;
        let orderbook = self.resolve_orderbook_binding(
            requested_chain.or(self.trader_defaults.chain_id),
            requested_env.or(self.trader_defaults.env),
            TradingError::MissingTraderParams("chainId, appCode"),
        )?;

        Ok((
            TraderParams {
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

    pub(super) fn resolve_chain_partial_trader(
        &self,
        requested_chain: Option<SupportedChainId>,
        requested_env: Option<CowEnv>,
    ) -> Result<(PartialTraderParams, ResolvedOrderbookBinding), TradingError> {
        let orderbook = self.resolve_orderbook_binding(
            requested_chain.or(self.trader_defaults.chain_id),
            requested_env.or(self.trader_defaults.env),
            TradingError::MissingTraderParams("chainId"),
        )?;

        Ok((
            PartialTraderParams {
                chain_id: Some(orderbook.chain_id),
                app_code: self.trader_defaults.app_code.clone(),
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

    pub(super) fn resolve_orderbook_binding(
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
        // The default-built client carries the standard orderbook transport
        // policy and works on every target: the orderbook builder's
        // default-transport terminal constructs `ReqwestTransport` on native
        // and the browser `FetchTransport` on `wasm32`. Consumers needing a
        // custom retry/rate-limit policy build their own `OrderbookApi` with
        // it and inject it through `TradingOptions::with_orderbook_client`.
        let client = OrderbookApi::builder().chain(chain_id).env(env).build()?;
        Ok(ResolvedOrderbookBinding {
            client: Arc::new(client),
            chain_id,
            env,
        })
    }
}
