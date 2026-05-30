use cow_sdk_core::{Address, Signer};
use cow_sdk_orderbook::SigningScheme;

use super::generic::current_unix_seconds;
use crate::types::{validate_orderbook_context, validate_orderbook_env_context};
use crate::validation::OrderBoundsValidator;
use crate::{
    LimitTradeParametersFromQuote, OrderPostingResult, OrderbookClient, TraderParameters,
    TradingAppDataInfo, TradingError,
};

/// Submits an `EthFlow`-style native-currency sell order.
///
/// This path uploads the supplied app-data, sends the prepared transaction through the signer, and
/// returns the resulting transaction hash. Callers that need cooperative
/// cancellation wrap this future through
/// [`cow_sdk_core::Cancellable::cancel_with`] at the call site; cancellation
/// only affects pre-broadcast work, because once the signer has broadcast the
/// prepared transaction, it cannot be withdrawn and the returned receipt will
/// reflect the chain result even if cancellation fires after submission. A
/// cancellation fired between transaction preparation and app-data upload is
/// a no-op on the orderbook service.
///
/// # Errors
///
/// Returns an error when transaction preparation fails, when app-data upload fails, or when the
/// signer cannot send the transaction.
#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        skip_all,
        fields(
            chain = ?trader.chain_id,
            env = ?trader.env,
            endpoint = "trading.post_sell_native_currency_order",
        ),
    ),
)]
#[allow(
    clippy::too_many_arguments,
    reason = "the eth-flow submission seam threads orchestration, validator, and runtime context through one entry point for parity with the reviewed services authority"
)]
pub async fn post_sell_native_currency_order<O, S>(
    orderbook: &O,
    app_data: &TradingAppDataInfo,
    params: &LimitTradeParametersFromQuote,
    additional_params: &crate::types::PostTradeAdditionalParams,
    trader: &TraderParameters,
    signer: &S,
    app_data_signer: Option<Address>,
) -> Result<OrderPostingResult, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: Signer,
    S::Error: std::fmt::Display + cow_sdk_core::SignerError,
{
    validate_orderbook_context(orderbook, Some(trader.chain_id), trader.env)?;
    validate_orderbook_env_context(orderbook, params.as_limit().env)?;

    let orderbook_context = orderbook.context();
    let canonical_chain_id = orderbook_context.chain_id;
    let canonical_env = orderbook_context.env;
    let mut inner = params.as_limit().clone();
    inner.env = Some(canonical_env);
    let params = LimitTradeParametersFromQuote::try_from_limit(inner)?;

    let tx = crate::get_eth_flow_transaction(
        &app_data.app_data_keccak256,
        &params,
        canonical_chain_id,
        additional_params,
        trader,
        signer,
    )
    .await?;

    let validator = OrderBoundsValidator::services_default_for_chain(canonical_chain_id);
    validator
        .validate(
            &tx.order_to_sign,
            tx.from,
            SigningScheme::Eip1271,
            app_data_signer,
            current_unix_seconds(),
            true,
        )
        .map_err(TradingError::ClientRejected)?;

    orderbook
        .upload_app_data(&app_data.app_data_keccak256, &app_data.full_app_data)
        .await?;

    let broadcast = signer
        .send_transaction(&tx.transaction)
        .await
        .map_err(|error| TradingError::Signer {
            operation: "send_transaction",
            message: error.to_string().into(),
        })?;

    Ok(OrderPostingResult {
        order_id: tx.order_id,
        tx_hash: Some(broadcast.transaction_hash),
        order_to_sign: tx.order_to_sign,
        signature: String::new(),
        signing_scheme: SigningScheme::Eip1271,
    })
}
