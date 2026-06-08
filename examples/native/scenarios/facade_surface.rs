//! Facade construction and chain resolution.
//!
//! Builds a ready-state `Trading` client through `TradingBuilder::ready`, then
//! resolves the on-chain deployment (`deployment_for_chain`) and the
//! wrapped-native token (`wrapped_native_token`) for the selected chain. Pure
//! construction — no transport and no network.

use std::error::Error;

use serde_json::json;

use cow_sdk::contracts::deployment_for_chain;
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

    // Resolve the protocol deployment (settlement, vault relayer, eth-flow) for
    // this chain from the bundled contract metadata — no RPC needed.
    let deployment = deployment_for_chain(u64::from(chain_id))?;

    // Resolve the wrapped-native token (the WETH-equivalent) for this chain.
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
