#![allow(dead_code)]

use std::{cell::RefCell, collections::BTreeMap, fmt, rc::Rc};

use cow_sdk_core::{
    Address, BlockInfo, ContractCall, ContractHandle, Hash32, HexData, Provider,
    TransactionReceipt, TransactionRequest,
};
use serde_json::Value;

pub fn contracts_fixture() -> Value {
    serde_json::from_str(include_str!("../../../../parity/fixtures/contracts.json"))
        .expect("contracts fixture must remain valid JSON")
}

pub fn fixture_case(id: &str) -> Value {
    contracts_fixture()["cases"]
        .as_array()
        .expect("fixture cases must be an array")
        .iter()
        .find(|case| case["id"] == id)
        .cloned()
        .unwrap_or_else(|| panic!("missing fixture case {id}"))
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
            chain_id: 1,
        }
    }

    pub fn set_storage(&self, address: &Address, slot: &str, value: &str) {
        self.storage.borrow_mut().insert(
            (address.normalized_key(), slot.to_ascii_lowercase()),
            value.to_owned(),
        );
    }

    pub fn set_response(&self, value: &str) {
        *self.response.borrow_mut() = value.to_owned();
    }
}

impl Provider for MockProvider {
    type Signer = DummySigner;
    type Error = MockProviderError;

    fn signer_or_null(&self) -> Option<&Self::Signer> {
        None
    }

    fn get_chain_id(&self) -> Result<u64, Self::Error> {
        Ok(self.chain_id)
    }

    fn get_code(&self, _address: &Address) -> Result<Option<HexData>, Self::Error> {
        Ok(None)
    }

    fn get_transaction_receipt(
        &self,
        _transaction_hash: &Hash32,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        Ok(None)
    }

    fn create_signer(&self, _signer_hint: &str) -> Result<Self::Signer, Self::Error> {
        Ok(DummySigner)
    }

    fn get_storage_at(&self, address: &Address, slot: &str) -> Result<HexData, Self::Error> {
        let value = self
            .storage
            .borrow()
            .get(&(address.normalized_key(), slot.to_ascii_lowercase()))
            .cloned()
            .ok_or_else(|| {
                MockProviderError(format!("missing storage for {} at {}", address, slot))
            })?;
        HexData::new(value).map_err(|error| MockProviderError(error.to_string()))
    }

    fn call(&self, _tx: &TransactionRequest) -> Result<HexData, Self::Error> {
        Err(MockProviderError("call not implemented".to_owned()))
    }

    fn read_contract(&self, request: &ContractCall) -> Result<String, Self::Error> {
        self.calls.borrow_mut().push(request.clone());
        Ok(self.response.borrow().clone())
    }

    fn get_block(&self, _block_tag: &str) -> Result<BlockInfo, Self::Error> {
        Ok(BlockInfo {
            number: 0,
            hash: None,
        })
    }

    fn set_signer(&mut self, _signer: Self::Signer) {}

    fn set_provider(&mut self, _provider_hint: String) {}

    fn get_contract(
        &self,
        address: &Address,
        abi_json: &str,
    ) -> Result<ContractHandle, Self::Error> {
        Ok(ContractHandle {
            address: address.clone(),
            abi_json: abi_json.to_owned(),
        })
    }
}
