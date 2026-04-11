use std::error::Error;

use serde_json::json;

use cow_sdk::core::EVM_NATIVE_CURRENCY_ADDRESS;
use cow_sdk::trading::{
    PostTradeAdditionalParams, build_app_data, get_eth_flow_transaction,
    post_sell_native_currency_order,
};

use cow_sdk_examples_native::support::{
    MockOrderbook, MockSigner, sample_limit_parameters, sample_quote_response,
    sample_trader_parameters, text_preview,
};

fn call_data_prefix(data: &cow_sdk::HexData) -> &str {
    text_preview(data.as_str(), 10)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let orderbook = MockOrderbook::new(
        cow_sdk::SupportedChainId::Sepolia,
        sample_quote_response(),
    );
    let signer = MockSigner::default();
    let trader = sample_trader_parameters();
    let mut params = sample_limit_parameters();
    params.sell_token = cow_sdk::Address::new(EVM_NATIVE_CURRENCY_ADDRESS)?;
    params.valid_to = Some(1_737_464_594);

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
            "orderId": ethflow.order_id.as_str(),
            "contract": ethflow.transaction.to.as_ref().map(|address| address.as_str()),
            "value": ethflow.transaction.value.as_ref().map(|value| value.as_str()),
            "gasLimit": ethflow.transaction.gas_limit.as_ref().map(|value| value.as_str()),
            "callDataPrefix": call_data_prefix(
                ethflow
                    .transaction
                    .data
                    .as_ref()
                    .expect("ethflow transaction should include call data"),
            ),
            "requestedSellToken": params.sell_token.as_str(),
            "effectiveSellToken": ethflow.order_to_sign.sell_token.as_str(),
            "buyToken": ethflow.order_to_sign.buy_token.as_str(),
            "quoteId": params.quote_id,
            "appDataHash": app_data.app_data_keccak256.as_str()
        },
        "nativeSellPosting": {
            "orderId": submitted.order_id.as_str(),
            "txHash": submitted.tx_hash.as_ref().map(|hash| hash.as_str()),
            "signingScheme": format!("{:?}", submitted.signing_scheme),
            "uploadedAppDataHash": upload.0.as_str(),
            "uploadedAppDataPreview": text_preview(&upload.1, 96),
            "uploadedAppDataEntries": state.uploads.len()
        }
    });

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
