//! Demonstrates the broadcast / receipt lifecycle: build a self-transfer,
//! broadcast it once via `send_transaction`, print the broadcast hash, then
//! exit. The receipt-polling helper is shown in a follow-up release once the
//! trading helper crate exposes it.

use std::{error::Error, sync::{Arc, Mutex}};

use cow_sdk::alloy::AlloyClient;
use cow_sdk::core::{
    Address, Amount, AsyncSigner, AsyncSigningProvider, SupportedChainId, TransactionBroadcast,
    TransactionRequest,
};
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

    let broadcast: TransactionBroadcast = signer.send_transaction(&tx).await?;

    let methods = methods
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    let report = json!({
        "surface": "transaction lifecycle",
        "broadcastHash": broadcast.transaction_hash.as_str(),
        "receiptRequestsDuringBroadcast": methods
            .iter()
            .filter(|method| method.as_str() == "eth_getTransactionReceipt")
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

fn self_transfer(address: &Address) -> TransactionRequest {
    TransactionRequest::new(
        Some(address.clone()),
        None,
        Some(Amount::zero()),
        Some(Amount::from(21_000u32)),
    )
}
