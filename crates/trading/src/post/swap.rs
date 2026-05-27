use cow_sdk_core::Signer;

use super::from_quote::post_swap_order_from_quote_with_bounds;
use crate::{
    OrderPostingResult, OrderbookClient, TradeAdvancedSettings, TradeParameters, TraderParameters,
    TradingError,
};

/// Quotes, signs, and submits a swap order.
///
/// Advanced settings override overlapping trade and app-data fields before submission.
///
/// # Errors
///
/// Returns an error when quoting fails, when app-data generation or merging fails, when signing
/// fails, or when the orderbook rejects the order submission.
pub async fn post_swap_order<O, S>(
    trade_parameters: &TradeParameters,
    trader: &TraderParameters,
    signer: &S,
    advanced_settings: Option<&TradeAdvancedSettings>,
    orderbook: &O,
) -> Result<OrderPostingResult, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: Signer,
    S::Error: std::fmt::Display + cow_sdk_core::SignerError,
{
    post_swap_order_with_bounds(
        trade_parameters,
        trader,
        signer,
        advanced_settings,
        orderbook,
        crate::validation::OrderValidityBounds::SERVICES_DEFAULT,
    )
    .await
}

/// Variant of [`post_swap_order`] that accepts caller-supplied
/// [`crate::validation::OrderValidityBounds`] so the reviewed lifetime
/// ceiling can be tightened by policy.
///
/// # Errors
///
/// Returns an error when quoting fails, when app-data generation or merging
/// fails, when signing fails, or when the orderbook rejects the order
/// submission.
pub async fn post_swap_order_with_bounds<O, S>(
    trade_parameters: &TradeParameters,
    trader: &TraderParameters,
    signer: &S,
    advanced_settings: Option<&TradeAdvancedSettings>,
    orderbook: &O,
    order_bounds: crate::validation::OrderValidityBounds,
) -> Result<OrderPostingResult, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: Signer,
    S::Error: std::fmt::Display + cow_sdk_core::SignerError,
{
    let quote_results = crate::get_quote_results(
        trade_parameters,
        trader,
        signer,
        advanced_settings,
        orderbook,
    )
    .await?;

    post_swap_order_from_quote_with_bounds(
        &quote_results,
        trader,
        signer,
        advanced_settings,
        orderbook,
        order_bounds,
    )
    .await
}
