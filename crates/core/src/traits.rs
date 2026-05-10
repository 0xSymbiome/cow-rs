use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::types::{Address, Amount, BlockHash, ChainId, HexData, TransactionHash};

/// Typed-data domain metadata used for EIP-712 signing.
#[non_exhaustive]
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

impl TypedDataDomain {
    /// Creates typed-data domain metadata for EIP-712 signing.
    #[inline]
    #[must_use]
    pub const fn new(
        name: String,
        version: String,
        chain_id: ChainId,
        verifying_contract: Address,
    ) -> Self {
        Self {
            name,
            version,
            chain_id,
            verifying_contract,
        }
    }
}

/// A single EIP-712 typed-data field descriptor.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypedDataField {
    /// Field name as it appears in the typed-data schema.
    pub name: String,
    #[serde(rename = "type")]
    /// Solidity type name for the field.
    pub kind: String,
}

impl TypedDataField {
    /// Creates an EIP-712 typed-data field descriptor.
    #[inline]
    #[must_use]
    pub const fn new(name: String, kind: String) -> Self {
        Self { name, kind }
    }
}

/// EIP-712 type map keyed by type name.
pub type TypedDataTypes = BTreeMap<String, Vec<TypedDataField>>;

/// Generic EIP-712 envelope shape used by typed helpers and signer payloads.
///
/// The signer-facing alias uses a canonical JSON string for `message` so
/// existing `Signer` implementors can keep the legacy `sign_typed_data`
/// method and still gain additive compatibility through the default
/// `sign_typed_data_payload` implementation.
#[non_exhaustive]
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
    /// Creates an EIP-712 typed-data envelope.
    #[inline]
    #[must_use]
    pub const fn new(
        domain: TypedDataDomain,
        primary_type: String,
        types: TypedDataTypes,
        message: M,
    ) -> Self {
        Self {
            domain,
            primary_type,
            types,
            message,
        }
    }

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
    pub const fn message_json(&self) -> &str {
        self.message.as_str()
    }
}

/// Transaction request shape used across signer and provider traits.
#[non_exhaustive]
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

impl TransactionRequest {
    /// Creates a transaction request shape.
    #[inline]
    #[must_use]
    pub const fn new(
        to: Option<Address>,
        data: Option<HexData>,
        value: Option<Amount>,
        gas_limit: Option<Amount>,
    ) -> Self {
        Self {
            to,
            data,
            value,
            gas_limit,
        }
    }
}

/// Broadcast acknowledgement returned by signer-backed transaction submission.
///
/// This value confirms that a backend accepted or observed a transaction hash.
/// It does not imply that the transaction has been mined, succeeded, or even
/// become visible to a read provider. Use [`Provider::get_transaction_receipt`],
/// [`AsyncProvider::get_transaction_receipt`], or a higher-level
/// `cow-sdk-trading` wait helper when lifecycle state is required.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionBroadcast {
    /// Transaction hash for the submitted transaction.
    pub transaction_hash: TransactionHash,
}

impl TransactionBroadcast {
    /// Creates a transaction broadcast acknowledgement from its hash.
    #[inline]
    #[must_use]
    pub const fn new(transaction_hash: TransactionHash) -> Self {
        Self { transaction_hash }
    }
}

/// Terminal transaction execution state exposed by receipt-capable providers.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TransactionStatus {
    /// The transaction was mined successfully.
    Success,
    /// The transaction was mined and reverted.
    Reverted,
}

/// Transaction receipt contract returned by provider receipt lookups.
///
/// [`TransactionReceipt::new`] preserves hash-only adapters by leaving every
/// rich lifecycle field empty. Receipt-capable providers can populate the
/// optional fields with [`TransactionReceipt::from_parts`] or the builder
/// methods as adapter support matures.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionReceipt {
    /// Transaction hash for the observed transaction.
    pub transaction_hash: TransactionHash,
    /// Optional terminal execution status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<TransactionStatus>,
    /// Optional block number that included the transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_number: Option<u64>,
    /// Optional block hash that included the transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_hash: Option<BlockHash>,
    /// Optional gas used by the transaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_used: Option<Amount>,
    /// Optional sender address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Address>,
    /// Optional destination address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<Address>,
}

impl TransactionReceipt {
    /// Creates a hash-only transaction receipt.
    #[inline]
    #[must_use]
    pub const fn new(transaction_hash: TransactionHash) -> Self {
        Self {
            transaction_hash,
            status: None,
            block_number: None,
            block_hash: None,
            gas_used: None,
            from: None,
            to: None,
        }
    }

