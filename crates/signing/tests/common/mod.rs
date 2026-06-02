#![allow(
    dead_code,
    reason = "shared test-helper module exposes sample builders that not every integration test binary exercises; a given test may use only a subset without leaving the rest permanently unused"
)]

use cow_sdk_core::OrderData;

pub fn sample_order() -> OrderData {
    cow_sdk_test_utils::builders::OrderBuilder::default().build()
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
