use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::types::{Address, Amount, BlockHash, ChainId, HexData, TransactionHash};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedDataDomain {
    pub name: String,
    pub version: String,
    pub chain_id: ChainId,
    pub verifying_contract: Address,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypedDataField {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: String,
}

pub type TypedDataTypes = BTreeMap<String, Vec<TypedDataField>>;

/// Generic EIP-712 envelope shape used by typed helpers and signer payloads.
///
/// The signer-facing alias uses a canonical JSON string for `message` so
/// existing `Signer` implementors can keep the legacy `sign_typed_data`
/// method and still gain additive compatibility through the default
/// `sign_typed_data_payload` implementation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedDataEnvelope<M> {
    pub domain: TypedDataDomain,
    pub primary_type: String,
    pub types: TypedDataTypes,
    pub message: M,
}

pub type TypedDataPayload = TypedDataEnvelope<String>;

impl<M> TypedDataEnvelope<M> {
    pub fn primary_type_fields(&self) -> Option<&[TypedDataField]> {
        self.types.get(&self.primary_type).map(Vec::as_slice)
    }

    pub fn with_message<N>(self, message: N) -> TypedDataEnvelope<N> {
        TypedDataEnvelope {
            domain: self.domain,
            primary_type: self.primary_type,
            types: self.types,
            message,
        }
    }
}

