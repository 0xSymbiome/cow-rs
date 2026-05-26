use std::{
    error::Error,
    sync::{Arc, Mutex},
};

use cow_sdk::alloy::AlloyClient;
use cow_sdk::core::{
    Amount, AsyncSigningProvider, CowEnv, OrderUid, SupportedChainId, TransactionHash,
    TransactionStatus,
};
use cow_sdk::trading::{
    AllowanceParameters, ApprovalParameters, OrderTraderParameters, TradingSdk, WaitOptions,
    approval_transaction, submit_and_wait_for_receipt,
};
use serde_json::{Value, json};
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

const TEST_KEY: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
const HASH: &str = "0x13579bdf2468ace013579bdf2468ace013579bdf2468ace013579bdf2468ace0";
const ADDRESS: &str = "0x1111111111111111111111111111111111111111";
const COW: &str = "0x0625aFB445C3B6B7B929342a04A22599fd5dBB59";
const OWNER: &str = "0xc8c753Ee51E8Fc80e199AB297fB575634a1aC1d3";
const ORDER_UID: &str = "0xd64389693b6cf89ad6c140a113b10df08073e5ef3063d05a02f3f42e1a42f0ad0b7795e18767259cc253a2af471dbc4c72b49516ffffffff";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    let methods = mount_rpc(&server).await;
    // build_checked() verifies the configured chain id against the RPC endpoint.
    let client = AlloyClient::builder()
        .http(server.uri())?
        .private_key(TEST_KEY)?
        .chain_id(SupportedChainId::Mainnet)
        .build_checked()
        .await?;
    let signer = client.create_signer("local-key").await?;
    let sdk = TradingSdk::builder()
        .with_chain_id(SupportedChainId::Mainnet)
        .with_env(CowEnv::Prod)
        .build_helper_only()?;

    let allowance = sdk
        .get_cow_protocol_allowance(
            &client,
            &AllowanceParameters::new(address(COW), address(OWNER)),
        )
        .await?;
    assert_eq!(allowance, Amount::from(42u32));

    let approval_params = ApprovalParameters::new(address(COW), Amount::new("1000")?);
    let approval_tx =
        approval_transaction(&approval_params, SupportedChainId::Mainnet, CowEnv::Prod)?;
    let approval_receipt = submit_and_wait_for_receipt(
        &signer,
        &client,
        &approval_tx,
        WaitOptions::approve_default(),
    )
    .await?;
    assert_eq!(
        approval_receipt.transaction_hash,
        TransactionHash::new(HASH)?
    );
    assert_eq!(approval_receipt.status, Some(TransactionStatus::Success));

    let pre_sign = sdk
        .get_pre_sign_transaction(&OrderTraderParameters::new(order_uid()), &signer)
        .await?;
    assert_eq!(pre_sign.gas_limit, Some(Amount::from(25_200u32)));

    let methods = methods
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    let report = json!({
        "surface": "cow-sdk::alloy::AlloyClient with TradingSdk",
        "allowance": allowance,
        "approvalTxHash": approval_receipt.transaction_hash.to_hex_string(),
        "approvalStatus": format!("{:?}", approval_receipt.status),
        "approvalBlockNumber": approval_receipt.block_number,
        "approvalGasUsed": approval_receipt.gas_used,
        "preSignGasLimit": pre_sign.gas_limit,
        "rpcMethods": methods
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
        "eth_call" => json!(format!("0x{:0>64}", "2a")),
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

fn address(value: &str) -> cow_sdk::core::Address {
    cow_sdk::core::Address::new(value).unwrap()
}

fn order_uid() -> OrderUid {
    OrderUid::new(ORDER_UID).unwrap()
}
