//! Native-sell / EthFlow order construction and posting.
//!
//! Builds the on-chain EthFlow transaction (`eth_flow_transaction`) and
//! posts a native-currency sell order (`post_sell_native_currency_order`) with
//! merged app data (`build_app_data`), against a transport-mocked orderbook and
//! signer. EthFlow lets a user sell the native token (for example ETH) directly.

use std::error::Error;

use serde_json::json;

use cow_sdk::core::{NATIVE_CURRENCY_ADDRESS, SupportedChainId};
use cow_sdk::orderbook::OrderClass;
use cow_sdk::trading::{
    LimitTradeParamsFromQuote, PostTradeAdditionalParams, build_app_data, eth_flow_transaction,
    post_sell_native_currency_order,
};

use cow_sdk::testing::{MockOrderbook, MockSigner};
use cow_sdk_examples_native::support::{
    OWNER, call_data_prefix, sample_limit_parameters, sample_quote_response,
    sample_trader_parameters, text_preview,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let orderbook = MockOrderbook::builder(SupportedChainId::Sepolia)
        .quote(sample_quote_response())
        .build();
    let signer = MockSigner::builder().address(OWNER).build();
    let trader = sample_trader_parameters();
    // Sell the native token: set the sell token to the typed native sentinel and
    // give the order a one-hour validity window from now.
    let mut params = sample_limit_parameters().with_valid_for(3600);
    params.sell_token = NATIVE_CURRENCY_ADDRESS;

    // Build the app data, then finalize the limit parameters into from-quote form.
    let app_data = build_app_data(&trader.app_code, 0, OrderClass::Market, None, None).await?;
    let additional = PostTradeAdditionalParams::default();

    let requested_sell_token = params.sell_token;
    let requested_quote_id = params.quote_id;
    let from_quote = LimitTradeParamsFromQuote::try_from_limit(params)?;

    // Build the on-chain EthFlow transaction (the contract call that creates the
    // order) without posting anything. The chain comes from `trader.chain_id`.
    let ethflow = eth_flow_transaction(
        &app_data.app_data_keccak256,
        &from_quote,
        &additional,
        &trader,
        &signer,
    )
    .await?;

    // Post the native-sell order to the orderbook; this uploads the app data.
    let submitted = post_sell_native_currency_order(
        &orderbook,
        &app_data,
        &from_quote,
        &additional,
        &trader,
        &signer,
        None,
    )
    .await?;
    let state = orderbook.recorded();
    let upload = state
        .uploads
        .first()
        .expect("native-sell simulation should upload app data");

    // Every `PreparedTransaction` field is unconditionally set, so the report
    // reads them directly — no unwrapping of SDK output.
    let report = json!({
        "surface": "cow_sdk::trading::{eth_flow_transaction, post_sell_native_currency_order}",
        "mode": "simulated-transport",
        "ethFlowTransaction": {
            "orderId": ethflow.order_id.to_hex_string(),
            "contract": ethflow.transaction.to.to_hex_string(),
            "value": ethflow.transaction.value.to_string(),
            "gasLimit": ethflow.transaction.gas_limit.to_string(),
            "callDataPrefix": call_data_prefix(&ethflow.transaction.data),
            "requestedSellToken": requested_sell_token.to_hex_string(),
            "effectiveSellToken": ethflow.order_to_sign.sell_token.to_hex_string(),
            "buyToken": ethflow.order_to_sign.buy_token.to_hex_string(),
            "quoteId": requested_quote_id,
            "appDataHash": app_data.app_data_keccak256.to_hex_string()
        },
        "nativeSellPosting": {
            "orderId": submitted.order_id.to_hex_string(),
            "txHash": submitted.tx_hash.as_ref().map(|hash| hash.to_hex_string()),
            "signingScheme": submitted.signing_scheme,
            "uploadedAppDataHash": upload.0.to_hex_string(),
            "uploadedAppDataPreview": text_preview(&upload.1, 96),
            "uploadedAppDataEntries": state.uploads.len()
        }
    });

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
