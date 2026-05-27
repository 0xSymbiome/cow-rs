//! Live-services example: place a real WETH → COW order on Sepolia.
//!
//! Gated by the `live-services` cargo feature. Reads a Sepolia RPC
//! endpoint and private key from environment variables. Not part of
//! the deterministic example suite.
//!
//! # Required environment variables
//!
//! - `COW_SMOKE_LIVE_ORDER_CONFIRM` — must equal `yes` to opt in. Acts
//!   as a stale-shell guard: a forgotten env-var set from yesterday
//!   does not silently produce an on-chain order.
//! - `COW_SMOKE_LIVE_ORDER_RPC_URL` — Sepolia RPC endpoint.
//! - `COW_SMOKE_LIVE_ORDER_PRIVATE_KEY` — hex-encoded private key, with
//!   or without `0x` prefix.
//!
//! The wallet must hold at least `COW_SMOKE_LIVE_ORDER_SELL_AMOUNT_WEI`
//! (default `1_000_000_000_000_000` wei = 0.001 WETH) of Sepolia WETH,
//! plus some Sepolia ETH for the approval transaction gas.
//!
//! # Optional environment variables
//!
//! - `COW_SMOKE_LIVE_ORDER_SELL_AMOUNT_WEI` — default `1000000000000000`.
//! - `COW_SMOKE_LIVE_ORDER_SLIPPAGE_BPS` — default `50`.
//! - `COW_SMOKE_LIVE_ORDER_APP_CODE` — default `cow-rs-live-order-example`.
//! - `COW_SMOKE_LIVE_ORDER_POLL_SECONDS` — default `60`; set to `0` to
//!   skip the post-placement order-status loop.
//! - `COW_SMOKE_LIVE_ORDER_BUY_TOKEN` — default Sepolia COW
//!   `0x0625afb445c3b6b7b929342a04a22599fd5dbb59`. Override to use a
//!   different Sepolia ERC-20.
//!
//! # Safety
//!
//! Places a real on-chain ERC-20 `approve` call against the CoW
//! Protocol vault relayer (only when allowance is insufficient) and a
//! real signed CoW Protocol order on Sepolia.
//!
//! Chain-id parity is enforced by `AlloyClient::builder().build_checked()`,
//! which issues one `eth_chainId` call and aborts with
//! `AlloyClientBuilderError::ChainMismatch` if the RPC reports anything
//! other than Sepolia.
//!
//! The private key is read from the environment, wrapped in `Redacted`,
//! and never logged. The RPC URL is never logged because the path
//! segment may carry an API key.

use std::{env, error::Error, io, time::Duration};

use serde_json::json;

use cow_sdk::{
    SupportedChainId, TradingSdk,
    alloy::AlloyClient,
    core::{Address, Amount, OrderKind, Redacted, Signer, SigningProvider},
    orderbook::{ApiContext, CowEnv, OrderBookApi, OrderStatus},
    trading::{
        AllowanceParameters, ApprovalParameters, TradeParameters, WaitOptions,
        approval_transaction, submit_and_wait_for_receipt,
    },
};

const SEPOLIA_WETH: &str = "0xfff9976782d46cc05630d1f6ebab18b2324d6b14";
const SEPOLIA_COW: &str = "0x0625afb445c3b6b7b929342a04a22599fd5dbb59";

