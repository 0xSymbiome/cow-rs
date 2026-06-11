//! Shared fixtures and domain helpers for the native example scenarios.
//!
//! Trait doubles (`OrderbookClient` / `Signer` / `Provider`) come from the
//! published `cow_sdk::testing` crate; this module keeps only the example-domain
//! constants, parameter builders, and wire fixtures the scenarios share.

use std::{
    env,
    error::Error,
    io,
    sync::{Arc, Mutex, PoisonError},
};

use serde_json::{Value, json};

use cow_sdk::core::{
    Address, Amount, BuyTokenDestination, CowEnv, HexData, OrderData, OrderKind, OrderUid,
    SellTokenSource, SupportedChainId, address,
};
use cow_sdk::orderbook::{AppDataHash, Order, OrderQuoteResponse};
use cow_sdk::trading::{
    LimitTradeParams, TradeParams, TraderParams, swap_params_to_limit_order_params,
};
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

// Compile-time validated address constants in the canonical lowercase wire
// form (`address!` rejects malformed or mixed-case literals at build time).
// `Address` serializes to the same lowercase wire string, so the JSON
// fixtures below embed the constants directly.
pub const WETH: Address = address!("0xfff9976782d46cc05630d1f6ebab18b2324d6b14");
pub const COW: Address = address!("0x0625afb445c3b6b7b929342a04a22599fd5dbb59");
pub const OWNER: Address = address!("0xc8c753ee51e8fc80e199ab297fb575634a1ac1d3");
pub const ALT_RECEIVER: Address = address!("0x974caa59e49682cda0ad2bbe82983419a2ecc400");
pub const SETTLEMENT: Address = address!("0x9008d19f58aabd9ed0d60971565aa8510560ab41");
pub const ORDER_UID: &str = "0xd64389693b6cf89ad6c140a113b10df08073e5ef3063d05a02f3f42e1a42f0ad0b7795e18767259cc253a2af471dbc4c72b49516ffffffff";
pub const APP_DATA_HASH: &str =
    "0xe269b09f45b1d3c98d8e4e841b99a0779fbd3b77943d069b91ddc4fd9789e27e";
pub const TYPED_SIGNATURE: &str = "0x111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111b";
pub const MESSAGE_SIGNATURE: &str = "0x222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222221c";
pub const TX_HASH: &str = "0x13579bdf2468ace013579bdf2468ace013579bdf2468ace013579bdf2468ace0";

/// Well-known Anvil/Hardhat development private key (a public test key, never a
/// real secret) used to build the local signer in the Alloy scenarios.
pub const TEST_KEY: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";

/// Filler account address for the synthetic JSON-RPC block and receipt fixtures.
const RPC_FIXTURE_ACCOUNT: &str = "0x1111111111111111111111111111111111111111";

pub fn sample_order_uid() -> OrderUid {
    OrderUid::new(ORDER_UID).expect("example order uid literal must remain valid")
}

pub fn sample_app_data_hash() -> AppDataHash {
    AppDataHash::new(APP_DATA_HASH).expect("example app-data hash must remain valid")
}

pub fn text_preview(value: &str, max_chars: usize) -> &str {
    if max_chars == 0 {
        return "";
    }

    value
        .char_indices()
        .nth(max_chars)
        .map_or(value, |(index, _)| &value[..index])
}

/// A short `0x`-prefixed preview of contract call data for the JSON reports.
pub fn call_data_prefix(data: &HexData) -> String {
    text_preview(&data.to_hex_string(), 10).to_owned()
}

/// The environment variable's value, when set to something non-blank.
pub fn optional_env(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.trim().is_empty())
}

/// The environment variable's value, failing with a configuration hint when
/// it is unset or blank.
pub fn required_env(name: &str) -> Result<String, Box<dyn Error>> {
    optional_env(name).ok_or_else(|| {
        io::Error::other(format!(
            "{name} is required for this live example. Configure it explicitly before running."
        ))
        .into()
    })
}

/// A [`CowEnv`] parsed from the environment, defaulting to production.
pub fn optional_cow_env(name: &str) -> Result<CowEnv, Box<dyn Error>> {
    let Some(raw_value) = optional_env(name) else {
        return Ok(CowEnv::Prod);
    };

    match raw_value.to_ascii_lowercase().as_str() {
        "prod" | "production" => Ok(CowEnv::Prod),
        "staging" | "barn" => Ok(CowEnv::Staging),
        other => Err(io::Error::other(format!(
            "{name} must be one of prod or staging. Received {other}."
        ))
        .into()),
    }
}

