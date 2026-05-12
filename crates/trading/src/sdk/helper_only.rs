#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;

use cow_sdk_core::{
    Amount, AsyncProvider, AsyncSigner, CowEnv, Provider, Signer, SupportedChainId, TransactionHash,
};
#[cfg(not(target_arch = "wasm32"))]
use cow_sdk_orderbook::OrderBookApi;

use super::{HelperOnlySdk, helpers::ResolvedOrderbookBinding};
use crate::{
    AllowanceParameters, ApprovalParameters, OrderTraderParameters, PartialTraderParameters,
    TradingError, TradingSdkOptions, cancel_order_onchain_async, get_cow_protocol_allowance,
    get_cow_protocol_allowance_async, get_pre_sign_transaction, get_pre_sign_transaction_async,
    onchain::protocol_options_for_partial_order, types::validate_orderbook_context,
};

impl HelperOnlySdk {
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
                endpoint = "trading.helper_only.get_pre_sign_transaction_async",
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

    /// Cancels an order on-chain using a sync signer.
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
                endpoint = "trading.helper_only.on_chain_cancel_order",
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
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site.
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
                endpoint = "trading.helper_only.on_chain_cancel_order_async",
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
                endpoint = "trading.helper_only.get_cow_protocol_allowance_async",
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
    /// [`cow_sdk_core::Cancellable::cancel_with`] at the call site.
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
                endpoint = "trading.helper_only.approve_cow_protocol_async",
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
            let _ = (requested_chain, requested_env);
            Err(missing_chain_error)
        }
    }
}
