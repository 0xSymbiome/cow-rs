//! Demonstrates two transaction lifecycle shapes:
//! (A) submit and wait for one mined receipt through the trading helper, and
//! (B) broadcast once and keep receipt observation separate.

use std::{
    error::Error,
    sync::{Arc, Mutex},
};

use cow_sdk::alloy::AlloyClient;
use cow_sdk::core::{
    Address, Amount, Signer, SigningProvider, SupportedChainId, TransactionBroadcast,
    TransactionRequest, TransactionStatus,
};
use cow_sdk::trading::{WaitOptions, submit_and_wait_for_receipt};
use serde_json::{Value, json};
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

const TEST_KEY: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
const HASH: &str = "0x13579bdf2468ace013579bdf2468ace013579bdf2468ace013579bdf2468ace0";
const ADDRESS: &str = "0x1111111111111111111111111111111111111111";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    let methods = mount_rpc(&server).await;
    let client = AlloyClient::builder()
        .http(server.uri())?
        .private_key(TEST_KEY)?
        .chain_id(SupportedChainId::Mainnet)
        .build()
        .await?;
    let signer = client.create_signer("local-key").await?;
    let tx = self_transfer(&signer.get_address().await?);

    // Shape A: one helper call broadcasts once and returns the mined receipt.
    let helper_receipt =
        submit_and_wait_for_receipt(&signer, &client, &tx, WaitOptions::approve_default()).await?;
    assert_eq!(helper_receipt.status, Some(TransactionStatus::Success));

    // Shape B: one manual broadcast, with receipt observation left separate.
    let method_start = {
        methods
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    };
    let broadcast: TransactionBroadcast = signer.send_transaction(&tx).await?;

    let methods = methods
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    let shape_b_methods = &methods[method_start..];
    let report = json!({
        "surface": "transaction lifecycle",
        "shapeA": {
            "receiptHash": helper_receipt.transaction_hash.to_hex_string(),
            "status": format!("{:?}", helper_receipt.status),
            "blockNumber": helper_receipt.block_number,
            "gasUsed": helper_receipt.gas_used,
        },
        "shapeB": {
            "broadcastHash": broadcast.transaction_hash.to_hex_string(),
            "receiptRequestsDuringBroadcast": shape_b_methods
                .iter()
                .filter(|method| method.as_str() == "eth_getTransactionReceipt")
                .count()
        },
        "totalBroadcasts": methods
            .iter()
            .filter(|method| method.as_str() == "eth_sendRawTransaction")
            .count()
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

async fn mount_rpc(server: &MockServer) -> Arc<Mutex<Vec<String>>> {
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
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
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
    methods
}

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
        "eth_sendRawTransaction" => json!(HASH),
        "eth_getTransactionReceipt" => receipt_response(),
        _ => return Err(format!("unexpected JSON-RPC method `{method}`")),
    };
    Ok(result)
}

fn block_response(number: &str) -> Value {
    json!({
        "hash": HASH,
        "parentHash": HASH,
        "sha3Uncles": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
        "miner": ADDRESS,
        "stateRoot": HASH,
        "transactionsRoot": HASH,
        "receiptsRoot": HASH,
        "logsBloom": format!("0x{}", "00".repeat(256)),
        "difficulty": "0x0",
        "number": number,
        "gasLimit": "0x1c9c380",
        "gasUsed": "0x5208",
        "timestamp": "0x5",
        "extraData": "0x",
        "mixHash": HASH,
        "nonce": "0x0000000000000000",
        "baseFeePerGas": "0x1",
        "transactions": [],
        "uncles": [],
        "totalDifficulty": "0x0",
        "size": "0x1",
    })
}

fn receipt_response() -> Value {
    json!({
        "transactionHash": HASH,
        "transactionIndex": "0x0",
        "blockHash": HASH,
        "blockNumber": "0x2a",
        "from": ADDRESS,
        "to": ADDRESS,
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

fn self_transfer(address: &Address) -> TransactionRequest {
    TransactionRequest::new(
        Some(*address),
        None,
        Some(Amount::ZERO),
        Some(Amount::from(21_000u32)),
    )
}
