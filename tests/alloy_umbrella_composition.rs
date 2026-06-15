#![cfg(not(target_arch = "wasm32"))]

use cow_sdk_alloy::AlloyClient;
use cow_sdk_core::{Amount, CowEnv, OrderUid, SigningProvider, SupportedChainId, TransactionHash};
use cow_sdk_trading::{AllowanceParams, ApprovalParams, OrderTraderParams, Trading};
use wiremock::MockServer;

#[path = "support/rpc.rs"]
mod support;
use support::{HASH, mount_rpc};

const TEST_KEY: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
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
    let trading = Trading::builder()
        .chain_id(SupportedChainId::Mainnet)
        .env(CowEnv::Prod)
        .app_code("cow-rs/umbrella-composition-test")
        .build()
        .unwrap();

    let allowance = trading
        .cow_protocol_allowance(&client, &AllowanceParams::new(address(COW), address(OWNER)))
        .await
        .unwrap();
    assert_eq!(allowance, Amount::from(42u32));

    let approval_hash = trading
        .approve_cow_protocol(
            &signer,
            &ApprovalParams::new(address(COW), Amount::new("1000").unwrap()),
        )
        .await
        .unwrap();
    assert_eq!(approval_hash, TransactionHash::new(HASH).unwrap());

    let pre_sign = trading
        .pre_sign_transaction(&OrderTraderParams::new(order_uid()), &signer)
        .await
        .unwrap();
    assert!(!pre_sign.to.is_zero());
    assert!(!pre_sign.data.as_slice().is_empty());
    assert_eq!(pre_sign.value, Amount::ZERO);
    assert_eq!(pre_sign.gas_limit, Amount::from(25_200u32));

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

fn address(value: &str) -> cow_sdk_core::Address {
    cow_sdk_core::Address::new(value).unwrap()
}

fn order_uid() -> OrderUid {
    OrderUid::new(ORDER_UID).unwrap()
}
