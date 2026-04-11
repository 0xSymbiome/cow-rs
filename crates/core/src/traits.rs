use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::types::{Address, Amount, BlockHash, ChainId, HexData, TransactionHash};

/// Typed-data domain metadata used for EIP-712 signing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedDataDomain {
    /// Human-readable protocol name.
    pub name: String,
    /// Domain version string.
    pub version: String,
    /// Numeric chain id for the typed-data domain.
    pub chain_id: ChainId,
    /// Contract address used as the domain verifier.
    pub verifying_contract: Address,
}

/// A single EIP-712 typed-data field descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypedDataField {
    /// Field name as it appears in the typed-data schema.
    pub name: String,
    #[serde(rename = "type")]
    /// Solidity type name for the field.
    pub kind: String,
}

/// EIP-712 type map keyed by type name.
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
    /// Domain metadata used to compute the typed-data digest.
    pub domain: TypedDataDomain,
    /// Primary type name for the payload.
    pub primary_type: String,
    /// Full type map including the primary type and `EIP712Domain`.
    pub types: TypedDataTypes,
    /// Payload message body.
    pub message: M,
}

/// Typed-data envelope that stores the message body as canonical JSON.
pub type TypedDataPayload = TypedDataEnvelope<String>;

impl<M> TypedDataEnvelope<M> {
    /// Returns the field list for the current primary type, if present.
    #[must_use]
    pub fn primary_type_fields(&self) -> Option<&[TypedDataField]> {
        self.types.get(&self.primary_type).map(Vec::as_slice)
    }

    /// Replaces the message body while preserving domain and type metadata.
    #[must_use]
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
    /// Returns the canonical JSON message body.
    #[must_use]
    pub fn message_json(&self) -> &str {
        self.message.as_str()
    }
}

/// Transaction request shape used across signer and provider traits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Destination address for the transaction.
    pub to: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Hex-encoded calldata payload.
    pub data: Option<HexData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Native token value to transfer.
    pub value: Option<Amount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Optional gas limit override.
    pub gas_limit: Option<Amount>,
}

/// Minimal transaction receipt contract used by the SDK surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionReceipt {
    /// Transaction hash for the submitted transaction.
    pub transaction_hash: TransactionHash,
}

/// Minimal block information contract used by provider traits.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockInfo {
    /// Block number.
    pub number: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Optional block hash when the backend returns it.
    pub hash: Option<BlockHash>,
}

/// Typed contract-read request used by runtime-neutral providers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractCall {
    /// Target contract address.
    pub address: Address,
    /// ABI method name to invoke.
    pub method: String,
    /// JSON ABI fragment describing the contract or function.
    pub abi_json: String,
    /// JSON-encoded function arguments.
    pub args_json: String,
}

/// Contract handle returned by providers that support typed contract creation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractHandle {
    /// Target contract address.
    pub address: Address,
    /// JSON ABI for the contract handle.
    pub abi_json: String,
}

/// Synchronous signing boundary for native or test signers.
///
/// This is an active SDK contract: signing and trading workflows accept it
/// directly, and any implementor also gets `AsyncSigner` through the blanket
/// implementation below.
pub trait Signer {
    /// Provider type that can be attached to this signer.
    type Provider;
    /// Error type returned by signer operations.
    type Error;

    /// Attaches a provider or provider-like runtime to the signer.
    fn connect(&mut self, provider: Self::Provider);
    /// Returns the signer address.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when address resolution fails.
    fn get_address(&self) -> Result<Address, Self::Error>;
    /// Signs arbitrary bytes according to the backend's message-signing rules.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when signing fails.
    fn sign_message(&self, message: &[u8]) -> Result<String, Self::Error>;
    /// Signs a transaction payload.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when signing fails.
    fn sign_transaction(&self, tx: &TransactionRequest) -> Result<String, Self::Error>;
    /// Signs an explicit typed-data payload.
    ///
    /// # Errors
    ///
    /// Returns any error from [`Signer::sign_typed_data`].
    fn sign_typed_data_payload(&self, payload: &TypedDataPayload) -> Result<String, Self::Error> {
        self.sign_typed_data(
            &payload.domain,
            payload.primary_type_fields().unwrap_or_default(),
            payload.message_json(),
        )
    }
    /// Signs typed-data components using the compatibility field-based contract.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when signing fails.
    fn sign_typed_data(
        &self,
        domain: &TypedDataDomain,
        fields: &[TypedDataField],
        value_json: &str,
    ) -> Result<String, Self::Error>;
    /// Sends a transaction and returns a minimal receipt contract.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when submission fails.
    fn send_transaction(&self, tx: &TransactionRequest) -> Result<TransactionReceipt, Self::Error>;
    /// Estimates gas for a transaction request.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when estimation fails.
    fn estimate_gas(&self, tx: &TransactionRequest) -> Result<Amount, Self::Error>;
}

/// Asynchronous signing boundary for browser wallets and async runtimes.
///
/// Browser wallet support implements this trait directly. Synchronous signers
/// also implement it through the blanket implementation so public trading flows
/// can keep one async-first internal path.
#[allow(async_fn_in_trait)]
pub trait AsyncSigner {
    /// Error type returned by signer operations.
    type Error;

