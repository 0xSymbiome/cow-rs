//! Minimal browser-wallet + trade example — Rust + Dioxus (web/wasm).
//!
//! The whole `cow-sdk-browser-wallet` contract on one screen:
//! **discover** an injected wallet (EIP-6963) → **connect** → **sign** → **wrap**
//! / **approve** / **quote** / **swap** a CoW order, either direction between
//! WETH and COW on Sepolia.
//!
//! The shape to copy: a UI component (`App`) that only manipulates signals, and
//! a set of small `async` SDK functions below it that each do one thing and
//! return `anyhow::Result`. The wallet, signer, and trading client are the SDK's
//! public types — no JavaScript and no raw RPC.

use std::future::Future;
use std::sync::Arc;

use anyhow::Result;
use dioxus::prelude::*;

use cow_sdk::browser_wallet::{BrowserWallet, Eip1193Signer};
use cow_sdk::contracts::wrap_interaction;
use cow_sdk::core::{HexData, OrderKind, TransactionRequest, wrapped_native_token};
use cow_sdk::orderbook::ApiContext;
use cow_sdk::prelude::{
    Address, Amount, CowEnv, OrderbookApi, Signer, SupportedChainId, TradeParameters, Trading,
};
use cow_sdk::trading::{ApprovalParameters, TradingOptions, approval_transaction};

const CHAIN: SupportedChainId = SupportedChainId::Sepolia;
// CoW's Sepolia liquidity lives on the production API (`api.cow.fi/sepolia`);
const ENV: CowEnv = CowEnv::Prod;
const APP_CODE: &str = "cow-rs/dioxus-example";
/// COW token on Sepolia — CoW's canonical test pair with WETH.
const COW_SEPOLIA: &str = "0x0625afb445c3b6b7b929342a04a22599fd5dbb59";

/// A discovered injected wallet: its provider handle and display label.
#[derive(Clone)]
struct Found {
    wallet: BrowserWallet,
    label: String,
}

