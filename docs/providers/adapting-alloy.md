# Adapting alloy

This page shows how to adapt `alloy::providers::Provider` and
`alloy::signers::Signer` to the `cow-sdk-core` trait surface. It is a
seam demonstration, so the code below documents the method-by-method
shape of an idiomatic adapter rather than a compiled implementation
inside the repository.

## Purpose

Give integrators a copyable reference for wiring an alloy-powered
RPC client and signer into `TradingSdk` and the lower-level crate
surfaces. Every method the SDK ever calls on a provider or signer is
covered by one of the four trait methods demonstrated here, so the
adapter is the only place that needs to know about the alloy-specific
types.

## Prerequisites

- Rust toolchain pinned by `rust-toolchain.toml` (MSRV 1.94.0).
- `alloy` 1.x (the 0.x line moved into `alloy` in 2025; the examples
  below reference the 1.x trait names).
- The workspace has `cow-sdk-core` on its dependency list.
  `cow-rs` pins `alloy-sol-macro`, `alloy-sol-types`, `alloy-primitives`,
  `alloy-dyn-abi`, and `alloy-json-abi` in its workspace for the
  deterministic binding, typed-data, and redaction surfaces; the
  chain-RPC half (`alloy-provider`) is intentionally NOT a workspace
  dependency, so consumers select their own provider ecosystem and
  bring the adapter described below.

## Public Trait Surface

The adapter implements two async traits from `cow-sdk-core::traits`:

- [`AsyncProvider`](https://docs.rs/cow-sdk-core/latest/cow_sdk_core/traits/trait.AsyncProvider.html)
  for chain reads, storage reads, contract calls, receipts, and
  contract handle construction.
- [`AsyncSigner`](https://docs.rs/cow-sdk-core/latest/cow_sdk_core/traits/trait.AsyncSigner.html)
  for address resolution, message signing, typed-data signing,
  transaction signing, transaction submission, and gas estimation.

Every input and output type the traits use
(`Address`, `ChainId`, `HexData`, `Amount`, `TransactionHash`,
`TransactionRequest`, `TransactionReceipt`, `BlockInfo`,
`ContractCall`, `ContractHandle`, `TypedDataDomain`,
`TypedDataField`) is defined in `cow_sdk_core::types` and
`cow_sdk_core::traits`.

## Implementing `AsyncProvider` And `AsyncSigningProvider`

The adapter wraps a single alloy provider handle and converts typed
inputs from `cow-sdk-core` into the alloy call shape, then converts
results back into the `cow-sdk-core` types the SDK expects.

```rust,no_run
use std::sync::Arc;

use cow_sdk_core::traits::{
    AsyncProvider, AsyncSigningProvider, BlockInfo, ContractCall, ContractHandle,
    TransactionReceipt, TransactionRequest,
};
use cow_sdk_core::types::{Address, Amount, BlockHash, ChainId, HexData, TransactionHash};

// Replace `AlloyProviderHandle` and `AlloySigner` with the concrete types from
// your alloy integration (for example `alloy::providers::RootProvider<Http>`
// and `alloy::signers::local::PrivateKeySigner`).
pub struct AlloyProviderAdapter<P> {
    provider: Arc<P>,
}

#[derive(Debug, thiserror::Error)]
pub enum AlloyAdapterError {
    #[error("alloy transport error: {0}")]
    Transport(String),
    #[error("missing field in alloy response: {0}")]
    MissingField(&'static str),
    #[error("conversion error: {0}")]
    Conversion(String),
}

pub struct AlloySignerAdapter<S> {
    signer: Arc<S>,
}

#[allow(unused_variables)]
impl<P> AsyncProvider for AlloyProviderAdapter<P>
where
    P: Send + Sync,
{
    type Error = AlloyAdapterError;

    async fn get_chain_id(&self) -> Result<ChainId, Self::Error> {
        // let id: u64 = self.provider.get_chain_id().await.map_err(|e| AlloyAdapterError::Transport(e.to_string()))?;
        // Ok(id)
        Ok(1)
    }

    async fn get_code(&self, address: &Address) -> Result<Option<HexData>, Self::Error> {
        // let bytes = self.provider.get_code_at(address.into_alloy()).await.map_err(...)?;
        // Ok(Some(HexData::from(bytes.as_ref())))
        unimplemented!("wire alloy provider.get_code_at and wrap bytes in HexData")
    }

    async fn get_transaction_receipt(
        &self,
        transaction_hash: &TransactionHash,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        // let receipt = self.provider.get_transaction_receipt(transaction_hash.into_alloy()).await.map_err(...)?;
        // Ok(receipt.map(|r| TransactionReceipt::new(TransactionHash::from(r.transaction_hash))))
        unimplemented!("wire alloy get_transaction_receipt and map to TransactionReceipt")
    }

    async fn get_storage_at(&self, address: &Address, slot: &str) -> Result<HexData, Self::Error> {
        // let value = self.provider.get_storage_at(address.into_alloy(), slot.parse()?).await.map_err(...)?;
        // Ok(HexData::from(value.as_le_bytes()))
        unimplemented!("wire alloy get_storage_at and wrap the slot value in HexData")
    }

    async fn call(&self, tx: &TransactionRequest) -> Result<HexData, Self::Error> {
        // let alloy_tx = to_alloy_tx(tx)?;
        // let returned = self.provider.call(&alloy_tx).await.map_err(...)?;
        // Ok(HexData::from(returned.as_ref()))
        unimplemented!("wire alloy call and wrap the return data in HexData")
    }

    async fn read_contract(&self, request: &ContractCall) -> Result<String, Self::Error> {
        // Parse `request.abi_json` + `request.args_json` through the alloy JsonAbi / DynSolValue family.
        unimplemented!("execute the alloy contract read and serialize the return value to JSON")
    }

    async fn get_block(&self, block_tag: &str) -> Result<BlockInfo, Self::Error> {
        // let tag = parse_tag(block_tag)?;
        // let block = self.provider.get_block(tag).await.map_err(...)?.ok_or(AlloyAdapterError::MissingField("block"))?;
        // Ok(BlockInfo::new(block.number, block.hash.map(BlockHash::from)))
        unimplemented!("map the alloy block response to cow_sdk_core::traits::BlockInfo")
    }

    async fn get_contract(
        &self,
        address: &Address,
        abi_json: &str,
    ) -> Result<ContractHandle, Self::Error> {
        // Instantiate the alloy Contract<...> and return a handle that carries the typed ABI back to the SDK.
        unimplemented!("construct the alloy contract handle and return a ContractHandle wrapper")
    }
}

#[allow(unused_variables)]
impl<P> AsyncSigningProvider for AlloyProviderAdapter<P>
where
    P: Send + Sync,
{
    type Signer = AlloySignerAdapter<()>; // replace `()` with your signer handle

    async fn create_signer(&self, signer_hint: &str) -> Result<Self::Signer, Self::Error> {
        // Parse `signer_hint` (for example a private-key hex or keystore path) using your signer factory.
        unimplemented!("build the signer handle the adapter needs from signer_hint")
    }
}
```

The concrete `get_chain_id` body demonstrates the minimum shape of a method
implementation: await the alloy call in a real adapter, map the transport
error into the adapter error type, and return the `cow-sdk-core` primitive.
The remaining `unimplemented!` bodies mark the lines where a real adapter
crate would call into alloy. The `AsyncProvider` method signatures are fixed by
the read-only trait; the adapter just provides the conversion between
`cow-sdk-core` types and the alloy equivalents. Implement
`AsyncSigningProvider` only when the same adapter can create a signer.

## Implementing `AsyncSigner`

`AsyncSigner` is the callable the trading flows use for signing and
submission. The default `sign_typed_data_payload` method is provided
by the trait, so an adapter only has to implement the six primitive
methods below.

```rust,no_run
use cow_sdk_core::traits::{
    AsyncSigner, TransactionReceipt, TransactionRequest, TypedDataDomain, TypedDataField,
};
use cow_sdk_core::types::{Address, Amount};

use crate::{AlloyAdapterError, AlloySignerAdapter};

#[allow(unused_variables)]
impl<S> AsyncSigner for AlloySignerAdapter<S>
where
    S: Send + Sync,
{
    type Error = AlloyAdapterError;

    async fn get_address(&self) -> Result<Address, Self::Error> {
        // let addr = self.signer.address();
        // Ok(Address::from(addr))
        unimplemented!("call alloy signer.address() and wrap it in cow_sdk_core::types::Address")
    }

    async fn sign_message(&self, message: &[u8]) -> Result<String, Self::Error> {
        // let sig = self.signer.sign_message(message).await.map_err(|e| AlloyAdapterError::Transport(e.to_string()))?;
        // Ok(format!("0x{}", hex::encode(sig.as_bytes())))
        unimplemented!("call alloy signer.sign_message and return the 0x-prefixed hex signature")
    }

    async fn sign_transaction(&self, tx: &TransactionRequest) -> Result<String, Self::Error> {
        // Convert `tx` to an alloy `TransactionRequest`, sign it, and return the RLP-encoded
        // signed transaction as 0x-prefixed hex. The alloy-specific conversion is the only
        // place the adapter owns.
        unimplemented!("sign the alloy transaction and return the 0x-prefixed signed payload")
    }

    async fn sign_typed_data(
        &self,
        domain: &TypedDataDomain,
        fields: &[TypedDataField],
        value_json: &str,
    ) -> Result<String, Self::Error> {
        // Build an alloy `TypedData` value from the three inputs, then call
        // `alloy::signers::Signer::sign_typed_data`. Return the 0x-prefixed hex signature.
        unimplemented!("assemble the alloy TypedData and return the EIP-712 signature as hex")
    }

    async fn send_transaction(
        &self,
        tx: &TransactionRequest,
    ) -> Result<TransactionReceipt, Self::Error> {
        // Submit through the attached alloy provider and wait for the transaction receipt.
        // Return a `TransactionReceipt` populated with the transaction hash.
        unimplemented!("submit via the alloy provider and return the receipt hash")
    }

    async fn estimate_gas(&self, tx: &TransactionRequest) -> Result<Amount, Self::Error> {
        // let estimate = self.provider.estimate_gas(&to_alloy_tx(tx)?).await.map_err(...)?;
        // Ok(Amount::from(estimate))
        unimplemented!("call alloy estimate_gas and wrap the result in cow_sdk_core::types::Amount")
    }
}
```

## Consumption Example

Once the adapter implements `AsyncProvider`, `AsyncSigningProvider`, and `AsyncSigner`, the
consumer wires it into the higher-level SDK surfaces the same way any
other adapter does:

```rust,no_run
# use std::sync::Arc;
# use cow_sdk_core::types::ChainId;
// Pseudocode: replace the Alloy* types with the concrete wrappers from the
// snippets above.
# struct AlloyProviderAdapter<T>(T);
# struct AlloyClient;
# let alloy_client = AlloyClient;
# fn build_alloy_signer() -> () { () }
let provider = AlloyProviderAdapter {
    provider: Arc::new(alloy_client),
};
let signer = build_alloy_signer();

// Hand the provider or signer to any cow-sdk surface that takes an
// `AsyncProvider`, `AsyncSigningProvider`, or `AsyncSigner`. Crates such
// as cow-sdk-trading, cow-sdk-signing, and cow-sdk-browser-wallet accept
// trait objects or generic parameters bound to these traits.
```

The exact call shape of the SDK entry points that accept the adapter
depends on the consumer crate. See the crate-level rustdoc for the
current recommended builder surface.

## Scope Boundary

- This page documents how to implement the `cow-sdk-core` trait
  surface against an external provider and signer library.
- The alloy crates are caller-supplied dependencies. `cow-rs` does
  not depend on alloy directly; the adapter lives in the consumer
  workspace.