    /// Creates a transaction receipt from every supported receipt field.
    #[inline]
    #[must_use]
    pub const fn from_parts(
        transaction_hash: TransactionHash,
        status: Option<TransactionStatus>,
        block_number: Option<u64>,
        block_hash: Option<BlockHash>,
        gas_used: Option<Amount>,
        from: Option<Address>,
        to: Option<Address>,
    ) -> Self {
        Self {
            transaction_hash,
            status,
            block_number,
            block_hash,
            gas_used,
            from,
            to,
        }
    }

    /// Sets the terminal execution status.
    #[must_use]
    pub const fn with_status(mut self, status: TransactionStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Sets the block number that included the transaction.
    #[must_use]
    pub const fn with_block_number(mut self, block_number: u64) -> Self {
        self.block_number = Some(block_number);
        self
    }

    /// Sets the block hash that included the transaction.
    #[must_use]
    pub fn with_block_hash(mut self, block_hash: BlockHash) -> Self {
        self.block_hash = Some(block_hash);
        self
    }

    /// Sets the gas used by the transaction.
    #[must_use]
    pub fn with_gas_used(mut self, gas_used: Amount) -> Self {
        self.gas_used = Some(gas_used);
        self
    }

    /// Sets the sender address.
    #[must_use]
    pub fn with_from(mut self, from: Address) -> Self {
        self.from = Some(from);
        self
    }

    /// Sets the destination address.
    #[must_use]
    pub fn with_to(mut self, to: Address) -> Self {
        self.to = Some(to);
        self
    }
}

/// Minimal block information contract used by provider traits.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockInfo {
    /// Block number.
    pub number: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Optional block hash when the backend returns it.
    pub hash: Option<BlockHash>,
}

impl BlockInfo {
    /// Creates minimal block information.
    #[inline]
    #[must_use]
    pub const fn new(number: u64, hash: Option<BlockHash>) -> Self {
        Self { number, hash }
    }
}

/// Typed contract-read request used by runtime-neutral providers.
#[non_exhaustive]
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

impl ContractCall {
    /// Creates a typed contract-read request.
    #[inline]
    #[must_use]
    pub const fn new(
        address: Address,
        method: String,
        abi_json: String,
        args_json: String,
    ) -> Self {
        Self {
            address,
            method,
            abi_json,
            args_json,
        }
    }
}

/// Contract handle returned by providers that support typed contract creation.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractHandle {
    /// Target contract address.
    pub address: Address,
    /// JSON ABI for the contract handle.
    pub abi_json: String,
}

impl ContractHandle {
    /// Creates a typed contract handle.
    #[inline]
    #[must_use]
    pub const fn new(address: Address, abi_json: String) -> Self {
        Self { address, abi_json }
    }
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
    /// Sends a transaction and returns the broadcast transaction hash.
    ///
    /// This confirms only that the signer backend returned a transaction hash.
    /// Use [`Provider::get_transaction_receipt`] or a higher-level
    /// `cow-sdk-trading` wait helper to observe mining status and receipt
    /// fields.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when submission fails.
    fn send_transaction(
        &self,
        tx: &TransactionRequest,
    ) -> Result<TransactionBroadcast, Self::Error>;
    /// Estimates gas for a transaction request.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when estimation fails.
    fn estimate_gas(&self, tx: &TransactionRequest) -> Result<Amount, Self::Error>;
}

/// Asynchronous owner-address capability.
///
/// This narrow trait lets async flows ask only for signer ownership when no
/// signing operation is required.
#[allow(async_fn_in_trait)]
pub trait AsyncOwner {
    /// Error type returned by owner resolution.
    type Error;

    /// Returns the signer address.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when address resolution fails.
    async fn get_address(&self) -> Result<Address, Self::Error>;
}

/// Asynchronous EIP-712 typed-data signing capability.
#[allow(async_fn_in_trait)]
pub trait AsyncTypedDataSigner {
    /// Error type returned by typed-data signing.
    type Error;

    /// Signs an explicit typed-data payload.
    ///
    /// # Errors
    ///
    /// Returns any error from [`AsyncTypedDataSigner::sign_typed_data`].
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
}

/// Asynchronous digest-signing capability.
#[allow(async_fn_in_trait)]
pub trait AsyncDigestSigner {
    /// Error type returned by digest signing.
    type Error;