fn main() {
    console_error_panic_hook::set_once();
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Only `sell_is_weth` and `amount` are mutated directly in the UI; the rest
    // are mutated by the helper functions they're passed to.
    let wallet = use_signal(|| Option::<BrowserWallet>::None);
    let discovered = use_signal(Vec::<Found>::new);
    let mut sell_is_weth = use_signal(|| true);
    let mut amount = use_signal(|| "0.01".to_string());
    let output = use_signal(String::new);
    let busy = use_signal(|| false);

    // Everything below is derived from the signals above — no duplicated state.
    let connected = wallet().is_some();
    let session_text = wallet().as_ref().map(summarize).unwrap_or_default();
    let sell_symbol = if sell_is_weth() { "WETH" } else { "COW" };
    let wallet_list: Vec<(usize, String)> = discovered
        .read()
        .iter()
        .enumerate()
        .map(|(index, found)| (index, found.label.clone()))
        .collect();

    rsx! {
        style { {CSS} }
        main { class: "card",
            h1 { "CoW · Browser-Wallet Trade" }
            p { class: "muted",
                "Discover an injected wallet (EIP-6963), connect, sign, and swap WETH↔COW on Sepolia — all in Rust with Dioxus."
            }

            // 1 — Connect: discover wallets, then pick one (MetaMask, Phantom, …).
            div { class: "step",
                div { class: "step-title", "1 · Connect" }
                div { class: "row",
                    button {
                        disabled: busy(),
                        onclick: move |_| async move {
                            run_discover(discovered, busy, output).await;
                        },
                        "Discover wallets"
                    }
                }
                for (index, label) in wallet_list {
                    div { class: "row",
                        button {
                            class: "ghost",
                            disabled: busy(),
                            onclick: move |_| async move {
                                let chosen = discovered.peek().get(index).map(|f| f.wallet.clone());
                                let Some(active) = chosen else { return };
                                connect_active(wallet, busy, output, active).await;
                            },
                            "Connect: {label}"
                        }
                    }
                }
                if !session_text.is_empty() {
                    p { class: "mono session", "{session_text}" }
                }
            }

            // 2 — Sign: prove the wallet signs.
            div { class: "step",
                div { class: "step-title", "2 · Sign" }
                button {
                    disabled: busy() || !connected,
                    onclick: move |_| async move {
                        let Some(active) = wallet() else { return };
                        run(busy, output, sign_message(&active)).await;
                    },
                    "Sign message (personal_sign)"
                }
            }

            // 3 — Trade.
            div { class: "step",
                div { class: "step-title", "3 · Trade" }
                div { class: "row",
                    select {
                        value: if sell_is_weth() { "weth-cow" } else { "cow-weth" },
                        onchange: move |event| {
                            let weth = event.value() == "weth-cow";
                            sell_is_weth.set(weth);
                            // The fee is a fixed gas cost charged in the sell token
                            // (~0.04 COW), so a 0.01 COW sell is below it and gets
                            // "no route found" — default to a larger COW amount.
                            amount.set(if weth { "0.01".to_string() } else { "1".to_string() });
                        },
                        option { value: "weth-cow", "WETH → COW" }
                        option { value: "cow-weth", "COW → WETH" }
                    }
                }
                div { class: "row",
                    label { class: "muted", "Sell amount ({sell_symbol})" }
                    input {
                        value: "{amount}",
                        oninput: move |event| amount.set(event.value()),
                    }
                }
                div { class: "row",
                    button {
                        class: "ghost",
                        disabled: busy() || !connected,
                        onclick: move |_| async move {
                            let Some(active) = wallet() else { return };
                            run(busy, output, wrap(&active, &amount())).await;
                        },
                        "Wrap ETH→WETH"
                    }
                    button {
                        class: "ghost",
                        disabled: busy() || !connected,
                        onclick: move |_| async move {
                            let Some(active) = wallet() else { return };
                            run(busy, output, approve(&active, sell_is_weth(), &amount())).await;
                        },
                        "Approve sell token"
                    }
                    button {
                        disabled: busy() || !connected,
                        onclick: move |_| async move {
                            let Some(active) = wallet() else { return };
                            run(busy, output, quote(&active, sell_is_weth(), &amount())).await;
                        },
                        "Get quote"
                    }
                    button {
                        class: "primary",
                        disabled: busy() || !connected,
                        onclick: move |_| async move {
                            let Some(active) = wallet() else { return };
                            run(busy, output, swap(&active, sell_is_weth(), &amount())).await;
                        },
                        "Sign & submit swap"
                    }
                }
                p { class: "muted small",
                    "Both directions work, but the fee is a fixed gas cost in the sell token (~0.04 COW), so selling COW needs a larger amount (≥ ~0.1). If you only hold ETH: Wrap → Approve → swap."
                }
            }

            if busy() {
                p { class: "muted", "working…" }
            }
            if !output().is_empty() {
                pre { class: "output", "{output}" }
            }
        }
    }
}

// --- UI plumbing -----------------------------------------------------------

/// Runs an SDK action that yields a status string, routing the result (or a
/// formatted error) to `output` and toggling the global `busy` flag.
async fn run(
    mut busy: Signal<bool>,
    mut output: Signal<String>,
    action: impl Future<Output = Result<String>>,
) {
    busy.set(true);
    output.set(match action.await {
        Ok(text) => text,
        Err(error) => format!("error: {error:#}"),
    });
    busy.set(false);
}

/// Connects a wallet and makes it the active one.
async fn connect_active(
    mut wallet: Signal<Option<BrowserWallet>>,
    mut busy: Signal<bool>,
    mut output: Signal<String>,
    candidate: BrowserWallet,
) {
    busy.set(true);
    match candidate.connect().await {
        Ok(_) => {
            wallet.set(Some(candidate));
            output.set("wallet connected".to_string());
        }
        Err(error) => output.set(format!("error: {error:#}")),
    }
    busy.set(false);
}

/// Runs EIP-6963 discovery and stores the discovered wallets.
async fn run_discover(
    mut discovered: Signal<Vec<Found>>,
    mut busy: Signal<bool>,
    mut output: Signal<String>,
) {
    busy.set(true);
    match discover_wallets().await {
        Ok(found) => {
            output.set(if found.is_empty() {
                "no injected wallet found — install/enable one and reload".to_string()
            } else {
                format!("found {} wallet(s) — pick one below", found.len())
            });
            discovered.set(found);
        }
        Err(error) => output.set(format!("error: {error:#}")),
    }
    busy.set(false);
}

