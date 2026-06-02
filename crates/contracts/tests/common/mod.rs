#![allow(
    dead_code,
    reason = "shared test-helper module aggregates fixtures, constants, and adapters that not every integration test binary exercises; an integration test may use only a subset of the shared helpers without leaving the others permanently unused"
)]

use std::{cell::RefCell, collections::BTreeMap, fmt, rc::Rc};

use cow_sdk_core::{
    Address, BlockInfo, ContractCall, ContractHandle, Hash32, HexData, Provider,
    TransactionReceipt, TransactionRequest,
};
use serde_json::Value;

pub fn fixture_case(id: &str) -> Value {
    cow_sdk_test_utils::fixtures::case("contracts", id)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MockProviderError(pub String);

impl fmt::Display for MockProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Default)]
pub struct DummySigner;

#[derive(Debug, Clone)]
pub struct MockProvider {
    pub storage: Rc<RefCell<BTreeMap<(String, String), String>>>,
    pub calls: Rc<RefCell<Vec<ContractCall>>>,
    pub response: Rc<RefCell<String>>,
    pub response_error: Rc<RefCell<Option<String>>>,
    pub code: Rc<RefCell<Option<HexData>>>,
    pub code_error: Rc<RefCell<Option<String>>>,
    pub chain_id: u64,
}

impl Default for MockProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl MockProvider {
    pub fn new() -> Self {
        Self {
            storage: Rc::new(RefCell::new(BTreeMap::new())),
            calls: Rc::new(RefCell::new(Vec::new())),
            response: Rc::new(RefCell::new("null".to_owned())),
            response_error: Rc::new(RefCell::new(None)),
            code: Rc::new(RefCell::new(None)),
            code_error: Rc::new(RefCell::new(None)),
            chain_id: 1,
        }
    }

    pub fn set_storage(&self, address: &Address, slot: &str, value: &str) {
        self.storage.borrow_mut().insert(
            (address.to_hex_string(), slot.to_ascii_lowercase()),
            value.to_owned(),
        );
    }

    pub fn set_response(&self, value: &str) {
        let mut response = self.response.borrow_mut();
        value.clone_into(&mut response);
    }

    pub fn set_response_error(&self, value: Option<&str>) {
        *self.response_error.borrow_mut() = value.map(str::to_owned);
    }

    pub fn set_code(&self, value: Option<&str>) {
        *self.code.borrow_mut() = value.map(|value| HexData::new(value).unwrap());
    }

    pub fn set_code_error(&self, value: Option<&str>) {
        *self.code_error.borrow_mut() = value.map(str::to_owned);
    }
}

impl Provider for MockProvider {
    type Error = MockProviderError;

    async fn get_chain_id(&self) -> Result<u64, Self::Error> {
        Ok(self.chain_id)
    }

    async fn get_code(&self, _address: &Address) -> Result<Option<HexData>, Self::Error> {
        if let Some(message) = self.code_error.borrow().clone() {
            return Err(MockProviderError(message));
        }
        Ok(self.code.borrow().clone())
    }

    async fn get_transaction_receipt(
        &self,
        _transaction_hash: &Hash32,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        Ok(None)
    }

    async fn get_storage_at(&self, address: &Address, slot: &str) -> Result<HexData, Self::Error> {
        let value = self
            .storage
            .borrow()
            .get(&(address.to_hex_string(), slot.to_ascii_lowercase()))
            .cloned()
            .ok_or_else(|| MockProviderError(format!("missing storage for {address} at {slot}")))?;
        HexData::new(value).map_err(|error| MockProviderError(error.to_string()))
    }

    async fn call(&self, _tx: &TransactionRequest) -> Result<HexData, Self::Error> {
        Err(MockProviderError("call not implemented".to_owned()))
    }

    async fn read_contract(&self, request: &ContractCall) -> Result<String, Self::Error> {
        self.calls.borrow_mut().push(request.clone());
        if let Some(message) = self.response_error.borrow().clone() {
            return Err(MockProviderError(message));
        }
        Ok(self.response.borrow().clone())
    }

    async fn get_block(&self, _block_tag: &str) -> Result<BlockInfo, Self::Error> {
        Ok(BlockInfo::new(0, None))
    }

    async fn get_contract(
        &self,
        address: &Address,
        abi_json: &str,
    ) -> Result<ContractHandle, Self::Error> {
        Ok(ContractHandle::new(*address, abi_json.to_owned()))
    }
}
