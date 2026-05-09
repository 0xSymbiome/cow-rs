# Integrations

This guide explains how native runtime adapters plug into the public `cow-rs`
surface.

Use it when you want to connect the SDK to:

- a custom RPC backend
- a custom signer implementation
- a provider ecosystem that is intentionally outside the default facade

The stable extension seam is owned by `cow-sdk-core`.

The root `cow-sdk` facade re-exports the traits for convenience, but the
contract itself lives in `cow-sdk-core::{Signer, AsyncSigner,
AsyncSigningProvider, Provider, AsyncProvider}`.

## Why This Guide Exists

`cow-rs` keeps provider ecosystems out of the default facade.

That design keeps the shipped surface:

- provider-agnostic
- easier to audit because every credential-bearing error surface is covered by
  [Credential Surface Audit](audit/credential-surface-audit.md), and long
  async methods are covered by
  [Cooperative Cancellation Contract Audit](audit/cooperative-cancellation-contract-audit.md)
- additive for downstream runtime crates

If you need native Alloy, use the shipped adapter crates. For a custom
in-house runtime, build an adapter in a leaf crate that implements the core
traits.

The native Alloy adapter exists to wire Alloy into the SDK's trading and
signing contracts. It is not a general Alloy improvement. Consumers building
generic Ethereum applications without trading helpers should depend on Alloy
directly.

## Shipped Alloy Adapters

The native Alloy family is opt-in:

- `cow-sdk-alloy-provider` implements read-only `AsyncProvider`.
- `cow-sdk-alloy-signer` implements local private-key `AsyncSigner`.
- `cow-sdk-alloy` composes provider and signer support and implements
  `AsyncSigningProvider` for `TradingSdk` helper flows.

The root facade exposes matching features named `alloy-provider`,
`alloy-signer`, and `alloy`. These features are native-only and hard-fail on
`wasm32-unknown-unknown`; browser integrations should use
`cow-sdk-browser-wallet`.

## The Six Runtime Seams

Import the owning traits from `cow-sdk-core`:

```rust
use cow_sdk_core::{
    AsyncProvider, AsyncSigner, AsyncSigningProvider, HttpTransport, Provider, Signer,
};
```

Their roles are:

`Signer`

- synchronous signing and transaction submission for native or test runtimes

`AsyncSigner`

- asynchronous signing for browser wallets and async-native runtimes

`Provider`

- synchronous chain reads, contract reads, signer creation, and signer
  attachment

`AsyncProvider`

- asynchronous chain reads and contract reads for browser or async runtimes

`AsyncSigningProvider`

- signer creation for async providers that can create wallet or hosted signers

`HttpTransport`

- production HTTPS dispatch for the orderbook and subgraph clients. The
  native default is `ReqwestTransport`; the browser default is
  `FetchTransport` from `cow-sdk-transport-wasm`. Custom implementations
  install through the builder's `.transport(Arc::new(...))` setter on
  both `OrderBookApi::builder()` and `SubgraphApi::builder()`. See
  [Transport](transport.md) for the full seam.

## TypeScript And JavaScript Runtime Boundary

`cow-sdk-wasm` exposes the SDK to JavaScript through typed callbacks rather
than a bundled wallet or HTTP library. It names five host callbacks:
`TypedDataSignerCallback`, `Eip1193RequestCallback`, `DigestSignerCallback`,
`CustomEip1271Callback`, and `CowFetchCallback`. The callback HTTP transport
uses SDK-owned timeout and a live `AbortSignal`, while the host runtime owns
actual network dispatch.

| Audience | Path |
| --- | --- |
| Native Rust services, bots, solvers, analytics | `cow-sdk` |
| Native Rust apps using Alloy directly | `cow-sdk` plus `cow-sdk-alloy-*` |
| Rust applications that compile to WASM and run in a browser | `cow-sdk-browser-wallet` plus `cow-sdk-transport-wasm` |
| TypeScript apps that want SDK-managed browser wallet flows | `cow-sdk-browser-wallet` (convenience integration) |
| TypeScript apps using viem, ethers, wagmi, or any EIP-1193 wallet | `cow-sdk-wasm` (after publication) |
| Node.js LTS backends | `cow-sdk-wasm` (`nodejs` wasm-pack target) |
| Cloudflare Workers | `cow-sdk-wasm` with callback transport (`OrderBookClientWithFetch`) |
| Deno (optional / experimental) | `cow-sdk-wasm` (`deno` wasm-pack target, opt-in only via `BUILD_DENO=1`; `./deno` npm export absent by default) |

