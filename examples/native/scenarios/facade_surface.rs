//! Facade construction and chain resolution.
//!
//! Builds a ready-state `Trading` client through `TradingBuilder::ready`, then
//! resolves the wrapped-native token (`wrapped_native_token`) for the selected
//! chain. Pure construction — no transport and no network.

use std::error::Error;

use serde_json::json;

use cow_sdk::core::{AppCode, SupportedChainId, wrapped_native_token};
use cow_sdk::trading::{TraderParameters, TradingBuilder, TradingOptions};

fn main() -> Result<(), Box<dyn Error>> {
    let chain_id = SupportedChainId::Sepolia;
    let app_code = AppCode::new("cow-rs/native-capability-report")?;

    // Construct a ready-state trading client — the minimal facade entry point.
    let trading = TradingBuilder::ready(
        TraderParameters::new(chain_id, app_code).expect("app code should validate"),
        TradingOptions::default(),
    )?;

    // Resolve the wrapped-native token (the WETH-equivalent) for this chain.
    let wrapped_native = wrapped_native_token(chain_id);

    let report = json!({
        "surface": "cow-sdk",
        "mode": "deterministic",
        "sdkConstructed": trading.chain_id() == Some(chain_id),
        "chainId": u64::from(chain_id),
        "wrappedNative": {
            "address": wrapped_native.address.to_hex_string(),
            "symbol": wrapped_native.symbol,
            "decimals": wrapped_native.decimals
        }
    });

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
