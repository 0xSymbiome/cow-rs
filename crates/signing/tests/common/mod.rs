#![allow(
    dead_code,
    reason = "shared test-helper module aggregates fixtures, constants, and adapters that not every integration test binary exercises; an integration test may use only a subset of the shared helpers without leaving the others permanently unused"
)]

use std::{cell::RefCell, fmt, rc::Rc};

use cow_sdk_core::{
    Address, Amount, Hash32, OrderData, Signer, TransactionBroadcast, TransactionRequest,
    TypedDataDomain, TypedDataField,
};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedDataCall {
    pub domain: TypedDataDomain,
    pub fields: Vec<TypedDataField>,
    pub value_json: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecordedCalls {
    pub messages: Vec<Vec<u8>>,
    pub typed_data: Vec<TypedDataCall>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MockSignerError(pub String);

impl fmt::Display for MockSignerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl cow_sdk_core::SignerError for MockSignerError {}

#[derive(Clone)]
pub struct MockSigner {
    pub address: Address,
    pub typed_data_signature: String,
    pub message_signature: String,
    pub calls: Rc<RefCell<RecordedCalls>>,
}

impl MockSigner {
    pub fn new() -> Self {
        Self {
            address: Address::new("0x4444444444444444444444444444444444444444").unwrap(),
            typed_data_signature: sample_signature("aa"),
            message_signature: sample_signature("bb"),
            calls: Rc::new(RefCell::new(RecordedCalls::default())),
        }
    }
}

impl Default for MockSigner {
    fn default() -> Self {
        Self::new()
    }
}

impl Signer for MockSigner {
    type Error = MockSignerError;

    async fn get_address(&self) -> Result<Address, Self::Error> {
        Ok(self.address)
    }

    async fn sign_message(&self, message: &[u8]) -> Result<String, Self::Error> {
        self.calls.borrow_mut().messages.push(message.to_vec());
        Ok(self.message_signature.clone())
    }

    async fn sign_transaction(&self, _tx: &TransactionRequest) -> Result<String, Self::Error> {
        Ok("0xsigned-transaction".to_owned())
    }

    async fn sign_typed_data(
        &self,
        domain: &TypedDataDomain,
        fields: &[TypedDataField],
        value_json: &str,
    ) -> Result<String, Self::Error> {
        self.calls.borrow_mut().typed_data.push(TypedDataCall {
            domain: domain.clone(),
            fields: fields.to_vec(),
            value_json: value_json.to_owned(),
        });
        Ok(self.typed_data_signature.clone())
    }

    async fn send_transaction(
        &self,
        _tx: &TransactionRequest,
    ) -> Result<TransactionBroadcast, Self::Error> {
        Ok(TransactionBroadcast::new(
            Hash32::new(format!("0x{}", "fa".repeat(32))).unwrap(),
        ))
    }

    async fn estimate_gas(&self, _tx: &TransactionRequest) -> Result<Amount, Self::Error> {
        Ok(Amount::from(21_000u32))
    }
}

pub fn signing_fixture() -> Value {
    serde_json::from_str(include_str!("../../../../parity/fixtures/signing.json"))
        .expect("signing fixture must remain valid JSON")
}

pub fn fixture_case(id: &str) -> Value {
    signing_fixture()["cases"]
        .as_array()
        .expect("fixture cases must be an array")
        .iter()
        .find(|case| case["id"] == id)
        .cloned()
        .unwrap_or_else(|| panic!("missing fixture case {id}"))
}

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