const DEFAULT_SELL_AMOUNT_WEI: &str = "1000000000000000";
const DEFAULT_SLIPPAGE_BPS: u32 = 50;
const DEFAULT_APP_CODE: &str = "cow-rs-live-order-example";
const DEFAULT_POLL_SECONDS: u64 = 60;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let confirm = required_env("COW_SMOKE_LIVE_ORDER_CONFIRM")?;
    if confirm != "yes" {
        return Err(io::Error::other(
            "COW_SMOKE_LIVE_ORDER_CONFIRM must equal `yes` to run this example. \
             This guard defends against stale shell environments — set the variable \
             explicitly when you intend to place a real order on Sepolia.",
        )
        .into());
    }

    let rpc_url = required_env("COW_SMOKE_LIVE_ORDER_RPC_URL")?;
    let private_key = Redacted::new(required_env("COW_SMOKE_LIVE_ORDER_PRIVATE_KEY")?);

    let app_code = optional_env("COW_SMOKE_LIVE_ORDER_APP_CODE")
        .unwrap_or_else(|| DEFAULT_APP_CODE.to_owned());
    let sell_amount_wei = optional_env("COW_SMOKE_LIVE_ORDER_SELL_AMOUNT_WEI")
        .unwrap_or_else(|| DEFAULT_SELL_AMOUNT_WEI.to_owned());
    let slippage_bps = optional_env("COW_SMOKE_LIVE_ORDER_SLIPPAGE_BPS")
        .and_then(|raw| raw.parse::<u32>().ok())
        .unwrap_or(DEFAULT_SLIPPAGE_BPS);
    let poll_seconds = optional_env("COW_SMOKE_LIVE_ORDER_POLL_SECONDS")
        .and_then(|raw| raw.parse::<u64>().ok())
        .unwrap_or(DEFAULT_POLL_SECONDS);
    let buy_token_str =
        optional_env("COW_SMOKE_LIVE_ORDER_BUY_TOKEN").unwrap_or_else(|| SEPOLIA_COW.to_owned());

    let sell_token = Address::new(SEPOLIA_WETH)
        .map_err(|err| io::Error::other(format!("Sepolia WETH address invalid: {err}")))?;
    let buy_token = Address::new(&buy_token_str).map_err(|err| {
        io::Error::other(format!(
            "COW_SMOKE_LIVE_ORDER_BUY_TOKEN must be a valid 0x-prefixed 20-byte hex \
             address (got `{buy_token_str}`): {err}"
        ))
    })?;
    let sell_amount = Amount::new(&sell_amount_wei).map_err(|err| {
        io::Error::other(format!(
            "COW_SMOKE_LIVE_ORDER_SELL_AMOUNT_WEI must be a decimal integer of wei \
             (got `{sell_amount_wei}`): {err}"
        ))
    })?;

    let client = AlloyClient::builder()
        .http(rpc_url.as_str())?
        .private_key(private_key.as_inner().as_str())?
        .chain_id(SupportedChainId::Sepolia)
        .build_checked()
        .await?;
    let signer = client.create_signer("live-order-sepolia").await?;
    let owner = signer.get_address().await?;

    let sdk = TradingSdk::builder()
        .with_chain_id(SupportedChainId::Sepolia)
        .with_app_code(app_code.as_str())
        .build_ready()?;
    let context = ApiContext::new(SupportedChainId::Sepolia, CowEnv::Prod);
    let orderbook = OrderBookApi::builder_from_context(context).build()?;

    let allowance_params = AllowanceParameters::new(sell_token, owner);
    let current_allowance = sdk
        .get_cow_protocol_allowance(&client, &allowance_params)
        .await?;

    let approval_tx_hash = if current_allowance < sell_amount {
        let approval_params = ApprovalParameters::new(sell_token, sell_amount);
        let tx = approval_transaction(&approval_params, SupportedChainId::Sepolia, CowEnv::Prod)?;
        let receipt =
            submit_and_wait_for_receipt(&signer, &client, &tx, WaitOptions::approve_default())
                .await
                .map_err(|err| io::Error::other(format!("approval wait failed: {err}")))?;
        Some(receipt.transaction_hash)
    } else {
        None
    };

    let trade = TradeParameters::new(OrderKind::Sell, sell_token, buy_token, sell_amount)
        .with_slippage_bps(slippage_bps);
    let result = sdk.post_swap_order(trade, &signer, None).await?;

    let initial_report = json!({
        "mode": "live",
        "chain_id": u64::from(SupportedChainId::Sepolia),
        "signer": owner.to_hex_string(),
        "allowance_before": current_allowance.to_string(),
        "approval_tx_hash": approval_tx_hash
            .as_ref()
            .map(|hash| hash.to_hex_string()),
        "order_id": result.order_id.to_hex_string(),
        "explorer_url": format!(
            "https://explorer.cow.fi/sepolia/orders/{}",
            result.order_id.to_hex_string()
        ),
    });
    println!("{}", serde_json::to_string_pretty(&initial_report)?);

    if poll_seconds > 0 {
        poll_order_status(&orderbook, &result.order_id, poll_seconds).await?;
    }

    Ok(())
}

async fn poll_order_status(
    orderbook: &OrderBookApi,
    order_id: &cow_sdk::orderbook::OrderUid,
    poll_seconds: u64,
) -> Result<(), Box<dyn Error>> {
    let interval = Duration::from_secs(5);
    let start = std::time::Instant::now();
    let deadline = start + Duration::from_secs(poll_seconds);
    loop {
        let elapsed = start.elapsed().as_secs();
        match orderbook.get_order(order_id).await {
            Ok(order) => {
                let report = json!({
                    "elapsed_secs": elapsed,
                    "status": order.status,
                });
                println!("{}", serde_json::to_string_pretty(&report)?);
                if matches!(
                    order.status,
                    OrderStatus::Fulfilled | OrderStatus::Cancelled | OrderStatus::Expired
                ) {
                    return Ok(());
                }
            }
            Err(err) => {
                let report = json!({
                    "elapsed_secs": elapsed,
                    "transient_lookup_error": err.to_string(),
                });
                println!("{}", serde_json::to_string_pretty(&report)?);
            }
        }
        if std::time::Instant::now() >= deadline {
            return Ok(());
        }
        tokio::time::sleep(interval).await;
    }
}

fn required_env(name: &str) -> Result<String, Box<dyn Error>> {
    optional_env(name)
        .ok_or_else(|| io::Error::other(format!("{name} must be set to run this example")).into())
}

fn optional_env(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.trim().is_empty())
}