`signOrderWithCustomEip1271` is the smart-account integration point when a
JavaScript application owns the account-abstraction client and the SDK should
only consume the resolved EIP-1271 signature.

## Contract Shape

The traits are intentionally narrow.

### `Signer`

A sync signer owns:

- provider attachment via `connect`
- address resolution via `get_address`
- message signing via `sign_message`
- transaction signing via `sign_transaction`
- typed-data signing via `sign_typed_data` or `sign_typed_data_payload`
- transaction submission via `send_transaction`, which returns a
  `TransactionBroadcast` carrying the broadcast hash. This is not a mined
  receipt and does not prove block inclusion or execution success.
- gas estimation via `estimate_gas`

### `AsyncSigner`

An async signer owns the same conceptual operations as `Signer`, but exposes
them as async methods.

Browser-wallet support implements `AsyncSigner` directly.

### `Provider`

A sync provider owns:

- optional signer exposure through `signer_or_null`
- chain id lookup
- code lookup
- transaction-receipt lookup
- signer creation from a runtime-specific hint
- storage lookup
- generic call execution
- typed contract reads through `read_contract`
- block lookup
- signer replacement
- provider replacement
- typed contract-handle creation

### `AsyncProvider`

An async provider owns the read-side operations from `Provider` in async form.

It does not expose signer creation, `set_signer`, or `set_provider`.

### `AsyncSigningProvider`

An async signing provider extends `AsyncProvider` with signer creation for
providers that can create wallet, hosted, or locally managed signers.

Read-only async providers do not implement this extension.

Those mutating hooks remain part of the sync provider seam.

## Important Compatibility Rule

You do **not** always need to implement all runtime traits separately.

`cow-sdk-core` already provides blanket implementations:

- any `T: Signer` also implements `AsyncSigner`
- any `T: Provider` also implements `AsyncProvider`
- any `T: Provider` also implements `AsyncSigningProvider` when `T::Signer` satisfies
  `AsyncSigner<Error = T::Error>`

That means a synchronous native adapter can often implement only:

- `Signer`
- `Provider`

and still satisfy async-first downstream helper paths through the blanket impls.

## Minimal Worked Example

The example below shows one small in-memory adapter pair:

- `StaticSigner`
- `StaticProvider`

It is intentionally simple.

Its job is to demonstrate the trait shape, not to model a production RPC stack.

```rust
use cow_sdk_core::{
    Address, Amount, BlockInfo, ChainId, ContractCall, ContractHandle, CoreError, HexData,
    Provider, Signer, TransactionBroadcast, TransactionHash, TransactionReceipt,
    TransactionRequest, TransactionStatus, TypedDataDomain, TypedDataField,
};

#[derive(Debug, Clone)]
struct StaticSigner {
    address: Address,
    receipt_hash: TransactionHash,
    gas_limit: Amount,
}

impl Signer for StaticSigner {
    type Provider = ();
    type Error = CoreError;

    fn connect(&mut self, _provider: Self::Provider) {}

    fn get_address(&self) -> Result<Address, Self::Error> {
        Ok(self.address.clone())
    }

    fn sign_message(&self, _message: &[u8]) -> Result<String, Self::Error> {
        Ok("0xfeedface".to_owned())
    }

    fn sign_transaction(&self, _tx: &TransactionRequest) -> Result<String, Self::Error> {
        Ok("0xdeadbeef".to_owned())
    }

    fn sign_typed_data(
        &self,
        _domain: &TypedDataDomain,
        _fields: &[TypedDataField],
        _value_json: &str,
    ) -> Result<String, Self::Error> {
        Ok("0xtypeddata".to_owned())
    }

    fn send_transaction(
        &self,
        _tx: &TransactionRequest,
    ) -> Result<TransactionBroadcast, Self::Error> {
        Ok(TransactionBroadcast::new(self.receipt_hash.clone()))
    }

    fn estimate_gas(&self, _tx: &TransactionRequest) -> Result<Amount, Self::Error> {
        Ok(self.gas_limit.clone())
    }
}

#[derive(Debug, Clone)]
struct StaticProvider {
    chain_id: ChainId,
    signer: StaticSigner,
    allowance_result: String,
}

impl Provider for StaticProvider {
    type Signer = StaticSigner;
    type Error = CoreError;

    fn signer_or_null(&self) -> Option<&Self::Signer> {
        Some(&self.signer)
    }

    fn get_chain_id(&self) -> Result<ChainId, Self::Error> {
        Ok(self.chain_id)
    }

    fn get_code(&self, _address: &Address) -> Result<Option<HexData>, Self::Error> {
        Ok(None)
    }

    fn get_transaction_receipt(
        &self,
        transaction_hash: &TransactionHash,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        Ok(Some(
            TransactionReceipt::new(transaction_hash.clone())
                .with_status(TransactionStatus::Success)
                .with_block_number(1)
                .with_gas_used(Amount::from(21_000u64))
                .with_from(self.signer.address.clone()),
        ))
    }

    fn create_signer(&self, _signer_hint: &str) -> Result<Self::Signer, Self::Error> {
        Ok(self.signer.clone())
    }

    fn get_storage_at(&self, _address: &Address, _slot: &str) -> Result<HexData, Self::Error> {
        HexData::new("0x00")
    }

    fn call(&self, _tx: &TransactionRequest) -> Result<HexData, Self::Error> {
        HexData::new("0x")
    }

    fn read_contract(&self, _request: &ContractCall) -> Result<String, Self::Error> {
        Ok(self.allowance_result.clone())
    }

    fn get_block(&self, _block_tag: &str) -> Result<BlockInfo, Self::Error> {
        Ok(BlockInfo::new(1, None))
    }

    fn set_signer(&mut self, signer: Self::Signer) {
        self.signer = signer;
    }

    fn set_provider(&mut self, _provider_hint: String) {}

    fn get_contract(
        &self,
        address: &Address,
        abi_json: &str,
    ) -> Result<ContractHandle, Self::Error> {
        Ok(ContractHandle::new(address.clone(), abi_json.to_owned()))
    }
}
```

