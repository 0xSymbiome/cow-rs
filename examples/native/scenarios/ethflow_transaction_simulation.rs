use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;

use cow_sdk::core::{Address, EVM_NATIVE_CURRENCY_ADDRESS, HexData};
use cow_sdk::prelude::SupportedChainId;
use cow_sdk::trading::{
    OrderValidityBounds, PostTradeAdditionalParams, build_app_data, get_eth_flow_transaction,
    post_sell_native_currency_order,
};

use cow_sdk_examples_native::support::{
    MockOrderbook, MockSigner, sample_limit_parameters, sample_quote_response,
    sample_trader_parameters, text_preview,
};

fn call_data_prefix(data: &HexData) -> String {
    text_preview(&data.to_hex_string(), 10).to_owned()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let orderbook = MockOrderbook::new(SupportedChainId::Sepolia, sample_quote_response());
    let signer = MockSigner::default();
    let trader = sample_trader_parameters();
    let mut params = sample_limit_parameters();
    params.sell_token = Address::new(EVM_NATIVE_CURRENCY_ADDRESS)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be at or after the unix epoch")
        .as_secs();
    params.valid_to =
        Some(u32::try_from(now + 3600).expect("valid_to fits in u32 for the next century"));

    let app_data = build_app_data(&trader.app_code, 0, "market", None, None).await?;
    let additional = PostTradeAdditionalParams::default();

    let ethflow = get_eth_flow_transaction(
        &app_data.app_data_keccak256,
        &params,
        trader.chain_id,
        &additional,
        &trader,
        &signer,
    )
    .await?;

    let submitted = post_sell_native_currency_order(
        &orderbook,
        &app_data,
        &params,
        &additional,
        &trader,
        &signer,
        OrderValidityBounds::SERVICES_DEFAULT,
        None,
    )
    .await?;
    let state = orderbook.state();
    let upload = state
        .uploads
        .first()
        .expect("native-sell simulation should upload app data");

    let report = json!({
        "surface": "cow-sdk::trading::get_eth_flow_transaction + post_sell_native_currency_order",
        "mode": "simulated-transport",
        "ethFlowTransaction": {
            "orderId": ethflow.order_id.to_hex_string(),
            "contract": ethflow.transaction.to.as_ref().map(Address::to_hex_string),
            "value": ethflow.transaction.value.as_ref().map(ToString::to_string),
            "gasLimit": ethflow.transaction.gas_limit.as_ref().map(ToString::to_string),
            "callDataPrefix": call_data_prefix(
                ethflow
                    .transaction
                    .data
                    .as_ref()
                    .expect("ethflow transaction should include call data"),
            ),
            "requestedSellToken": params.sell_token.to_hex_string(),
            "effectiveSellToken": ethflow.order_to_sign.sell_token.to_hex_string(),
            "buyToken": ethflow.order_to_sign.buy_token.to_hex_string(),
            "quoteId": params.quote_id,
            "appDataHash": app_data.app_data_keccak256.to_hex_string()
        },
        "nativeSellPosting": {
            "orderId": submitted.order_id.to_hex_string(),
            "txHash": submitted.tx_hash.as_ref().map(|hash| hash.to_hex_string()),
            "signingScheme": format!("{:?}", submitted.signing_scheme),
            "uploadedAppDataHash": upload.0.to_hex_string(),
            "uploadedAppDataPreview": text_preview(&upload.1, 96),
            "uploadedAppDataEntries": state.uploads.len()
        }
    });

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
