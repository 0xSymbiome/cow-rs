#![cfg(not(target_arch = "wasm32"))]

use std::sync::{Arc, Mutex};

use cow_sdk_alloy::AlloyClient;
use cow_sdk_core::{
    Amount, CowEnv, HexData, OrderUid, SigningProvider, SupportedChainId, TransactionHash,
    TransactionRequest,
};
use cow_sdk_trading::{AllowanceParameters, ApprovalParameters, OrderTraderParameters, Trading};
use serde_json::{Value, json};
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

const TEST_KEY: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
const HASH: &str = "0x13579bdf2468ace013579bdf2468ace013579bdf2468ace013579bdf2468ace0";
const ADDRESS: &str = "0x1111111111111111111111111111111111111111";
const COW: &str = "0x0625aFB445C3B6B7B929342a04A22599fd5dBB59";
const OWNER: &str = "0xc8c753Ee51E8Fc80e199AB297fB575634a1aC1d3";
const ORDER_UID: &str = "0xd64389693b6cf89ad6c140a113b10df08073e5ef3063d05a02f3f42e1a42f0ad0b7795e18767259cc253a2af471dbc4c72b49516ffffffff";

#[tokio::test]
async fn alloy_client_satisfies_trading_sdk_boundaries() {
    let server = MockServer::start().await;
    let methods = mount_rpc(&server).await;
    let client = AlloyClient::builder()
        .http(server.uri())
        .unwrap()
        .private_key(TEST_KEY)
        .unwrap()
        .chain_id(SupportedChainId::Mainnet)
        .build()
        .await
        .unwrap();
    let signer = client.create_signer("local-key").await.unwrap();
    let sdk = Trading::builder()
        .chain_id(SupportedChainId::Mainnet)
        .env(CowEnv::Prod)
        .app_code("cow-rs/umbrella-composition-test")
        .build()
        .unwrap();

    let allowance = sdk
        .get_cow_protocol_allowance(
            &client,
            &AllowanceParameters::new(address(COW), address(OWNER)),
        )
        .await
        .unwrap();
    assert_eq!(allowance, Amount::from(42u32));

    let approval_hash = sdk
        .approve_cow_protocol(
            &signer,
            &ApprovalParameters::new(address(COW), Amount::new("1000").unwrap()),
        )
        .await
        .unwrap();
    assert_eq!(approval_hash, TransactionHash::new(HASH).unwrap());

    let pre_sign = sdk
        .get_pre_sign_transaction(&OrderTraderParameters::new(order_uid()), &signer)
        .await
        .unwrap();
    assert!(pre_sign.to.is_some());
    assert!(pre_sign.data.is_some());
    assert_eq!(pre_sign.value, Some(Amount::ZERO));
    assert_eq!(pre_sign.gas_limit, Some(Amount::from(25_200u32)));

    let methods = {
        let guard = methods
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.clone()
    };
    assert!(
        methods.iter().any(|method| method == "eth_call"),
        "{methods:?}"
    );
    assert!(
        methods
            .iter()
            .any(|method| method == "eth_sendRawTransaction"),
        "{methods:?}"
    );
    assert!(
        methods
            .iter()
            .all(|method| method != "eth_getTransactionReceipt"),
        "{methods:?}"
    );
    assert!(
        methods
            .iter()
            .filter(|method| method.as_str() == "eth_estimateGas")
            .count()
            >= 2,
        "{methods:?}"
    );
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

fn address(value: &str) -> cow_sdk_core::Address {
    cow_sdk_core::Address::new(value).unwrap()
}

fn order_uid() -> OrderUid {
    OrderUid::new(ORDER_UID).unwrap()
}

#[allow(
    dead_code,
    reason = "test-helper fixture stays available for transaction-request shape coverage even when the active assertions exercise only the helper subset that the current test rows need"
)]
fn sample_transaction() -> TransactionRequest {
    TransactionRequest::new(
        Some(address(COW)),
        Some(HexData::new("0x").unwrap()),
        Some(Amount::ZERO),
        None,
    )
}