### What The Example Shows

The example is intentionally small, but it already satisfies the stable native
integration contract:

- address resolution comes from the signer
- transaction submission comes from the signer
- transaction observation comes from the provider receipt lookup
- typed contract reads come from the provider
- signer creation is provider-owned
- the provider keeps chain authority

Because `StaticSigner` implements `Signer`, it also satisfies `AsyncSigner`
through the blanket implementation.

Because `StaticProvider` implements `Provider`, it also satisfies
`AsyncProvider`. Because its signer satisfies `AsyncSigner`, it also satisfies
`AsyncSigningProvider`.

## Using The Adapter With Downstream Helpers

Once your adapter implements the traits, you can pass it into downstream
helpers that are generic over `Provider`, `AsyncProvider`, `Signer`, or
`AsyncSigningProvider`, or `AsyncSigner`.

For example, the trading crate exposes allowance helpers over the provider seam:

```rust
use cow_sdk_core::{Address, CowEnv, SupportedChainId};
use cow_sdk_trading::get_cow_protocol_allowance_async;

async fn read_allowance(
    provider: &StaticProvider,
) -> Result<(), Box<dyn std::error::Error>> {
    let token = Address::new("0xfff9976782d46cc05630d1f6ebab18b2324d6b14")?;
    let owner = Address::new("0x1111111111111111111111111111111111111111")?;

    let allowance = get_cow_protocol_allowance_async(
        provider,
        &token,
        &owner,
        SupportedChainId::Sepolia,
        CowEnv::Prod,
        None,
    )
    .await?;

    println!("allowance={}", allowance.as_str());
    Ok(())
}
```

The important point is the trait seam, not the concrete helper.

Any adapter that satisfies the runtime-neutral traits can participate in the
same public helper surface.

## Design Guidance For Real Adapters

When you build a production adapter crate:

- keep the provider-specific code in a leaf crate
- keep `cow-sdk-core` as the seam owner
- keep error translation explicit at the adapter boundary
- prefer typed contract-read handling over ad hoc JSON strings outside the
  trait call itself
- keep secret-bearing transport config and logging policy outside generic
  public examples

## Sync Versus Async Choice

Use a sync-first adapter when:

- you are integrating a native runtime
- your signer and provider already expose blocking operations
- you want the blanket async compatibility for high-level SDK helpers

Use an async-direct adapter when:

- the runtime is browser-bound
- the wallet or transport is inherently async
- you want explicit control over async behavior rather than relying on the
  blanket implementation

## Relationship To The Default Facade

The root `cow-sdk` facade stays trading-first.

It does not freeze one provider ecosystem into the default package.

That is why this guide exists as a separate page instead of widening the
onboarding path or the root crate identity.

If you are still choosing crate surfaces, read [Architecture](architecture.md).

If you want the shortest deterministic onboarding path first, read
[Getting Started](getting-started.md).
