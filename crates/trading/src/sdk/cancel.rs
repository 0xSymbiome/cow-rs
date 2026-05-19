use cow_sdk_core::{AsyncSigner, Signer, TransactionHash};

use super::TradingSdk;
use crate::{
    OrderTraderParameters, TradingError, cancel_order_onchain_async, off_chain_cancel_order_async,
    onchain::protocol_options_for_partial_order,
};

impl TradingSdk {
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
                order_uid = %params.order_uid,
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
        S::Error: std::fmt::Display + cow_sdk_core::SignerError,
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
                order_uid = %params.order_uid,
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
        S::Error: std::fmt::Display + cow_sdk_core::SignerError,
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
}
