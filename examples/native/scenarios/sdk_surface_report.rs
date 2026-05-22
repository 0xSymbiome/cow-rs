use std::error::Error;

use serde_json::json;

use cow_sdk::app_data::{
    AppDataParams, generate_app_data_doc, get_app_data_info, validate_app_data_doc,
};
use cow_sdk::contracts::deployment_for_chain;
use cow_sdk::core::wrapped_native_token;
use cow_sdk::prelude::{SupportedChainId, TraderParameters, TradingSdkBuilder};
use cow_sdk::signing::{generate_order_id, order_typed_data};
use cow_sdk::trading::TradingSdkOptions;

use cow_sdk_examples_native::support::{sample_owner, sample_unsigned_order};

fn main() -> Result<(), Box<dyn Error>> {
    let chain_id = SupportedChainId::Sepolia;
    let sdk = TradingSdkBuilder::ready(
        TraderParameters::new(chain_id, "cow-rs/native-capability-report")
            .expect("app code should validate"),
        TradingSdkOptions::default(),
    )?;
    let app_data_doc = generate_app_data_doc(
        AppDataParams::default()
            .with_app_code("cow-rs/native-capability-report")
            .with_environment("review"),
    );
    let app_data_validation = validate_app_data_doc(&app_data_doc);
    let app_data_info = get_app_data_info(&app_data_doc)?;
    let unsigned_order = sample_unsigned_order();
    let typed_order = order_typed_data(chain_id, &unsigned_order, None)?;
    let generated_order = generate_order_id(chain_id, &unsigned_order, &sample_owner(), None)?;
    let deployment = deployment_for_chain(u64::from(chain_id))?;
    let wrapped_native = wrapped_native_token(chain_id);

    let report = json!({
        "surface": "cow-sdk",
        "mode": "deterministic",
        "sdkConstructed": sdk.trader_defaults().chain_id == Some(chain_id),
        "chainId": u64::from(chain_id),
        "deployment": {
            "settlement": deployment.settlement.to_hex_string(),
            "vaultRelayer": deployment.vault_relayer.to_hex_string(),
            "ethFlow": deployment.eth_flow.to_hex_string()
        },
        "wrappedNative": {
            "address": wrapped_native.address.to_hex_string(),
            "symbol": wrapped_native.symbol,
            "decimals": wrapped_native.decimals
        },
        "appData": {
            "valid": app_data_validation.success,
            "cid": app_data_info.cid,
            "appDataHex": app_data_info.app_data_hex,
            "content": app_data_info.app_data_content
        },
        "orderEnvelope": {
            "primaryType": typed_order.primary_type,
            "domainName": typed_order.domain.name,
            "digest": generated_order.order_digest,
            "orderId": generated_order.order_id.to_hex_string()
        }
    });

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
