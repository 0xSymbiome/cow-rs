use std::error::Error;

use serde_json::json;

use cow_sdk::AppCode;
use cow_sdk::contracts::deployment_for_chain;
use cow_sdk::core::wrapped_native_token;
use cow_sdk::prelude::{SupportedChainId, TraderParameters, TradingBuilder};
use cow_sdk::trading::TradingOptions;

fn main() -> Result<(), Box<dyn Error>> {
    let chain_id = SupportedChainId::Sepolia;
    let app_code = AppCode::new("cow-rs/native-capability-report")?;
    let trading = TradingBuilder::ready(
        TraderParameters::new(chain_id, app_code).expect("app code should validate"),
        TradingOptions::default(),
    )?;
    let deployment = deployment_for_chain(u64::from(chain_id))?;
    let wrapped_native = wrapped_native_token(chain_id);

    let report = json!({
        "surface": "cow-sdk",
        "mode": "deterministic",
        "sdkConstructed": trading.trader_defaults().chain_id == Some(chain_id),
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
        }
    });

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
