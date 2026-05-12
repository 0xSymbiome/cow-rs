use cow_sdk_core::{AsyncSigner, Signer};

use super::from_quote::post_swap_order_from_quote_async_with_bounds;
use crate::{
    OrderPostingResult, OrderbookClient, SwapAdvancedSettings, TradeParameters, TraderParameters,
    TradingError,
};

// Non-suffixed posting functions are async entry points for synchronous Signer implementors.
// Keep workflow logic in the AsyncSigner implementations so both public paths stay aligned.
/// Quotes, signs, and submits a swap order using a synchronous signer.
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
    advanced_settings: Option<&SwapAdvancedSettings>,
    orderbook: &O,
) -> Result<OrderPostingResult, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: Signer,
    S::Error: std::fmt::Display,
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

/// Variant of [`post_swap_order`] that accepts a caller-supplied
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
    advanced_settings: Option<&SwapAdvancedSettings>,
    orderbook: &O,
    order_bounds: crate::validation::OrderValidityBounds,
) -> Result<OrderPostingResult, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: Signer,
    S::Error: std::fmt::Display,
{
    post_swap_order_async_with_bounds(
        trade_parameters,
        trader,
        signer,
        advanced_settings,
        orderbook,
        order_bounds,
    )
    .await
}

/// Quotes, signs, and submits a swap order using an asynchronous signer.
///
/// Advanced settings override overlapping trade and app-data fields before submission.
///
/// # Errors
///
/// Returns an error when quoting fails, when app-data generation or merging fails, when signing
/// fails, or when the orderbook rejects the order submission.
pub async fn post_swap_order_async<O, S>(
    trade_parameters: &TradeParameters,
    trader: &TraderParameters,
    signer: &S,
    advanced_settings: Option<&SwapAdvancedSettings>,
    orderbook: &O,
) -> Result<OrderPostingResult, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: AsyncSigner,
    S::Error: std::fmt::Display,
{
    post_swap_order_async_with_bounds(
        trade_parameters,
        trader,
        signer,
        advanced_settings,
        orderbook,
        crate::validation::OrderValidityBounds::SERVICES_DEFAULT,
    )
    .await
}

/// Variant of [`post_swap_order_async`] that accepts caller-supplied
/// [`crate::validation::OrderValidityBounds`].
///
/// # Errors
///
/// Returns an error when quoting fails, when app-data generation or merging
/// fails, when signing fails, or when the orderbook rejects the order
/// submission.
pub async fn post_swap_order_async_with_bounds<O, S>(
    trade_parameters: &TradeParameters,
    trader: &TraderParameters,
    signer: &S,
    advanced_settings: Option<&SwapAdvancedSettings>,
    orderbook: &O,
    order_bounds: crate::validation::OrderValidityBounds,
) -> Result<OrderPostingResult, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: AsyncSigner,
    S::Error: std::fmt::Display,
{
    let quote_results = crate::get_quote_results_async(
        trade_parameters,
        trader,
        signer,
        advanced_settings,
        orderbook,
    )
    .await?;

    post_swap_order_from_quote_async_with_bounds(
        &quote_results,
        trader,
        signer,
        advanced_settings,
        orderbook,
        order_bounds,
    )
    .await
}
