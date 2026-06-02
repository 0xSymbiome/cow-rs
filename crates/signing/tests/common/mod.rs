#![allow(
    dead_code,
    reason = "shared test-helper module exposes sample builders that not every integration test binary exercises; a given test may use only a subset without leaving the rest permanently unused"
)]

use cow_sdk_core::OrderData;

pub fn sample_order() -> OrderData {
    serde_json::from_value(serde_json::json!({
        "sellToken": "0xd057b63f5e69cf1b929b356b579cba08d7688048",
        "buyToken": "0x7b878668cd1a3adf89764d3a331e0a7bb832192d",
        "receiver": "0xa6ddbd0de6b310819b49f680f65871bee85f517e",
        "sellAmount": "500000000000000",
        "buyAmount": "23000020000",
        "validTo": 5_000_222,
        "appData": "0x0000000000000000000000000000000000000000000000000000000000000000",
        "feeAmount": "2300000",
        "kind": "sell",
        "partiallyFillable": true,
        "sellTokenBalance": "erc20",
        "buyTokenBalance": "erc20"
    }))
    .unwrap()
}

pub fn sample_order_uid() -> cow_sdk_core::OrderUid {
    cow_sdk_core::OrderUid::new(
        "0xdaaa7dddec9ad04cc101a121e3eed017eab4d3927c045d407d5ad6700eea2bf7fb3c7eb936caa12b5a884d612393969a557d430764060343",
    )
    .unwrap()
}

pub fn sample_signature(byte: &str) -> String {
    format!("0x{}1b", byte.repeat(64))
}