    /// Signs raw digest bytes according to the backend's message-signing rules.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when signing fails.
    async fn sign_digest(&self, digest: &[u8]) -> Result<String, Self::Error>;
}

/// Asynchronous EIP-1193 request capability.
#[allow(async_fn_in_trait)]
pub trait AsyncEip1193 {
    /// Error type returned by provider requests.
    type Error;

    /// Executes an EIP-1193 request with string parameters.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined provider error when the request fails.
    async fn request(&self, method: &str, params: &[String]) -> Result<String, Self::Error>;
}

/// Asynchronous signing boundary for wallets and async runtimes.
///
/// Synchronous signers implement this trait through the blanket implementation
/// so native trading flows can keep one async-first internal path. Narrow async
/// capability traits above are preferred for callback-shaped adapters that only
/// expose one signing operation.
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
    /// Sends a transaction and returns the broadcast transaction hash.
    ///
    /// This confirms only that the signer backend returned a transaction hash.
    /// Use [`AsyncProvider::get_transaction_receipt`] or a higher-level
    /// `cow-sdk-trading` wait helper to observe mining status and receipt
    /// fields.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined signer error when submission fails.
    async fn send_transaction(
        &self,
        tx: &TransactionRequest,
    ) -> Result<TransactionBroadcast, Self::Error>;
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
    ) -> Result<TransactionBroadcast, Self::Error> {
        Signer::send_transaction(self, tx)
    }

    async fn estimate_gas(&self, tx: &TransactionRequest) -> Result<Amount, Self::Error> {
        Signer::estimate_gas(self, tx)
    }
}

impl<T> AsyncOwner for T
where
    T: AsyncSigner,
{
    type Error = T::Error;

    async fn get_address(&self) -> Result<Address, Self::Error> {
        AsyncSigner::get_address(self).await
    }
}

impl<T> AsyncTypedDataSigner for T
where
    T: AsyncSigner,
{
    type Error = T::Error;

    async fn sign_typed_data_payload(
        &self,
        payload: &TypedDataPayload,
    ) -> Result<String, Self::Error> {
        AsyncSigner::sign_typed_data_payload(self, payload).await
    }

    async fn sign_typed_data(
        &self,
        domain: &TypedDataDomain,
        fields: &[TypedDataField],
        value_json: &str,
    ) -> Result<String, Self::Error> {
        AsyncSigner::sign_typed_data(self, domain, fields, value_json).await
    }
}