/// A [`SupportedChainId`] parsed from the environment, defaulting to mainnet.
pub fn optional_supported_chain_id(name: &str) -> Result<SupportedChainId, Box<dyn Error>> {
    let Some(raw_value) = optional_env(name) else {
        return Ok(SupportedChainId::Mainnet);
    };
    let chain_id: u64 = raw_value.parse()?;
    SupportedChainId::try_from(chain_id).map_err(|error| {
        io::Error::other(format!(
            "{name} must be a supported chain id. Received {chain_id}: {error}"
        ))
        .into()
    })
}

pub fn orderbook_version_response(version: &str) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_raw(version.as_bytes(), "text/plain; charset=utf-8")
}

pub fn sample_unsigned_order() -> OrderData {
    // `OrderData` has public fields by design: the struct literal is the
    // named-arguments surface, so amounts and addresses cannot transpose.
    OrderData {
        sell_token: WETH,
        buy_token: COW,
        receiver: ALT_RECEIVER,
        sell_amount: Amount::parse_units("0.1", 18).expect("example sell amount must remain valid"),
        buy_amount: Amount::parse_units("0.25", 18).expect("example buy amount must remain valid"),
        valid_to: 1_700_000_000,
        app_data: AppDataHash::new(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )
        .expect("example app-data hash must remain valid"),
        fee_amount: Amount::ZERO,
        kind: OrderKind::Sell,
        partially_fillable: false,
        sell_token_balance: SellTokenSource::Erc20,
        buy_token_balance: BuyTokenDestination::Erc20,
    }
}

pub fn sample_trade_parameters() -> TradeParams {
    TradeParams::new(
        OrderKind::Sell,
        WETH,
        COW,
        Amount::parse_units("0.1", 18).expect("example trade amount must remain valid"),
    )
    .with_owner(OWNER)
    .with_slippage_bps(50)
}

pub fn sample_limit_parameters() -> LimitTradeParams {
    // The public swap-to-limit conversion threads the quote amounts, balance
    // flavours, and quote id; no hand-copied quote fields.
    swap_params_to_limit_order_params(&sample_trade_parameters(), &sample_quote_response())
        .expect("example quote fixture carries a quote id")
        .into_limit()
}

pub fn sample_trader_parameters() -> TraderParams {
    TraderParams::new(SupportedChainId::Sepolia, "cow-rs-native-examples")
        .expect("app code should validate")
}

pub fn sample_quote_response() -> OrderQuoteResponse {
    serde_json::from_value(sample_quote_response_json())
        .expect("example quote response fixture must deserialize")
}

pub fn sample_quote_response_json() -> serde_json::Value {
    json!({
        "quote": {
            "sellToken": WETH,
            "buyToken": COW,
            "receiver": OWNER,
            "sellAmount": "98646335338956442",
            "buyAmount": "30000000000000000000",
            "validTo": 1737464594u32,
            "appData": APP_DATA_HASH,
            "feeAmount": "1353664661043558",
            "kind": "sell",
            "partiallyFillable": false,
            "sellTokenBalance": "erc20",
            "buyTokenBalance": "erc20"
        },
        "from": OWNER,
        "expiration": "2025-01-21T12:55:14.799709609Z",
        "id": 575401,
        "verified": true
    })
}

pub fn sample_signature() -> &'static str {
    TYPED_SIGNATURE
}

pub fn sample_open_order() -> Order {
    serde_json::from_value(json!({
        "sellToken": WETH,
        "buyToken": COW,
        "receiver": OWNER,
        "sellAmount": "1000000000000000000",
        "buyAmount": "500000000000000000",
        "validTo": 1234567890u32,
        "appData": APP_DATA_HASH,
        "feeAmount": "10000000000000000",
        "kind": "sell",
        "partiallyFillable": false,
        "sellTokenBalance": "erc20",
        "buyTokenBalance": "erc20",
        "signingScheme": "eip712",
        "signature": TYPED_SIGNATURE,
        "class": "market",
        "owner": OWNER,
        "uid": ORDER_UID,
        "settlementContract": SETTLEMENT,
        "executedSellAmount": "0",
        "executedBuyAmount": "0",
        "invalidated": false,
        "status": "open",
        "totalFee": "0"
    }))
    .expect("example order fixture must deserialize")
}

