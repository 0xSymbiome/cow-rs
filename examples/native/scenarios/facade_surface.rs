//! Facade construction and chain resolution.
//!
//! Builds a ready-state `Trading` client through `TradingBuilder::ready`, then
//! resolves the wrapped-native token (`wrapped_native_token`) for the selected
//! chain. Pure construction — no transport and no network.

use std::error::Error;

use serde_json::json;

use cow_sdk::core::{AppCode, SupportedChainId, wrapped_native_token};
use cow_sdk::trading::{TraderParams, TradingBuilder};

fn main() -> Result<(), Box<dyn Error>> {
    let chain_id = SupportedChainId::Sepolia;
    let app_code = AppCode::new("cow-rs/native-capability-report")?;

    // Construct a ready-state trading client from total trader parameters —
    // the minimal facade entry point (the default orderbook is built per chain).
    let trading = TradingBuilder::ready(TraderParams::new(chain_id, app_code)?);

    // Resolve the wrapped-native token (the WETH-equivalent) for this chain.
    let wrapped_native = wrapped_native_token(chain_id);

    let report = json!({
        "surface": "cow_sdk",
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