// --- SDK calls -------------------------------------------------------------

/// EIP-6963 discovery: returns every injected provider with its label so the
/// user can choose which to connect.
async fn discover_wallets() -> Result<Vec<Found>> {
    let discovery = BrowserWallet::discover().await?;
    let mut found = Vec::new();
    for (index, info) in discovery.wallets().into_iter().enumerate() {
        found.push(Found {
            wallet: discovery.wallet_at(index)?,
            label: info.provider_label,
        });
    }
    Ok(found)
}

/// `personal_sign` through the wallet.
async fn sign_message(wallet: &BrowserWallet) -> Result<String> {
    let signer = wallet.signer();
    Ok(signer.sign_message(b"cow-rs dioxus example").await?)
}

/// Price-only quote. Read-only, so it uses the chainless signer (just the
/// account) and never touches the wallet's network.
async fn quote(wallet: &BrowserWallet, sell_is_weth: bool, amount: &str) -> Result<String> {
    let signer = wallet.signer();
    let results = build_trading()?
        .get_quote_results(trade(amount, sell_is_weth)?, &signer, None)
        .await?;
    Ok(serde_json::to_string_pretty(&results)?)
}

/// Quote, EIP-712 sign in the wallet, and post the swap order.
async fn swap(wallet: &BrowserWallet, sell_is_weth: bool, amount: &str) -> Result<String> {
    let signer = signer_for(wallet).await?;
    let posting = build_trading()?
        .post_swap_order(trade(amount, sell_is_weth)?, &signer, None)
        .await?;
    Ok(format!(
        "order posted ✓\nuid: {}",
        posting.order_id.to_hex_string()
    ))
}

/// Approves the sell token for the CoW vault relayer — a sell order can't settle
/// until its sell token is approved. Sends an on-chain `approve` tx.
async fn approve(wallet: &BrowserWallet, sell_is_weth: bool, amount: &str) -> Result<String> {
    let signer = signer_for(wallet).await?;
    let approval = ApprovalParameters::new(sell_token(sell_is_weth)?, parse_amount(amount)?)
        .with_chain_id(CHAIN);
    let tx = approval_transaction(&approval, CHAIN, ENV)?;
    let broadcast = signer.send_transaction(&tx).await?;
    Ok(serde_json::to_string_pretty(&broadcast)?)
}

/// Wraps native ETH into WETH (`deposit()`), giving you a tradeable sell token —
/// useful when you only hold ETH.
async fn wrap(wallet: &BrowserWallet, amount: &str) -> Result<String> {
    let signer = signer_for(wallet).await?;
    let interaction = wrap_interaction(wrapped_native_token(CHAIN).address, parse_amount(amount)?);
    let tx = TransactionRequest::new(
        Some(interaction.target),
        Some(HexData::from_bytes(interaction.call_data)),
        Some(interaction.value),
        None,
    );
    let broadcast = signer.send_transaction(&tx).await?;
    Ok(serde_json::to_string_pretty(&broadcast)?)
}

/// Chain-validated signer. If the wallet is on another network, ask it to switch
/// (`wallet_switchEthereumChain`) first, then validate — `signer_for_chain` is
/// fail-closed and never signs on the wrong chain.
async fn signer_for(wallet: &BrowserWallet) -> Result<Eip1193Signer> {
    if wallet.chain_id() != Some(u64::from(CHAIN)) {
        wallet.switch_chain(CHAIN).await?;
    }
    Ok(wallet.signer_for_chain(CHAIN).await?)
}

/// The (sell, buy) token addresses for the chosen direction. WETH and COW are
/// both 18-decimal ERC-20s on Sepolia.
fn token_pair(sell_is_weth: bool) -> Result<(Address, Address)> {
    let weth = wrapped_native_token(CHAIN).address;
    let cow = Address::new(COW_SEPOLIA)?;
    Ok(if sell_is_weth {
        (weth, cow)
    } else {
        (cow, weth)
    })
}

fn sell_token(sell_is_weth: bool) -> Result<Address> {
    Ok(token_pair(sell_is_weth)?.0)
}