    /// Returns the signer address.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when address resolution fails.
    async fn get_address(&self) -> Result<Address, Self::Error>;
    /// Signs arbitrary bytes according to the backend's message-signing rules.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when signing fails.
    async fn sign_message(&self, message: &[u8]) -> Result<String, Self::Error>;
    /// Signs a transaction payload.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when signing fails.
    async fn sign_transaction(&self, tx: &TransactionRequest) -> Result<String, Self::Error>;
    /// Signs an explicit typed-data payload.
    ///
    /// # Errors
    ///
    /// Returns any error from [`AsyncSigner::sign_typed_data`].
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
    /// Signs typed-data components using the compatibility field-based contract.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when signing fails.
    async fn sign_typed_data(
        &self,
        domain: &TypedDataDomain,
        fields: &[TypedDataField],
        value_json: &str,
    ) -> Result<String, Self::Error>;
    /// Sends a transaction and returns a minimal receipt contract.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when submission fails.
    async fn send_transaction(
        &self,
        tx: &TransactionRequest,
    ) -> Result<TransactionReceipt, Self::Error>;
    /// Estimates gas for a transaction request.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when estimation fails.
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
    /// Signer type exposed by this provider.
    type Signer;
    /// Error type returned by provider operations.
    type Error;

    /// Returns the currently attached signer, if one exists.
    fn signer_or_null(&self) -> Option<&Self::Signer>;
    /// Returns the current chain id.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined provider error when the chain id cannot be loaded.
    fn get_chain_id(&self) -> Result<ChainId, Self::Error>;
    /// Returns deployed bytecode for an address, if present.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined provider error when the backend lookup fails.
    fn get_code(&self, address: &Address) -> Result<Option<HexData>, Self::Error>;
    /// Returns the receipt for a transaction hash, if known.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined provider error when the backend lookup fails.
    fn get_transaction_receipt(
        &self,
        transaction_hash: &TransactionHash,
    ) -> Result<Option<TransactionReceipt>, Self::Error>;
    /// Creates a signer from an implementation-defined hint.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined provider error when signer creation fails.
    fn create_signer(&self, signer_hint: &str) -> Result<Self::Signer, Self::Error>;
    /// Reads a storage slot from a contract address.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined provider error when the storage lookup fails.
    fn get_storage_at(&self, address: &Address, slot: &str) -> Result<HexData, Self::Error>;
    /// Executes a read-only call.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined provider error when the call fails.
    fn call(&self, tx: &TransactionRequest) -> Result<HexData, Self::Error>;
    /// Executes a typed contract read request.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined provider error when the contract read fails.
    fn read_contract(&self, request: &ContractCall) -> Result<String, Self::Error>;
    /// Returns block information for a backend-specific block tag.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined provider error when the block lookup fails.
    fn get_block(&self, block_tag: &str) -> Result<BlockInfo, Self::Error>;
    /// Replaces the attached signer.
    fn set_signer(&mut self, signer: Self::Signer);
    /// Replaces the provider runtime using an implementation-defined hint.
    fn set_provider(&mut self, provider_hint: String);
    /// Returns a typed contract handle for an address and ABI.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined provider error when contract creation fails.
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
    /// Signer type exposed by this provider.
    type Signer: AsyncSigner<Error = Self::Error>;
    /// Error type returned by provider operations.
    type Error;

    /// Returns the current chain id.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined provider error when the chain id cannot be loaded.
    async fn get_chain_id(&self) -> Result<ChainId, Self::Error>;
    /// Returns deployed bytecode for an address, if present.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined provider error when the backend lookup fails.
    async fn get_code(&self, address: &Address) -> Result<Option<HexData>, Self::Error>;
    /// Returns the receipt for a transaction hash, if known.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined provider error when the backend lookup fails.
    async fn get_transaction_receipt(
        &self,
        transaction_hash: &TransactionHash,
    ) -> Result<Option<TransactionReceipt>, Self::Error>;
    /// Creates a signer from an implementation-defined hint.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined provider error when signer creation fails.
    async fn create_signer(&self, signer_hint: &str) -> Result<Self::Signer, Self::Error>;
    /// Reads a storage slot from a contract address.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined provider error when the storage lookup fails.
    async fn get_storage_at(&self, address: &Address, slot: &str) -> Result<HexData, Self::Error>;
    /// Executes a read-only call.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined provider error when the call fails.
    async fn call(&self, tx: &TransactionRequest) -> Result<HexData, Self::Error>;
    /// Executes a typed contract read request.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined provider error when the contract read fails.
    async fn read_contract(&self, request: &ContractCall) -> Result<String, Self::Error>;
    /// Returns block information for a backend-specific block tag.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined provider error when the block lookup fails.
    async fn get_block(&self, block_tag: &str) -> Result<BlockInfo, Self::Error>;
    /// Returns a typed contract handle for an address and ABI.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined provider error when contract creation fails.
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
    /// Error type returned by transport operations.
    type Error;

    /// Performs an HTTP `GET`.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined transport error when the request fails.
    fn get(&self, path: &str) -> Result<String, Self::Error>;
    /// Performs an HTTP `POST`.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined transport error when the request fails.
    fn post(&self, path: &str, body: &str) -> Result<String, Self::Error>;
    /// Performs an HTTP `DELETE`.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined transport error when the request fails.
    fn delete(&self, path: &str, body: &str) -> Result<String, Self::Error>;
}

/// Extension seam for downstream GraphQL adapters.
///
/// The current subgraph client owns its typed query execution directly. Keep
/// this as an adapter contract for consumers or future transport unification.
pub trait GraphTransport {
    /// Error type returned by transport operations.
    type Error;

    /// Executes a GraphQL request against the supplied endpoint.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined transport error when the request fails.
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
    /// Error type returned by transport operations.
    type Error;

    /// Pins a JSON payload and returns an implementation-defined identifier.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined transport error when pinning fails.
    fn pin_json(&self, payload: &str) -> Result<String, Self::Error>;
}
