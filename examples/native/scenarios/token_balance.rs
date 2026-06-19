//! Read an ERC-20 token balance through the SDK's generic contract-read seam.
//!
//! Balance reads are a wallet-layer concern, not a protocol one: the orderbook
//! never needs them, so — matching the upstream TypeScript SDK, which ships a
//! typed allowance read but no balance read and leaves balances to the host
//! wallet stack — the SDK exposes no `balance_of` method. A native consumer
//! still needs to check funds before trading, and unlike a browser host it has
//! no built-in ERC-20 ABI, so this scenario shows the canonical pattern: the
//! same `Provider::read_contract` seam the SDK uses internally for its
//! `cow_protocol_allowance` read, pointed at the standard `balanceOf(address)`
//! view. Callers who prefer typed encode/decode can use the `IERC20` binding
//! the SDK re-exports through `cow_sdk::contracts` instead.
//!
//! Runs against a wiremock JSON-RPC endpoint, so it is deterministic and needs
//! no network and no key.

use std::error::Error;

use cow_sdk::alloy_provider::RpcAlloyProvider;
use cow_sdk::core::{Amount, ContractCall, Provider};
use cow_sdk_examples_native::support::{OWNER, WETH};
use serde_json::{Value, json};
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

// Minimal JSON ABI for the single `balanceOf(address) -> uint256` view. The
// SDK's own vault-relayer allowance read uses this same JSON-ABI-plus-args shape
// with `read_contract`; only the function and the argument differ.
const ERC20_BALANCE_OF_ABI: &str = r#"[{"type":"function","name":"balanceOf","inputs":[{"name":"account","type":"address"}],"outputs":[{"name":"","type":"uint256"}],"stateMutability":"view"}]"#;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // A deterministic JSON-RPC endpoint whose `eth_call` answers with a 2 WETH
    // balance encoded as a 32-byte `uint256` word.
    let balance_wei: u128 = 2_000_000_000_000_000_000;
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": format!("0x{balance_wei:064x}"),
        })))
        .mount(&server)
        .await;

    // The read-only Alloy provider leaf (no signer) pointed at the mock RPC.
    let provider = RpcAlloyProvider::builder().http(server.uri())?.build()?;

    // Encode `balanceOf(OWNER)` and read it through the generic seam. The args are
    // a JSON array of the call's inputs, exactly as the allowance read builds them.
    let args = serde_json::to_string(&(OWNER.to_hex_string(),))?;
    let raw = provider
        .read_contract(&ContractCall::new(
            WETH,
            "balanceOf".to_owned(),
            ERC20_BALANCE_OF_ABI.to_owned(),
            args,
        ))
        .await?;

    // `read_contract` returns the ABI-decoded return value as a JSON token; a
    // `uint256` balance arrives as a JSON string (or number). Parse it into a
    // typed `Amount`, then render it with WETH's 18 decimals.
    let balance = match serde_json::from_str::<Value>(&raw)? {
        Value::String(value) => Amount::new(value)?,
        Value::Number(value) => Amount::new(value.to_string())?,
        other => return Err(format!("unexpected balanceOf return shape: {other}").into()),
    };

    println!(
        "WETH balance of {OWNER}: {} ({} wei)",
        balance.format_units(18),
        balance
    );
    Ok(())
}
