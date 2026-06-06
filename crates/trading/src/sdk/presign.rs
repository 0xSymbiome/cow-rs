use cow_sdk_core::Signer;

use super::Trading;
use crate::{
    OrderTraderParameters, TradingError, pre_sign_transaction,
    onchain::protocol_options_for_partial_order,
};

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
