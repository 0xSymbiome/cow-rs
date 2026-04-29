#![allow(dead_code)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::iter_on_single_items,
    clippy::missing_const_for_fn,
    clippy::option_if_let_else,
    clippy::redundant_clone,
    clippy::too_many_lines,
    clippy::unnested_or_patterns,
    reason = "pedantic, nursery, and perf lints acceptable in test helper code"
)]

use serde_json::{Value, json};

use cow_sdk_orderbook::{
    Address, ApiContext, AppDataHash, CowEnv, ExternalHostPolicy, OrderBookApi,
    OrderBookTransportPolicy, OrderUid, SupportedChainId,
};

pub fn address(value: &str) -> Address {
    Address::new(value).expect("test address literal must be valid")
}

pub fn app_data_hash(value: &str) -> AppDataHash {
    AppDataHash::new(value).expect("test app-data hash literal must be valid")
}

pub fn order_uid(value: &str) -> OrderUid {
    OrderUid::new(value).expect("test order uid literal must be valid")
}

pub fn default_context(chain_id: SupportedChainId, env: CowEnv) -> ApiContext {
    ApiContext::new(chain_id, env)
}

pub fn build_orderbook_api(context: ApiContext) -> OrderBookApi {
    OrderBookApi::builder_from_context(context)
        .build()
        .expect("default orderbook test client must build")
}

pub fn build_orderbook_api_with_base_url(
    context: ApiContext,
    base_url: impl Into<String>,
) -> OrderBookApi {
    OrderBookApi::builder_from_context(context)
        .with_external_host_policy(ExternalHostPolicy::Test)
        .base_url(base_url)
        .build()
        .expect("orderbook test client with loopback base URL must build")
}

pub fn build_orderbook_api_with_policy(
    context: ApiContext,
    policy: OrderBookTransportPolicy,
) -> OrderBookApi {
    OrderBookApi::builder_from_context(context)
        .policy(policy)
        .build()
        .expect("orderbook test client with custom policy must build")
}

pub fn build_orderbook_api_with_shared_client(
    client: reqwest::Client,
    context: ApiContext,
) -> OrderBookApi {
    OrderBookApi::builder_from_context(context)
        .with_external_host_policy(ExternalHostPolicy::Test)
        .client(client)
        .build()
        .expect("orderbook test client with shared client must build")
}

pub fn sample_order_uid() -> OrderUid {
    order_uid(
        "0x59920c85de0162e9e55df8d396e75f3b6b7c2dfdb535f03e5c807731c31585eaff714b8b0e2700303ec912bd40496c3997ceea2b616d6710",
    )
}

pub fn sample_app_data_hash() -> AppDataHash {
    app_data_hash("0x0000000000000000000000000000000000000000000000000000000000000000")
}

pub fn sample_owner() -> Address {
    address("0x6810e776880c02933d47db1b9fc05908e5386b96")
}

pub fn sample_buy_token() -> Address {
    address("0x1111111111111111111111111111111111111111")
}

pub fn sample_signature() -> &'static str {
    "0x4d306ce7c770d22005bcfc00223f8d9aaa04e8a20099cc986cb9ccf60c7e876b777ceafb1e03f359ebc6d3dc84245d111a3df584212b5679cb5f9e6717b69b031b"
}

pub fn sample_tx_hash() -> &'static str {
    "0xd51f28edffcaaa76be4a22f6375ad289272c037f3cc072345676e88d92ced8b5"
}

pub fn sample_order_json(uid: &OrderUid) -> Value {
    json!({
        "sellToken": sample_owner().as_str(),
        "buyToken": sample_buy_token().as_str(),
        "receiver": sample_owner().as_str(),
        "sellAmount": "1234567890",
        "buyAmount": "1200000000",
        "validTo": 1_700_000_000,
        "appData": sample_app_data_hash().as_str(),
        "appDataHash": sample_app_data_hash().as_str(),
        "feeAmount": "0",
        "fullBalanceCheck": true,
        "kind": "buy",
        "partiallyFillable": true,
        "sellTokenBalance": "erc20",
        "buyTokenBalance": "erc20",
        "signingScheme": "eip712",
        "signature": "0x1234",
        "owner": sample_owner().as_str(),
        "uid": uid.as_str(),
        "creationDate": "2020-12-03T18:35:18.814523Z",
        "executedSellAmount": "100",
        "executedSellAmountBeforeFees": "99",
        "executedBuyAmount": "90",
        "executedFee": "20",
        "executedFeeAmount": "0",
        "executedFeeToken": sample_owner().as_str(),
        "invalidated": false,
        "status": "open",
        "class": "market",
        "isLiquidityOrder": false,
        "settlementContract": sample_owner().as_str()
    })
}

pub fn sample_ethflow_order_json(uid: &OrderUid) -> Value {
    json!({
        "sellToken": sample_owner().as_str(),
        "buyToken": sample_buy_token().as_str(),
        "receiver": sample_owner().as_str(),
        "sellAmount": "1234567890",
        "buyAmount": "1200000000",
        "validTo": 4_294_967_295u32,
        "appData": sample_app_data_hash().as_str(),
        "feeAmount": "0",
        "kind": "sell",
        "partiallyFillable": false,
        "sellTokenBalance": "erc20",
        "buyTokenBalance": "erc20",
        "signingScheme": "eip712",
        "signature": "0x1234",
        "owner": "0xba3cb449bd2b4adddbc894d8697f5170800eadec",
        "uid": uid.as_str(),
        "creationDate": "2020-12-03T18:35:18.814523Z",
        "executedSellAmountBeforeFees": "100",
        "executedFeeAmount": "0",
        "settlementContract": sample_owner().as_str(),
        "executedSellAmount": "100",
        "executedBuyAmount": "90",
        "executedFee": "10",
        "status": "open",
        "class": "market",
        "onchainUser": sample_owner().as_str(),
        "ethflowData": {
            "refundTxHash": null,
            "userValidTo": 1_700_000_123u32
        }
    })
}

pub fn sample_quote_response_json() -> Value {
    json!({
        "quote": {
            "sellToken": sample_owner().as_str(),
            "buyToken": sample_buy_token().as_str(),
            "receiver": sample_owner().as_str(),
            "sellAmount": "1000",
            "buyAmount": "900",
            "validTo": 1_700_000_000,
            "appData": sample_app_data_hash().as_str(),
            "feeAmount": "10",
            "kind": "sell",
            "partiallyFillable": false,
            "sellTokenBalance": "erc20",
            "buyTokenBalance": "erc20"
        },
        "from": sample_owner().as_str(),
        "expiration": "2026-04-08T10:00:00Z",
        "id": 42,
        "verified": true,
        "protocolFeeBps": "2"
    })
}

pub fn sample_trade_json() -> Value {
    json!({
        "blockNumber": 1,
        "logIndex": 0,
        "orderUid": sample_order_uid().as_str(),
        "owner": sample_owner().as_str(),
        "sellToken": sample_owner().as_str(),
        "buyToken": sample_buy_token().as_str(),
        "sellAmount": "1000",
        "sellAmountBeforeFees": "990",
        "buyAmount": "900",
        "txHash": sample_tx_hash()
    })
}
