//! EthFlow order-id collision avoidance through the public `EthFlowOrderExistsChecker` seam.
//!
//! Implements `EthFlowOrderExistsChecker` and wires it through
//! `PostTradeAdditionalParams::with_check_eth_flow_order_exists`, then builds the
//! on-chain EthFlow transaction with `get_eth_flow_transaction`. Each reported
//! collision forces a fresh order id, so a run with scripted collisions resolves
//! to a different id than a collision-free run; both runs drain their checker
//! queue. Uses the `cow_sdk::testing` signer and no live transport.

use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use serde_json::json;

use cow_sdk::core::{Address, EVM_NATIVE_CURRENCY_ADDRESS, OrderDigest, OrderUid};
use cow_sdk::trading::{
    EthFlowOrderExistsChecker, LimitTradeParametersFromQuote, PostTradeAdditionalParams,
    TradingError, build_app_data, get_eth_flow_transaction,
};

use cow_sdk::testing::MockSigner;
use cow_sdk_examples_native::support::{
    sample_limit_parameters, sample_owner, sample_trader_parameters,
};

/// A consumer existence checker that replays a scripted collision sequence: each
/// `true` reports the generated id as already taken, forcing a fresh attempt.
struct ScriptedEthFlowChecker {
    collisions: Arc<Mutex<Vec<bool>>>,
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl EthFlowOrderExistsChecker for ScriptedEthFlowChecker {
    async fn order_exists(
        &self,
        _order_id: &OrderUid,
        _order_digest: &OrderDigest,
    ) -> Result<bool, TradingError> {
        let mut collisions = self
            .collisions
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Ok(if collisions.is_empty() {
            false
        } else {
            collisions.remove(0)
        })
    }
}

fn native_sell_params() -> Result<LimitTradeParametersFromQuote, Box<dyn Error>> {
    // Sell the native token with a one-hour validity window from now.
    let mut params = sample_limit_parameters();
    params.sell_token = Address::new(EVM_NATIVE_CURRENCY_ADDRESS)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be at or after the unix epoch")
        .as_secs();
    params.valid_to = Some(u32::try_from(now + 3600).expect("valid_to fits in u32"));
    Ok(LimitTradeParametersFromQuote::try_from_limit(params)?)
}

/// Builds an EthFlow order id under a scripted collision sequence and reports the
/// resolved id plus how many scripted entries were left unconsumed.
async fn build_order_id(collisions: Vec<bool>) -> Result<(OrderUid, usize), Box<dyn Error>> {
    let trader = sample_trader_parameters();
    let signer = MockSigner::builder().address(sample_owner()).build();
    let app_data = build_app_data(&trader.app_code, 0, "market", None, None).await?;

    let queue = Arc::new(Mutex::new(collisions));
    let additional = PostTradeAdditionalParams::new().with_check_eth_flow_order_exists(Arc::new(
        ScriptedEthFlowChecker {
            collisions: queue.clone(),
        },
    ));

    let ethflow = get_eth_flow_transaction(
        &app_data.app_data_keccak256,
        &native_sell_params()?,
        trader.chain_id,
        &additional,
        &trader,
        &signer,
    )
    .await?;

    let remaining = queue
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .len();
    Ok((ethflow.order_id, remaining))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // A collision-free run and a run with two scripted collisions resolve to
    // different order ids; both drain their checker queue to empty.
    let (collision_free_id, collision_free_remaining) = build_order_id(vec![false]).await?;
    let (after_collisions_id, after_collisions_remaining) =
        build_order_id(vec![true, true, false]).await?;

    let report = json!({
        "surface": "cow-sdk::trading::EthFlowOrderExistsChecker",
        "mode": "simulated-transport",
        "collisionFreeOrderId": collision_free_id.to_hex_string(),
        "collisionFreeQueueRemaining": collision_free_remaining,
        "afterTwoCollisionsOrderId": after_collisions_id.to_hex_string(),
        "afterTwoCollisionsQueueRemaining": after_collisions_remaining,
        "collisionsChangedOrderId": collision_free_id != after_collisions_id
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
