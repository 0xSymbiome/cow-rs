#![allow(dead_code)]

use serde_json::{Value, json};

pub const APP_DATA_HEX: &str = "0x337aa6e6c2a7a0d1eb79a35ebd88b08fc963d5f7a3fc953b7ffb2b7f5898a1df";
pub const CID: &str = "f01551b20337aa6e6c2a7a0d1eb79a35ebd88b08fc963d5f7a3fc953b7ffb2b7f5898a1df";

pub const APP_DATA_HEX_2: &str =
    "0x8af4e8c9973577b08ac21d17d331aade86c11ebcc5124744d621ca8365ec9424";
pub const CID_2: &str = "f01551b208af4e8c9973577b08ac21d17d331aade86c11ebcc5124744d621ca8365ec9424";

pub const APP_DATA_HEX_LEGACY: &str =
    "0x447320af985c5e834321dc495545f764ad20d8397eeed2f4a2dcbee44a56b725";
pub const CID_LEGACY: &str = "QmSwrFbdFcryazEr361YmSwtGcN4uo4U5DKpzA4KbGxw4Q";

pub const PINATA_IPFS_HASH: &str = "QmU4j5Y6JM9DqQ6yxB6nMHq4GChWg1zPehs1U7nGPHABRu";
pub const PINATA_APP_DATA_HEX: &str =
    "0x5511c4eac66ab272d9a6ab90e07977d00ff7375fc4dc1038a3c05b2c16ca0b74";

pub const APP_DATA_STRING: &str =
    "{\"appCode\":\"CoW Swap\",\"metadata\":{},\"version\":\"0.7.0\"}";
pub const APP_DATA_STRING_2: &str = "{\"appCode\":\"CoW Swap\",\"environment\":\"production\",\"metadata\":{\"quote\":{\"slippageBips\":\"50\",\"version\":\"0.2.0\"},\"orderClass\":{\"orderClass\":\"market\",\"version\":\"0.1.0\"}},\"version\":\"0.6.0\"}";
pub const APP_DATA_STRING_LEGACY: &str =
    "{\"version\":\"0.7.0\",\"appCode\":\"CowSwap\",\"metadata\":{}}";

pub fn parity_fixture() -> Value {
    serde_json::from_str(include_str!("../../../../parity/fixtures/app-data.json")).unwrap()
}

pub fn app_data_doc() -> Value {
    json!({
        "version": "0.7.0",
        "appCode": "CoW Swap",
        "metadata": {}
    })
}

pub fn app_data_doc_custom() -> Value {
    json!({
        "version": "1.14.0",
        "appCode": "CoW Swap",
        "environment": "test",
        "metadata": {
            "referrer": {
                "code": "COWREF1"
            },
            "quote": {
                "slippageBips": 1
            }
        }
    })
}

pub fn invalid_referrer_doc() -> Value {
    json!({
        "version": "0.4.0",
        "metadata": {
            "referrer": {
                "version": "312313",
                "address": "0xssss"
            }
        }
    })
}
