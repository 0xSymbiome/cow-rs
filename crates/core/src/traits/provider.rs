use serde::{Deserialize, Serialize};

use crate::types::{Address, ChainId, HexData, LogQuery, RawLog, TransactionHash};

use super::signer::Signer;
use super::transaction::{BlockInfo, TransactionReceipt, TransactionRequest};

/// Read-only provider boundary for browser wallets, native RPC adapters, and
/// runtimes that expose chain data.
///
/// This trait intentionally contains only read-only chain RPC methods. Providers
/// that can create signers implement [`SigningProvider`] in addition to this
/// trait. This keeps read-only adapters free from signer dependencies while
/// preserving signer creation for wallet-capable providers.
///
/// # Examples
///
/// A read-only provider implements `Provider` without any signer wiring:
///
/// ```
/// # use cow_sdk_core::{
/// #     Address, BlockInfo, ContractCall, ContractHandle, Hash32, HexData, Provider,
/// #     TransactionHash, TransactionReceipt, TransactionRequest,
/// # };
/// struct ReadOnlyProvider;
///
/// impl Provider for ReadOnlyProvider {
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
/// Read-only providers do not satisfy `SigningProvider` unless they
/// explicitly implement the signing-capable extension:
///
/// ```compile_fail
/// # use cow_sdk_core::{
/// #     Address, BlockInfo, ContractCall, ContractHandle, HexData, Provider, SigningProvider,
/// #     TransactionHash, TransactionReceipt, TransactionRequest,
/// # };
/// # struct ReadOnlyProvider;
/// # impl Provider for ReadOnlyProvider {
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
/// fn requires_signing<P: SigningProvider>(_provider: &P) {}
///
/// fn main() {
///     let provider = ReadOnlyProvider;
///     requires_signing(&provider);
/// }
/// ```
#[expect(
    async_fn_in_trait,
    reason = "the trait surface adopts native async fn in trait per ADR 0010 runtime-neutral posture; the resulting non-Send futures are covered by the workspace future_not_send allow so wasm callbacks can satisfy the same trait without an explicit Send bound"
)]
pub trait Provider {
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
    /// Executes a contract read against a runtime-supplied ABI and returns the
    /// decoded result serialized as JSON.
    ///
    /// The ABI and arguments are supplied dynamically on [`ContractCall`], so the
    /// decoded output cannot be a compile-time type; it is returned as a
    /// serialized JSON string. This is also the form the result takes when the
    /// read is served across a TypeScript/WASM callback boundary, where JSON is
    /// the wire shape. It is therefore an explicit serialized boundary (see
    /// ADR 0005), and callers decode it into strong domain types through the
    /// dedicated helpers — for example the allowance reader in `cow-sdk-trading`
    /// and the EIP-1271 magic-value decoder in `cow-sdk-contracts` — rather than
    /// matching on the raw string.
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

/// Signing-capable extension for provider implementations.
///
/// Wallet-capable providers implement this trait in addition to
/// [`Provider`]. Read-only adapters implement only `Provider`.
///
/// # Examples
///
/// ```
/// # use cow_sdk_core::{
/// #     Address, Amount, BlockInfo, ContractCall, ContractHandle, Hash32, HexData, Provider,
/// #     Signer, SigningProvider, TransactionHash, TransactionReceipt, TransactionBroadcast,
/// #     TransactionRequest, TypedDataDomain, TypedDataField,
/// # };
/// #[derive(Clone)]
/// struct WalletSigner;
///
/// impl Signer for WalletSigner {
///     type Error = String;
///
///     async fn address(&self) -> Result<Address, Self::Error> {
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
/// impl Provider for WalletProvider {
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
/// impl SigningProvider for WalletProvider {
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
///     signer.address().await.unwrap().to_hex_string(),
///     "0x1111111111111111111111111111111111111111"
/// );
/// # }
/// ```
#[expect(
    async_fn_in_trait,
    reason = "the trait surface adopts native async fn in trait per ADR 0010 runtime-neutral posture; the resulting non-Send futures are covered by the workspace future_not_send allow so wasm callbacks can satisfy the same trait without an explicit Send bound"
)]
pub trait SigningProvider: Provider {
    /// Signer type exposed by this provider.
    type Signer: Signer<Error = Self::Error>;

    /// Creates a signer from an implementation-defined hint.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined provider error when signer creation fails.
    async fn create_signer(&self, signer_hint: &str) -> Result<Self::Signer, Self::Error>;
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

/// Log-fetch capability for providers that can serve `eth_getLogs`.
///
/// This is an opt-in capability supertrait layered on [`Provider`], mirroring
/// the [`SigningProvider`](super::SigningProvider) split: read-only adapters
/// implement only `Provider`; adapters that can additionally fetch event logs
/// implement `LogProvider`. A leaf crate bounds on `P: LogProvider` to fetch
/// logs without depending on any concrete provider adapter, and a read-only
/// adapter is never forced to carry log-fetch wiring it cannot serve.
///
/// [`get_logs`](LogProvider::get_logs) is the single bounded-call event scan:
/// one backend query over the caller's `[from_block, to_block]` range, returning
/// the raw logs for the caller to decode. It is deliberately not a watcher,
/// iterator, or indexer loop (ADR 0048); a caller that needs a wider range
/// issues further bounded calls itself.
///
/// ```
/// use cow_sdk_core::{LogProvider, LogQuery, RawLog};
///
/// async fn recent_logs<P: LogProvider>(provider: &P) -> Result<Vec<RawLog>, P::Error> {
///     provider.get_logs(&LogQuery::new(20_000_000, 20_000_100)).await
/// }
/// ```
#[expect(
    async_fn_in_trait,
    reason = "the trait surface adopts native async fn in trait per ADR 0010 runtime-neutral posture; the resulting non-Send futures are covered by the workspace future_not_send allow so wasm callbacks can satisfy the same trait without an explicit Send bound"
)]
pub trait LogProvider: Provider {
    /// Fetches event logs matching `query` in a single backend call.
    ///
    /// Issues exactly one backend log query over the query's caller-bounded
    /// `[from_block, to_block]` range and returns the raw logs for the caller to
    /// decode. A block range yields heterogeneous events, so decoding is left to
    /// the caller's family-specific decoder (`decode_settlement_log`,
    /// `decode_eth_flow_log`, …) applied to each [`RawLog::data`].
    /// Implementations must not expand the range, loop, poll, or watch.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined provider error when the log query
    /// fails.
    async fn get_logs(&self, query: &LogQuery) -> Result<Vec<RawLog>, Self::Error>;
}
