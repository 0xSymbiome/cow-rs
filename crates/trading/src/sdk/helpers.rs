use std::sync::Arc;

use cow_sdk_core::{Address, CowEnv, SupportedChainId};
#[cfg(not(target_arch = "wasm32"))]
use cow_sdk_orderbook::OrderBookApi;

use super::TradingSdk;
use crate::{
    OrderbookClient, PartialTraderParameters, QuoterParameters, SwapAdvancedSettings,
    TradeParameters, TraderParameters, TradingError, types::validate_orderbook_context,
};

#[derive(Clone)]
pub(super) struct ResolvedOrderbookBinding {
    pub(super) client: Arc<dyn OrderbookClient>,
    pub(super) chain_id: SupportedChainId,
    pub(super) env: CowEnv,
}

impl TradingSdk {
    pub(super) fn resolve_quote_owner(
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

    pub(super) fn resolve_quoter(
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

    pub(super) fn resolve_orderbook_trader(
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

    pub(super) fn resolve_chain_partial_trader(
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
