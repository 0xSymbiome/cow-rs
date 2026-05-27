use cow_sdk_core::Signer;

use super::generic::{
    apply_settings_to_limit_trade_parameters, post_cow_protocol_trade, swap_additional_params,
};
use crate::types::validate_quote_orderbook_binding;
use crate::{
    OrderPostingResult, OrderbookClient, QuoteResults, SwapAdvancedSettings, TraderParameters,
    TradingError, merge_and_seal_app_data, params_from_doc, swap_params_to_limit_order_params,
};

/// Signs and submits a swap order from previously computed quote results.
///
/// When advanced app-data settings are provided, they are merged on top of the quote-derived
/// document before submission. The submission orderbook must match the runtime
/// binding captured by the quote flow, and any explicit chain or environment
/// must agree with the injected orderbook client, which remains the canonical
/// runtime authority for signing and submission. Callers that need cooperative
/// cancellation wrap this future through
/// [`cow_sdk_core::Cancellable::cancel_with`] at the call site.
///
/// # Errors
///
/// Returns an error when the quoted trade cannot be converted into a postable order, when app-data
/// merging fails, when signing fails, or when the orderbook rejects the order submission.
#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        skip_all,
        fields(
            chain = ?trader.chain_id,
            env = ?trader.env,
            endpoint = "trading.post_swap_order_from_quote",
        ),
    ),
)]
pub async fn post_swap_order_from_quote<O, S>(
    quote_results: &QuoteResults,
    trader: &TraderParameters,
    signer: &S,
    advanced_settings: Option<&SwapAdvancedSettings>,
    orderbook: &O,
) -> Result<OrderPostingResult, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: Signer,
    S::Error: std::fmt::Display + cow_sdk_core::SignerError,
{
    post_swap_order_from_quote_with_bounds(
        quote_results,
        trader,
        signer,
        advanced_settings,
        orderbook,
        crate::validation::OrderValidityBounds::SERVICES_DEFAULT,
    )
    .await
}

/// Variant of [`post_swap_order_from_quote`] that accepts a caller-supplied
/// [`crate::validation::OrderValidityBounds`].
///
/// # Errors
///
/// Returns an error when the quoted trade cannot be converted into a
/// postable order, when app-data merging fails, when signing fails, or when
/// the orderbook rejects the order submission.
pub async fn post_swap_order_from_quote_with_bounds<O, S>(
    quote_results: &QuoteResults,
    trader: &TraderParameters,
    signer: &S,
    advanced_settings: Option<&SwapAdvancedSettings>,
    orderbook: &O,
    order_bounds: crate::validation::OrderValidityBounds,
) -> Result<OrderPostingResult, TradingError>
where
    O: OrderbookClient + ?Sized,
    S: Signer,
    S::Error: std::fmt::Display + cow_sdk_core::SignerError,
{
    validate_quote_orderbook_binding(orderbook, quote_results.orderbook_binding.as_ref())?;

    let (app_data_info, merged_params) = if let Some(app_data_override) =
        advanced_settings.and_then(|settings| settings.app_data.as_ref())
    {
        merge_and_seal_app_data(&quote_results.app_data_info.doc, app_data_override)?
    } else {
        // Even without an override the submission seam reads the typed
        // `metadata.signer` field for the `AppdataFromMismatch`
        // invariant, so parse the sealed base doc back into typed
        // params and surface its signer alongside the existing
        // `TradingAppDataInfo`.
        let base_params = params_from_doc(&quote_results.app_data_info.doc)?;
        (quote_results.app_data_info.clone(), base_params)
    };
    let app_data_signer = merged_params.signer;
    let params = apply_settings_to_limit_trade_parameters(
        &swap_params_to_limit_order_params(
            &quote_results.trade_parameters,
            &quote_results.quote_response,
        )?,
        advanced_settings.and_then(|settings| settings.quote_request.as_ref()),
        advanced_settings.and_then(|settings| settings.app_data.as_ref()),
    )?;
    let additional = swap_additional_params(advanced_settings);
    let additional_params = crate::types::PostTradeAdditionalParams {
        signing_scheme: advanced_settings
            .and_then(|settings| settings.quote_request.as_ref())
            .and_then(|request| request.signing_scheme),
        network_costs_amount: Some(*quote_results.quote_response.quote.network_cost_amount()),
        ..additional
    };

    post_cow_protocol_trade(
        orderbook,
        &app_data_info,
        &params,
        &additional_params,
        trader,
        signer,
        order_bounds,
        app_data_signer,
    )
    .await
}
