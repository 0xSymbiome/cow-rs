use cow_sdk_core::{AsyncSigner, Signer};

use super::generic::{
    apply_settings_to_limit_trade_parameters, limit_additional_params,
    post_cow_protocol_trade_async,
};
use crate::{
    LimitOrderAdvancedSettings, LimitTradeParameters, OrderPostingResult, OrderbookClient,
    TraderParameters, TradingError, build_app_data,
};

/// Signs and submits a limit order using a synchronous signer.
///
/// Advanced settings override overlapping quote-request and app-data fields before submission.
///
/// # Errors
///
/// Returns an error when app-data generation fails, when signing fails, or when the orderbook
/// rejects the order submission.
pub async fn post_limit_order<O, S>(
    params: &LimitTradeParameters,
    trader: &TraderParameters,
    signer: &S,
    advanced_settings: Option<&LimitOrderAdvancedSettings>,
    orderbook: &O,
) -> Result<OrderPostingResult, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: Signer,
    S::Error: std::fmt::Display + cow_sdk_core::SignerError,
{
    post_limit_order_async_with_bounds(
        params,
        trader,
        signer,
        advanced_settings,
        orderbook,
        crate::validation::OrderValidityBounds::SERVICES_DEFAULT,
    )
    .await
}

/// Signs and submits a limit order using an asynchronous signer.
///
/// Advanced settings override overlapping quote-request and app-data fields before submission.
/// When no slippage is supplied, limit-order posting uses `0` basis points in app-data and order
/// construction.
///
/// # Errors
///
/// Returns an error when app-data generation fails, when signing fails, or when the orderbook
/// rejects the order submission.
pub async fn post_limit_order_async<O, S>(
    params: &LimitTradeParameters,
    trader: &TraderParameters,
    signer: &S,
    advanced_settings: Option<&LimitOrderAdvancedSettings>,
    orderbook: &O,
) -> Result<OrderPostingResult, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: AsyncSigner,
    S::Error: std::fmt::Display + cow_sdk_core::SignerError,
{
    post_limit_order_async_with_bounds(
        params,
        trader,
        signer,
        advanced_settings,
        orderbook,
        crate::validation::OrderValidityBounds::SERVICES_DEFAULT,
    )
    .await
}

/// Variant of [`post_limit_order_async`] that accepts a caller-supplied
/// [`crate::validation::OrderValidityBounds`].
///
/// # Errors
///
/// Returns an error when app-data generation fails, when signing fails, or
/// when the orderbook rejects the order submission.
pub async fn post_limit_order_async_with_bounds<O, S>(
    params: &LimitTradeParameters,
    trader: &TraderParameters,
    signer: &S,
    advanced_settings: Option<&LimitOrderAdvancedSettings>,
    orderbook: &O,
    order_bounds: crate::validation::OrderValidityBounds,
) -> Result<OrderPostingResult, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: AsyncSigner,
    S::Error: std::fmt::Display + cow_sdk_core::SignerError,
{
    let app_data_signer = advanced_settings
        .and_then(|settings| settings.app_data.as_ref())
        .and_then(|params| params.signer.clone());

    let mut params = apply_settings_to_limit_trade_parameters(
        params,
        advanced_settings.and_then(|settings| settings.quote_request.as_ref()),
        advanced_settings.and_then(|settings| settings.app_data.as_ref()),
    )?;
    if params.slippage_bps.is_none() {
        params.slippage_bps = Some(0);
    }

    let app_data_info = build_app_data(
        &trader.app_code,
        params.slippage_bps.unwrap_or(0),
        "limit",
        params.partner_fee.as_ref(),
        advanced_settings.and_then(|settings| settings.app_data.as_ref()),
    )
    .await?;

    let mut additional = limit_additional_params(advanced_settings);
    if additional.apply_costs_slippage_and_fees.is_none() {
        additional.apply_costs_slippage_and_fees = Some(false);
    }

    post_cow_protocol_trade_async(
        orderbook,
        &app_data_info,
        &params,
        &additional,
        trader,
        signer,
        order_bounds,
        app_data_signer,
    )
    .await
}