impl<T> AsyncDigestSigner for T
where
    T: AsyncSigner,
{
    type Error = T::Error;

    async fn sign_digest(&self, digest: &[u8]) -> Result<String, Self::Error> {
        AsyncSigner::sign_message(self, digest).await
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
    /// Returns mined receipt information for a transaction hash, if known.
    ///
    /// Implementations may return hash-only receipts while richer lifecycle
    /// fields are unavailable from the backend.
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

/// Read-only asynchronous provider boundary for browser wallets, native RPC
/// adapters, and async runtimes.
///
/// This trait intentionally contains only read-only chain RPC methods. Providers
/// that can create signers implement [`AsyncSigningProvider`] in addition to
/// this trait. This keeps read-only adapters free from signer dependencies while
/// preserving signer creation for wallet-capable providers.
///
/// # Examples
///
/// A read-only provider implements `AsyncProvider` without any signer wiring:
///
/// ```
/// # use cow_sdk_core::{
/// #     Address, AsyncProvider, BlockInfo, ContractCall, ContractHandle, Hash32, HexData,
/// #     TransactionHash, TransactionReceipt, TransactionRequest,
/// # };
/// struct ReadOnlyProvider;
///
/// impl AsyncProvider for ReadOnlyProvider {
///     type Error = String;
///
///     async fn get_chain_id(&self) -> Result<u64, Self::Error> {
///         Ok(1)
///     }
///
/// #   async fn get_code(&self, _address: &Address) -> Result<Option<HexData>, Self::Error> {
/// #       Ok(None)
/// #   }
/// #   async fn get_transaction_receipt(
/// #       &self,
/// #       _transaction_hash: &TransactionHash,
/// #   ) -> Result<Option<TransactionReceipt>, Self::Error> {
/// #       Ok(None)
/// #   }
/// #   async fn get_storage_at(
/// #       &self,
/// #       _address: &Address,
/// #       _slot: &str,
/// #   ) -> Result<HexData, Self::Error> {
/// #       Ok(HexData::new("0x").unwrap())
/// #   }
/// #   async fn call(&self, _tx: &TransactionRequest) -> Result<HexData, Self::Error> {
/// #       Ok(HexData::new("0x").unwrap())
/// #   }
/// #   async fn read_contract(&self, _request: &ContractCall) -> Result<String, Self::Error> {
/// #       Ok("null".to_owned())
/// #   }
/// #   async fn get_block(&self, _block_tag: &str) -> Result<BlockInfo, Self::Error> {
/// #       Ok(BlockInfo::new(1, None))
/// #   }
/// #   async fn get_contract(
/// #       &self,
/// #       address: &Address,
/// #       abi_json: &str,
/// #   ) -> Result<ContractHandle, Self::Error> {
/// #       Ok(ContractHandle::new(address.clone(), abi_json.to_owned()))
/// #   }
/// }
///
/// # #[tokio::main]
/// # async fn main() {
/// let provider = ReadOnlyProvider;
/// assert_eq!(provider.get_chain_id().await.unwrap(), 1);
/// # }
/// ```
///
/// Read-only providers do not satisfy `AsyncSigningProvider` unless they
/// explicitly implement the signing-capable extension:
///
/// ```compile_fail
/// # use cow_sdk_core::{
/// #     Address, AsyncProvider, AsyncSigningProvider, BlockInfo, ContractCall, ContractHandle,
/// #     HexData, TransactionHash, TransactionReceipt, TransactionRequest,
/// # };
/// # struct ReadOnlyProvider;
/// # impl AsyncProvider for ReadOnlyProvider {
/// #     type Error = String;
/// #     async fn get_chain_id(&self) -> Result<u64, Self::Error> { Ok(1) }
/// #     async fn get_code(&self, _address: &Address) -> Result<Option<HexData>, Self::Error> { Ok(None) }
/// #     async fn get_transaction_receipt(
/// #         &self,
/// #         _transaction_hash: &TransactionHash,
/// #     ) -> Result<Option<TransactionReceipt>, Self::Error> { Ok(None) }
/// #     async fn get_storage_at(
/// #         &self,
/// #         _address: &Address,
/// #         _slot: &str,
/// #     ) -> Result<HexData, Self::Error> { Ok(HexData::new("0x").unwrap()) }
/// #     async fn call(&self, _tx: &TransactionRequest) -> Result<HexData, Self::Error> { Ok(HexData::new("0x").unwrap()) }
/// #     async fn read_contract(&self, _request: &ContractCall) -> Result<String, Self::Error> { Ok("null".to_owned()) }
/// #     async fn get_block(&self, _block_tag: &str) -> Result<BlockInfo, Self::Error> { Ok(BlockInfo::new(1, None)) }
/// #     async fn get_contract(
/// #         &self,
/// #         address: &Address,
/// #         abi_json: &str,
/// #     ) -> Result<ContractHandle, Self::Error> {
/// #         Ok(ContractHandle::new(address.clone(), abi_json.to_owned()))
/// #     }
/// # }
/// fn requires_signing<P: AsyncSigningProvider>(_provider: &P) {}
///
/// fn main() {
///     let provider = ReadOnlyProvider;
///     requires_signing(&provider);
/// }
/// ```
#[allow(async_fn_in_trait)]
pub trait AsyncProvider {
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
    /// Returns mined receipt information for a transaction hash, if known.
    ///
    /// Implementations may return hash-only receipts while richer lifecycle
    /// fields are unavailable from the backend.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined provider error when the backend lookup fails.
    async fn get_transaction_receipt(
        &self,
        transaction_hash: &TransactionHash,
    ) -> Result<Option<TransactionReceipt>, Self::Error>;
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
{
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

/// Signing-capable extension for asynchronous provider implementations.
///
/// Wallet-capable providers implement this trait in addition to
/// [`AsyncProvider`]. Read-only adapters implement only `AsyncProvider`.
///
/// # Examples
///
/// ```
/// # use cow_sdk_core::{
/// #     Address, Amount, AsyncProvider, AsyncSigner, AsyncSigningProvider, BlockInfo,
/// #     ContractCall, ContractHandle, Hash32, HexData, TransactionHash, TransactionReceipt,
/// #     TransactionBroadcast, TransactionRequest, TypedDataDomain, TypedDataField,
/// # };
/// #[derive(Clone)]
/// struct WalletSigner;
///
/// impl AsyncSigner for WalletSigner {
///     type Error = String;
///
///     async fn get_address(&self) -> Result<Address, Self::Error> {
///         Address::new("0x1111111111111111111111111111111111111111")
///             .map_err(|error| error.to_string())
///     }
///
/// #   async fn sign_message(&self, _message: &[u8]) -> Result<String, Self::Error> {
/// #       Ok("0xsigned".to_owned())
/// #   }
/// #   async fn sign_transaction(&self, _tx: &TransactionRequest) -> Result<String, Self::Error> {
/// #       Ok("0xsigned".to_owned())
/// #   }
/// #   async fn sign_typed_data(
/// #       &self,
/// #       _domain: &TypedDataDomain,
/// #       _fields: &[TypedDataField],
/// #       _value_json: &str,
/// #   ) -> Result<String, Self::Error> {
/// #       Ok("0xsigned".to_owned())
/// #   }
/// #   async fn send_transaction(
/// #       &self,
/// #       _tx: &TransactionRequest,
/// #   ) -> Result<TransactionBroadcast, Self::Error> {
/// #       Ok(TransactionBroadcast::new(Hash32::new(format!("0x{}", "11".repeat(32))).unwrap()))
/// #   }
/// #   async fn estimate_gas(&self, _tx: &TransactionRequest) -> Result<Amount, Self::Error> {
/// #       Ok(Amount::from(21_000u32))
/// #   }
/// }
///
/// struct WalletProvider;
///
/// impl AsyncProvider for WalletProvider {
///     type Error = String;
///
///     async fn get_chain_id(&self) -> Result<u64, Self::Error> {
///         Ok(1)
///     }
///
/// #   async fn get_code(&self, _address: &Address) -> Result<Option<HexData>, Self::Error> {
/// #       Ok(None)
/// #   }
/// #   async fn get_transaction_receipt(
/// #       &self,
/// #       _transaction_hash: &TransactionHash,
/// #   ) -> Result<Option<TransactionReceipt>, Self::Error> {
/// #       Ok(None)
/// #   }
/// #   async fn get_storage_at(
/// #       &self,
/// #       _address: &Address,
/// #       _slot: &str,
/// #   ) -> Result<HexData, Self::Error> {
/// #       Ok(HexData::new("0x").unwrap())
/// #   }
/// #   async fn call(&self, _tx: &TransactionRequest) -> Result<HexData, Self::Error> {
/// #       Ok(HexData::new("0x").unwrap())
/// #   }
/// #   async fn read_contract(&self, _request: &ContractCall) -> Result<String, Self::Error> {
/// #       Ok("null".to_owned())
/// #   }
/// #   async fn get_block(&self, _block_tag: &str) -> Result<BlockInfo, Self::Error> {
/// #       Ok(BlockInfo::new(1, None))
/// #   }
/// #   async fn get_contract(
/// #       &self,
/// #       address: &Address,
/// #       abi_json: &str,
/// #   ) -> Result<ContractHandle, Self::Error> {
/// #       Ok(ContractHandle::new(address.clone(), abi_json.to_owned()))
/// #   }
/// }
///
/// impl AsyncSigningProvider for WalletProvider {
///     type Signer = WalletSigner;
///
///     async fn create_signer(&self, _signer_hint: &str) -> Result<Self::Signer, Self::Error> {
///         Ok(WalletSigner)
///     }
/// }
///
/// # #[tokio::main]
/// # async fn main() {
/// let provider = WalletProvider;
/// let signer = provider.create_signer("primary").await.unwrap();
/// assert_eq!(
///     signer.get_address().await.unwrap().as_str(),
///     "0x1111111111111111111111111111111111111111"
/// );
/// # }
/// ```
#[allow(async_fn_in_trait)]
pub trait AsyncSigningProvider: AsyncProvider {
    /// Signer type exposed by this provider.
    type Signer: AsyncSigner<Error = Self::Error>;

    /// Creates a signer from an implementation-defined hint.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined provider error when signer creation fails.
    async fn create_signer(&self, signer_hint: &str) -> Result<Self::Signer, Self::Error>;
}

impl<T> AsyncSigningProvider for T
where
    T: Provider,
    T::Signer: AsyncSigner<Error = T::Error>,
{
    type Signer = T::Signer;

    async fn create_signer(&self, signer_hint: &str) -> Result<Self::Signer, Self::Error> {
        Provider::create_signer(self, signer_hint)
    }
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
