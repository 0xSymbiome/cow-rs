use crate::types::{Address, ChainId, HexData, TransactionHash};

use super::contract::{ContractCall, ContractHandle};
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
///     signer.get_address().await.unwrap().to_hex_string(),
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