impl TypedDataPayload {
    pub fn message_json(&self) -> &str {
        self.message.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<HexData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Amount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_limit: Option<Amount>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionReceipt {
    pub transaction_hash: TransactionHash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockInfo {
    pub number: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<BlockHash>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractCall {
    pub address: Address,
    pub method: String,
    pub abi_json: String,
    pub args_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractHandle {
    pub address: Address,
    pub abi_json: String,
}

/// Synchronous signing boundary for native or test signers.
///
/// This is an active SDK contract: signing and trading workflows accept it
/// directly, and any implementor also gets `AsyncSigner` through the blanket
/// implementation below.
pub trait Signer {
    type Provider;
    type Error;

    fn connect(&mut self, provider: Self::Provider);
    fn get_address(&self) -> Result<Address, Self::Error>;
    fn sign_message(&self, message: &[u8]) -> Result<String, Self::Error>;
    fn sign_transaction(&self, tx: &TransactionRequest) -> Result<String, Self::Error>;
    fn sign_typed_data_payload(&self, payload: &TypedDataPayload) -> Result<String, Self::Error> {
        self.sign_typed_data(
            &payload.domain,
            payload.primary_type_fields().unwrap_or_default(),
            payload.message_json(),
        )
    }
    fn sign_typed_data(
        &self,
        domain: &TypedDataDomain,
        fields: &[TypedDataField],
        value_json: &str,
    ) -> Result<String, Self::Error>;
    fn send_transaction(&self, tx: &TransactionRequest) -> Result<TransactionReceipt, Self::Error>;
    fn estimate_gas(&self, tx: &TransactionRequest) -> Result<Amount, Self::Error>;
}

/// Asynchronous signing boundary for browser wallets and async runtimes.
///
/// Browser wallet support implements this trait directly. Synchronous signers
/// also implement it through the blanket implementation so public trading flows
/// can keep one async-first internal path.
#[allow(async_fn_in_trait)]
pub trait AsyncSigner {
    type Error;

    async fn get_address(&self) -> Result<Address, Self::Error>;
    async fn sign_message(&self, message: &[u8]) -> Result<String, Self::Error>;
    async fn sign_transaction(&self, tx: &TransactionRequest) -> Result<String, Self::Error>;
    async fn sign_typed_data_payload(
        &self,
        payload: &TypedDataPayload,
    ) -> Result<String, Self::Error> {
        self.sign_typed_data(
            &payload.domain,
            payload.primary_type_fields().unwrap_or_default(),
            payload.message_json(),
        )
        .await
    }
    async fn sign_typed_data(
        &self,
        domain: &TypedDataDomain,
        fields: &[TypedDataField],
        value_json: &str,
    ) -> Result<String, Self::Error>;
    async fn send_transaction(
        &self,
        tx: &TransactionRequest,
    ) -> Result<TransactionReceipt, Self::Error>;
    async fn estimate_gas(&self, tx: &TransactionRequest) -> Result<Amount, Self::Error>;
}

impl<T> AsyncSigner for T
where
    T: Signer,
{
    type Error = T::Error;

    async fn get_address(&self) -> Result<Address, Self::Error> {
        Signer::get_address(self)
    }

    async fn sign_message(&self, message: &[u8]) -> Result<String, Self::Error> {
        Signer::sign_message(self, message)
    }

    async fn sign_transaction(&self, tx: &TransactionRequest) -> Result<String, Self::Error> {
        Signer::sign_transaction(self, tx)
    }

    async fn sign_typed_data(
        &self,
        domain: &TypedDataDomain,
        fields: &[TypedDataField],
        value_json: &str,
    ) -> Result<String, Self::Error> {
        Signer::sign_typed_data(self, domain, fields, value_json)
    }

    async fn send_transaction(
        &self,
        tx: &TransactionRequest,
    ) -> Result<TransactionReceipt, Self::Error> {
        Signer::send_transaction(self, tx)
    }

    async fn estimate_gas(&self, tx: &TransactionRequest) -> Result<Amount, Self::Error> {
        Signer::estimate_gas(self, tx)
    }
}

/// Synchronous provider boundary for native contract reads and transactions.
///
/// Contracts and trading helpers use this trait for provider-backed reads such
/// as storage lookups, allowance checks, and contract calls.
pub trait Provider {
    type Signer;
    type Error;

    fn signer_or_null(&self) -> Option<&Self::Signer>;
    fn get_chain_id(&self) -> Result<ChainId, Self::Error>;
    fn get_code(&self, address: &Address) -> Result<Option<HexData>, Self::Error>;
    fn get_transaction_receipt(
        &self,
        transaction_hash: &TransactionHash,
    ) -> Result<Option<TransactionReceipt>, Self::Error>;
    fn create_signer(&self, signer_hint: &str) -> Result<Self::Signer, Self::Error>;
    fn get_storage_at(&self, address: &Address, slot: &str) -> Result<HexData, Self::Error>;
    fn call(&self, tx: &TransactionRequest) -> Result<HexData, Self::Error>;
    fn read_contract(&self, request: &ContractCall) -> Result<String, Self::Error>;
    fn get_block(&self, block_tag: &str) -> Result<BlockInfo, Self::Error>;
    fn set_signer(&mut self, signer: Self::Signer);
    fn set_provider(&mut self, provider_hint: String);
    fn get_contract(
        &self,
        address: &Address,
        abi_json: &str,
    ) -> Result<ContractHandle, Self::Error>;
}

/// Asynchronous provider boundary for browser wallets and async runtimes.
///
/// `cow-sdk-browser-wallet` implements this directly. Synchronous providers get
/// async compatibility through the blanket implementation when their signer can
/// satisfy `AsyncSigner`.
#[allow(async_fn_in_trait)]
pub trait AsyncProvider {
    type Signer: AsyncSigner<Error = Self::Error>;
    type Error;

    async fn get_chain_id(&self) -> Result<ChainId, Self::Error>;
    async fn get_code(&self, address: &Address) -> Result<Option<HexData>, Self::Error>;
    async fn get_transaction_receipt(
        &self,
        transaction_hash: &TransactionHash,
    ) -> Result<Option<TransactionReceipt>, Self::Error>;
    async fn create_signer(&self, signer_hint: &str) -> Result<Self::Signer, Self::Error>;
    async fn get_storage_at(&self, address: &Address, slot: &str) -> Result<HexData, Self::Error>;
    async fn call(&self, tx: &TransactionRequest) -> Result<HexData, Self::Error>;
    async fn read_contract(&self, request: &ContractCall) -> Result<String, Self::Error>;
    async fn get_block(&self, block_tag: &str) -> Result<BlockInfo, Self::Error>;
    async fn get_contract(
        &self,
        address: &Address,
        abi_json: &str,
    ) -> Result<ContractHandle, Self::Error>;
}

impl<T> AsyncProvider for T
where
    T: Provider,
    T::Signer: AsyncSigner<Error = T::Error>,
{
    type Signer = T::Signer;
    type Error = T::Error;

    async fn get_chain_id(&self) -> Result<ChainId, Self::Error> {
        Provider::get_chain_id(self)
    }

    async fn get_code(&self, address: &Address) -> Result<Option<HexData>, Self::Error> {
        Provider::get_code(self, address)
    }

    async fn get_transaction_receipt(
        &self,
        transaction_hash: &TransactionHash,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        Provider::get_transaction_receipt(self, transaction_hash)
    }

    async fn create_signer(&self, signer_hint: &str) -> Result<Self::Signer, Self::Error> {
        Provider::create_signer(self, signer_hint)
    }

    async fn get_storage_at(&self, address: &Address, slot: &str) -> Result<HexData, Self::Error> {
        Provider::get_storage_at(self, address, slot)
    }

    async fn call(&self, tx: &TransactionRequest) -> Result<HexData, Self::Error> {
        Provider::call(self, tx)
    }

    async fn read_contract(&self, request: &ContractCall) -> Result<String, Self::Error> {
        Provider::read_contract(self, request)
    }

    async fn get_block(&self, block_tag: &str) -> Result<BlockInfo, Self::Error> {
        Provider::get_block(self, block_tag)
    }

    async fn get_contract(
        &self,
        address: &Address,
        abi_json: &str,
    ) -> Result<ContractHandle, Self::Error> {
        Provider::get_contract(self, address, abi_json)
    }
}

/// Extension seam for downstream HTTP adapters.
///
/// The current orderbook client owns its typed request policy directly instead
/// of routing through this trait. Keep this as an adapter contract for consumers
/// or future internal transport unification, not as a claim that orderbook uses
/// a generic core HTTP transport today.
pub trait HttpTransport {
    type Error;

    fn get(&self, path: &str) -> Result<String, Self::Error>;
    fn post(&self, path: &str, body: &str) -> Result<String, Self::Error>;
    fn delete(&self, path: &str, body: &str) -> Result<String, Self::Error>;
}

/// Extension seam for downstream GraphQL adapters.
///
/// The current subgraph client owns its typed query execution directly. Keep
/// this as an adapter contract for consumers or future transport unification.
pub trait GraphTransport {
    type Error;

    fn execute(
        &self,
        endpoint: &str,
        query: &str,
        variables_json: Option<&str>,
    ) -> Result<String, Self::Error>;
}

/// Extension seam for downstream JSON pinning adapters.
///
/// App-data pinning currently uses its own fetch and pinning contracts because
/// it needs app-data-specific request and credential semantics.
pub trait PinningTransport {
    type Error;

    fn pin_json(&self, payload: &str) -> Result<String, Self::Error>;
}
