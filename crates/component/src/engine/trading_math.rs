//! Pure quote-pipeline math for the engine world's `trading-math` interface.
//!
//! A thin lowering over the pure [`cow_sdk_trading`] helpers: it parses the JSON
//! quote inputs into the native types, runs the integer-exact amounts-and-costs
//! breakdown, the automatic slippage suggestion, and the high-level app-data
//! document builder, and returns the wire records. No host imports and no
//! network — the same audited math the network client lanes run, available to a
//! consumer that fetches its own `/quote`.

use cow_sdk_app_data::AppDataInfo;
use cow_sdk_core::{Address, AppCode, QuoteAmountsAndCosts, SupportedChainId};
use cow_sdk_orderbook::{OrderClass, OrderQuoteResponse, QuoteData};
use cow_sdk_trading::{
    QuoterParams, TradeParams, build_app_data_doc, calculate_quote_amounts_and_costs,
    sanitize_protocol_fee_bps, suggest_slippage_bps,
};

/// A placeholder address for the trade/quoter parameters the slippage suggestion
/// requires for shape only: [`suggest_slippage_bps`] reads the partner fee from
/// the trade parameters and the chain id from the quoter parameters, and the
/// chain id only feeds the eth-flow default floor (which is chain-independent), so
/// no token or account address participates in the math.
const PLACEHOLDER_ADDRESS: &str = "0x0000000000000000000000000000000000000000";

/// Parses the optional decimal-string protocol-fee bps echoed by a `/quote`
/// response into the sanitized fraction the math expects, treating an empty
/// string as absent.
fn protocol_fee(protocol_fee_bps: &str) -> Option<f64> {
    let trimmed = protocol_fee_bps.trim();
    if trimmed.is_empty() {
        None
    } else {
        sanitize_protocol_fee_bps(Some(trimmed))
    }
}

/// Computes the stepwise quote amounts and costs from a `QuoteData` JSON.
pub fn calculate_amounts_and_costs(
    quote_json: &str,
    slippage_bps: u32,
    partner_fee_bps: u32,
    protocol_fee_bps: &str,
) -> Result<QuoteAmountsAndCosts, String> {
    let quote: QuoteData = serde_json::from_str(quote_json).map_err(|error| error.to_string())?;
    calculate_quote_amounts_and_costs(
        &quote,
        slippage_bps,
        Some(partner_fee_bps),
        protocol_fee(protocol_fee_bps),
    )
    .map_err(|error| error.to_string())
}

/// Suggests an automatic slippage tolerance (bps) from a full `/quote` response.
pub fn suggest_slippage(
    quote_json: &str,
    partner_fee_bps: u32,
    is_eth_flow: bool,
) -> Result<u32, String> {
    let quote: OrderQuoteResponse =
        serde_json::from_str(quote_json).map_err(|error| error.to_string())?;
    let placeholder = Address::new(PLACEHOLDER_ADDRESS).map_err(|error| error.to_string())?;

    // `suggest_slippage_bps` reads only the partner fee from the trade
    // parameters and only the chain id from the quoter parameters; the chain id
    // selects the eth-flow default floor, which is chain-independent. Mainnet is
    // a stable placeholder, and the partner fee is carried by the bps argument.
    let mut trade_parameters = TradeParams::new(
        quote.quote.kind,
        placeholder,
        placeholder,
        quote.quote.sell_amount,
    );
    trade_parameters.partner_fee = synthetic_partner_fee(partner_fee_bps, placeholder);
    let trader = QuoterParams::new(SupportedChainId::Mainnet, "cow-sdk", placeholder)
        .map_err(|error| error.to_string())?;

    suggest_slippage_bps(&quote, &trade_parameters, &trader, is_eth_flow, None)
        .map_err(|error| error.to_string())
}

/// Builds a one-policy volume partner fee from a bps value, or `None` when the
/// bps is zero or out of the `u16` partner-fee range. The slippage suggestion
/// reads only the volume bps back through [`partner_fee_bps`], so the recipient
/// address is shape-only.
fn synthetic_partner_fee(bps: u32, recipient: Address) -> Option<cow_sdk_app_data::PartnerFee> {
    let volume_bps = u16::try_from(bps).ok().filter(|bps| *bps > 0)?;
    Some(cow_sdk_app_data::PartnerFee::Single(
        cow_sdk_app_data::PartnerFeePolicy::Volume {
            volume_bps,
            recipient,
        },
    ))
}

/// Builds the trading app-data document and its canonical identity from an
/// app-code, a slippage, and an order-class string (`market` default).
pub fn build_app_data(
    app_code: &str,
    slippage_bps: u32,
    order_class: Option<&str>,
) -> Result<AppDataInfo, String> {
    let app_code = AppCode::new(app_code).map_err(|error| error.to_string())?;
    let order_class = order_class.unwrap_or(OrderClass::Market.as_str());
    let built = build_app_data_doc(&app_code, slippage_bps, order_class, None, None)
        .map_err(|error| error.to_string())?;
    // Re-derive the full canonical info (cid + content + 0x hash) from the sealed
    // document, the same shape the `app-data` interface returns.
    Ok(cow_sdk_app_data::app_data_info(built.doc)
        .map_err(|error| error.to_string())?
        .info)
}