/// Handle onto the JSON-RPC method names recorded by [`mount_rpc`], sharing
/// one backing store with the mounted handler.
#[derive(Clone, Debug)]
pub struct RecordedRpc(Arc<Mutex<Vec<String>>>);

impl RecordedRpc {
    /// A snapshot of the JSON-RPC method names recorded so far.
    #[must_use]
    pub fn methods(&self) -> Vec<String> {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

/// Mounts a wiremock JSON-RPC `POST` handler that records every method it sees
/// and replays a canned result for it. The returned handle lets a scenario
/// report the exact RPC calls the SDK made.
pub async fn mount_rpc(server: &MockServer) -> RecordedRpc {
    let methods = Arc::new(Mutex::new(Vec::new()));
    Mock::given(method("POST"))
        .respond_with({
            let methods = Arc::clone(&methods);
            move |request: &wiremock::Request| {
                let body = request.body_json::<Value>().unwrap();
                let method_name = body
                    .get("method")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
                    .to_owned();
                methods
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .push(method_name.clone());
                let id = body.get("id").cloned().unwrap_or_else(|| json!(1));

                match rpc_result(&method_name) {
                    Ok(result) => ResponseTemplate::new(200).set_body_json(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "result": result,
                    })),
                    Err(message) => ResponseTemplate::new(200).set_body_json(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": {
                            "code": -32601,
                            "message": message,
                        },
                    })),
                }
            }
        })
        .mount(server)
        .await;
    RecordedRpc(methods)
}

/// Canned JSON-RPC results for the methods the Alloy scenarios exercise. The set
/// is a superset across the scenarios; an arm a given scenario never triggers is
/// simply unused by it.
fn rpc_result(method: &str) -> Result<Value, String> {
    let result = match method {
        "eth_chainId" => json!("0x1"),
        "eth_getTransactionCount" => json!("0x0"),
        "eth_estimateGas" => json!("0x5208"),
        "eth_gasPrice" | "eth_maxPriorityFeePerGas" => json!("0x3b9aca00"),
        "eth_feeHistory" => json!({
            "oldestBlock": "0x1",
            "baseFeePerGas": ["0x3b9aca00", "0x3b9aca00"],
            "gasUsedRatio": [0.1],
            "reward": [["0x3b9aca00"]],
        }),
        "eth_getBlockByNumber" => block_response("0x2a"),
        "eth_sendRawTransaction" => json!(TX_HASH),
        "eth_getTransactionReceipt" => receipt_response(),
        "eth_call" => json!(format!("0x{:0>64}", "2a")),
        _ => return Err(format!("unexpected JSON-RPC method `{method}`")),
    };
    Ok(result)
}

/// A synthetic Ethereum block body for `eth_getBlockByNumber`.
fn block_response(number: &str) -> Value {
    json!({
        "hash": TX_HASH,
        "parentHash": TX_HASH,
        "sha3Uncles": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
        "miner": RPC_FIXTURE_ACCOUNT,
        "stateRoot": TX_HASH,
        "transactionsRoot": TX_HASH,
        "receiptsRoot": TX_HASH,
        "logsBloom": format!("0x{}", "00".repeat(256)),
        "difficulty": "0x0",
        "number": number,
        "gasLimit": "0x1c9c380",
        "gasUsed": "0x5208",
        "timestamp": "0x5",
        "extraData": "0x",
        "mixHash": TX_HASH,
        "nonce": "0x0000000000000000",
        "baseFeePerGas": "0x1",
        "transactions": [],
        "uncles": [],
        "totalDifficulty": "0x0",
        "size": "0x1",
    })
}

/// A synthetic successful transaction receipt for `eth_getTransactionReceipt`.
fn receipt_response() -> Value {
    json!({
        "transactionHash": TX_HASH,
        "transactionIndex": "0x0",
        "blockHash": TX_HASH,
        "blockNumber": "0x2a",
        "from": RPC_FIXTURE_ACCOUNT,
        "to": RPC_FIXTURE_ACCOUNT,
        "contractAddress": null,
        "gasUsed": "0x5208",
        "effectiveGasPrice": "0x1",
        "cumulativeGasUsed": "0x5208",
        "logsBloom": format!("0x{}", "00".repeat(256)),
        "status": "0x1",
        "logs": [],
        "type": "0x2"
    })
}