/// Parses a human amount (e.g. `1.5`) into atoms; WETH and COW are 18 decimals.
fn parse_amount(amount: &str) -> Result<Amount> {
    Ok(Amount::parse_units(amount.trim(), 18)?)
}

/// Sells `amount` of the chosen sell token for the other one.
///
/// `with_slippage_bps(50)` is an explicit 0.5% tolerance the SDK signs verbatim;
/// omit it for AUTO slippage, where the SDK applies the quote's
/// `suggested_slippage_bps` instead (fee/volume-aware — well above 50 bps on
/// small, fee-heavy trades). A tighter bound prices better but can expire unfilled.
fn trade(amount: &str, sell_is_weth: bool) -> Result<TradeParameters> {
    let (sell, buy) = token_pair(sell_is_weth)?;
    Ok(
        TradeParameters::new(OrderKind::Sell, sell, buy, parse_amount(amount)?)
            .with_slippage_bps(50)
            .with_valid_for(1800),
    )
}

/// Trading client backed by the live CoW orderbook over the browser `fetch`
/// transport. Rebuilt per action so the example stays stateless and obvious.
fn build_trading() -> Result<Trading> {
    let context = ApiContext::new(CHAIN, ENV);

    #[cfg(target_arch = "wasm32")]
    let orderbook = {
        use cow_sdk::core::HttpTransport;
        use cow_sdk_transport_wasm::{FetchTransport, FetchTransportConfig};
        let base_url = context.resolved_base_url()?;
        let transport: Arc<dyn HttpTransport + Send + Sync> =
            Arc::new(FetchTransport::new(&FetchTransportConfig::new(base_url)));
        OrderbookApi::builder_from_context(context)
            .transport(transport)
            .build()?
    };
    #[cfg(not(target_arch = "wasm32"))]
    let orderbook = OrderbookApi::builder_from_context(context).build()?;

    Ok(Trading::builder()
        .chain_id(CHAIN)
        .app_code(APP_CODE)
        .options(TradingOptions::new().with_orderbook_client(Arc::new(orderbook)))
        .build()?)
}

/// `account · chain · wallet-label` from the wallet's current session.
fn summarize(wallet: &BrowserWallet) -> String {
    let session = wallet.session();
    let account = session
        .selected_account
        .map(|address| address.to_hex_string())
        .unwrap_or_else(|| "—".to_string());
    let chain = session
        .chain_id
        .map(|id| id.to_string())
        .unwrap_or_else(|| "—".to_string());
    format!("{account}  ·  chain {chain}  ·  {}", session.wallet_label)
}

const CSS: &str = r#"
body { margin:0; background:#0f1115; color:#e6e9ef;
  font-family: ui-sans-serif, system-ui, 'Segoe UI', Roboto, sans-serif; }
.card { max-width:560px; margin:40px auto; padding:24px; background:#171a21;
  border:1px solid #2a2f3a; border-radius:14px; }
h1 { font-size:18px; margin:0 0 6px; }
.muted { color:#9aa3b2; }
.small { font-size:12px; }
.step { margin-top:18px; padding-top:14px; border-top:1px solid #2a2f3a; }
.step-title { font-weight:600; margin-bottom:10px; }
.row { display:flex; gap:10px; align-items:center; flex-wrap:wrap; margin:6px 0; }
button { background:#1f232c; color:#e6e9ef; border:1px solid #2a2f3a; border-radius:8px;
  padding:9px 14px; font-size:13px; cursor:pointer; }
button:hover:enabled { border-color:#5b8cff; }
button:disabled { opacity:.5; cursor:not-allowed; }
button.primary { background:#5b8cff; border-color:#5b8cff; color:#fff; font-weight:600; }
button.ghost { background:transparent; }
input { background:#0f1115; border:1px solid #2a2f3a; border-radius:8px; color:#e6e9ef;
  padding:8px 10px; font-size:13px; flex:1; min-width:120px; }
.mono, .output { font-family: ui-monospace, Menlo, monospace; font-size:12px; }
.session { color:#18c08f; }
.output { background:#0f1115; border:1px solid #2a2f3a; border-radius:8px; padding:12px;
  margin-top:14px; white-space:pre-wrap; word-break:break-word; max-height:300px; overflow:auto; }
"#;
