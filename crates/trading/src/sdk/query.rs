use super::TradingSdk;
use crate::{OrderTraderParameters, TradingError};

impl TradingSdk {
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
                endpoint = "trading.get_order",
                order_uid = %params.order_uid,
            ),
        ),
    )]
    pub async fn get_order(
        &self,
        params: &OrderTraderParameters,
    ) -> Result<cow_sdk_orderbook::Order, TradingError> {
        let (_, orderbook) = self.resolve_chain_partial_trader(params.chain_id, params.env)?;

        orderbook
            .client
            .get_order(&params.order_uid)
            .await
            .map_err(Into::into)
    }
}
