# Integrations

This guide explains how native runtime adapters plug into the public `cow-rs`
surface.

Use it when you want to connect the SDK to:

- a custom RPC backend
- a custom signer implementation
- a provider ecosystem that is intentionally outside the default facade

The stable extension seam is owned by `cow-sdk-core`.

The root `cow-sdk` facade re-exports the traits for convenience, but the
contract itself lives in `cow-sdk-core::{Signer, Provider,
SigningProvider}`.

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

## Primitive Types

Consumers integrating with `cow-sdk-*` use cow-named identity and numeric
types directly from `cow_sdk_core::types::*`. The types are cow-owned
`#[repr(transparent)]` newtypes around `alloy_primitives` primitives per
[ADR 0052](adr/0052-alloy-primitives-canonical-primitive-layer.md), so
bridging to alloy types is zero-cost via `From::from(addr).into()` or `.0`
access. Constructors validate input; the cow public surface preserves the
lowercase wire form for byte-typed identities (`Address`, `Hash32`,
`AppDataHash`, `HexData`, `OrderUid`) and the strict-decimal-only wire form
for the numeric types (`Amount`, `SignedAmount`).

## Shipped Alloy Adapters

The native Alloy family is opt-in:

- `cow-sdk-alloy-provider` implements read-only `Provider`.
- `cow-sdk-alloy-signer` implements local private-key `Signer`.
- `cow-sdk-alloy` composes provider and signer support and implements
  `SigningProvider` for `Trading` helper flows.

The root facade exposes matching features named `alloy-provider`,
`alloy-signer`, and `alloy`. These features are native-only and hard-fail on
`wasm32-unknown-unknown`; browser integrations should use
`cow-sdk-browser-wallet`.

## Composable And COW Shed Readiness

The composable-order and COW Shed surfaces are prepared as typed Rust evidence
before their full helper crate bodies are exposed. The readiness layer improves
on directly copying TypeScript package behavior in seven concrete ways:

- deployment addresses resolve through one typed schema v2 registry rather than
  package-local constants
- not-deployed and unsupported chains live in a coverage manifest instead of
  being mixed into addressable rows
- EIP-1271 custom signature production is owned by `cow-sdk-signing`, so trading
  workflows consume signatures without owning smart-account implementation
  details
- Byte-identical Solidity mirrors of every upstream contract surface are
  committed under the contracts crate and bound through `alloy::sol!`,
  gated by `cargo parity-verify-sol-provenance`, so hand-written ABI
  encoders never enter the workspace
- source commits and npm package integrity evidence are pinned together, with
  public generated documentation treated only as drift signal
- COW Shed proxy creation-code bytes are hashed at build time before address
  derivation fixtures are trusted
- watch-tower behavior is represented as selectors, decoders, and local
  simulation boundaries; service loops, persistence, notification delivery, and
  automatic order posting remain outside the SDK

## Runtime Seams

Import the owning traits from `cow-sdk-core`:

```rust
use cow_sdk_core::{Provider, Signer, SigningProvider, HttpTransport};
```

Their roles are:

`Signer`

- signing and transaction submission for browser wallets, hosted-key services,
  native key stores, and async-native runtimes

`Provider`

- chain reads and contract reads for any runtime

`SigningProvider`

- signer creation for providers that can construct wallet, hosted, or
  locally managed signers

`HttpTransport`

- production HTTPS dispatch for the orderbook and subgraph clients. The
  native default is `ReqwestTransport`; the browser default is
  `FetchTransport` from `cow-sdk-transport-wasm`. Custom implementations
  install through the builder's `.transport(Arc::new(...))` setter on
  both `OrderbookApi::builder()` and `SubgraphApi::builder()`. See
  [Transport](transport.md) for the full seam.

`cow-sdk-core` also ships narrower capability traits — [`Owner`],
[`TypedDataSigner`], and [`DigestSigner`] per
[ADR 0045](adr/0045-async-signer-trait-narrowing.md) — for callback-shaped
adapters that expose only one signing operation.

## TypeScript And JavaScript Runtime Boundary

For most browser dapps, web apps, and CowSwap-style UIs, the upstream
[`@cowprotocol/cow-sdk`](https://www.npmjs.com/package/@cowprotocol/cow-sdk)
TypeScript SDK is the recommended choice; it is substantially smaller at
equivalent feature subsets. `cow-sdk-wasm` is appropriate for specialized
cases — TypeScript services that need byte-for-byte parity with the Rust
SDK's signing path, single-source-of-truth Rust + TypeScript embedding, and
Cloudflare Workers (size-compatible at the time of measurement; full Workers
support pending release-bundle and startup validation).

`cow-sdk-wasm` exposes the SDK to JavaScript through typed callbacks rather
than a bundled wallet or HTTP library. It names five host callbacks:
`TypedDataSignerCallback`, `Eip1193RequestCallback`, `DigestSignerCallback`,
`CustomEip1271Callback`, and `CowFetchCallback`. The callback HTTP transport
uses SDK-owned timeout and a live `AbortSignal`, while the host runtime owns
actual network dispatch.

<!-- runtime-routing:start -->
## Choose the crate or package by runtime

| You're building... | Use | Why |
| --- | --- | --- |
| Native Rust services, bots, solvers, analytics | `cow-sdk` | Native HTTP transport, signing, trading, orderbook, and subgraph surfaces. |
| Native Rust apps using Alloy | `cow-sdk` plus `cow-sdk-alloy-*` | Opt-in Alloy provider and signer adapters without widening the default facade. |
| Rust apps that compile to browser WASM | `cow-sdk-browser-wallet` plus `cow-sdk-transport-wasm` | Rust-on-wasm wallet and fetch plumbing; not the JavaScript-callable package. |
| Standard browser dapp or CowSwap-style UI in TypeScript | Upstream [`@cowprotocol/cow-sdk`](https://www.npmjs.com/package/@cowprotocol/cow-sdk) | Substantially smaller bundle at equivalent feature subsets; mature web ecosystem fit. |
| TypeScript apps that need byte-for-byte Rust signing parity (viem, ethers, wagmi, or EIP-1193 wallets) | `<published-cow-sdk-wasm-package>` | Wallet stack-agnostic callbacks and the full facade surface. |
| Browser dapps with a smaller bundle target | `<published-cow-sdk-wasm-package>/orderbook` | Orderbook and signing subset with a smaller raw wasm budget. |
| Signer services or HSM proxies | `<published-cow-sdk-wasm-package>/signing` | Signing, UID, EIP-1271, and deployment helpers without HTTP clients. |
| Node.js 22 or 24 LTS backends | `<published-cow-sdk-wasm-package>` | Node target works without browser polyfills when transport is configured. |
| Cloudflare Workers | `<published-cow-sdk-wasm-package>/cloudflare` plus `<published-cow-sdk-wasm-package>/cloudflare/wasm` | Worker-compatible web target with explicit module initialization. Size-compatible with current Workers Free compressed-size limit at the time of measurement; full Workers support pending release-bundle and startup validation. |
| Deno | `<published-cow-sdk-wasm-package>` | Experimental build-only support; validate in your own runtime before production use. |
| Non-JS wasm consumers, WASI, WebAssembly components, TinyGo, Blazor, AssemblyScript guests, or no_std | Out of scope for 0.1.0 | Use native Rust crates where possible; the npm package targets JavaScript hosts. |
<!-- runtime-routing:end -->

`signOrderWithCustomEip1271` is the smart-account integration point when a
JavaScript application owns the account-abstraction client and the SDK should
only consume the resolved EIP-1271 signature.

## Contract Shape

The traits are intentionally narrow.

### `Signer`

An async signer owns:

- address resolution via `get_address`
- message signing via `sign_message`
- transaction signing via `sign_transaction`
- typed-data signing via `sign_typed_data` or `sign_typed_data_payload`
- transaction submission via `send_transaction`, which returns a
  `TransactionBroadcast` carrying the broadcast hash. This is not a mined
  receipt and does not prove block inclusion or execution success.
- gas estimation via `estimate_gas`

Browser-wallet support implements `Signer` directly. Native key-store
adapters such as `cow-sdk-alloy-signer` implement `Signer` against their
own private-key backend.

### `Provider`

An async provider owns:

- chain id lookup
- code lookup
- transaction-receipt lookup
- storage lookup
- generic call execution
- typed contract reads through `read_contract`
- block lookup
- typed contract-handle creation

### `SigningProvider`

An async signing provider extends `Provider` with signer creation for
providers that can construct wallet, hosted, or locally managed signers.

Read-only async providers do not implement this extension.

## Minimal Worked Example

The example below shows one small in-memory adapter pair:

- `StaticSigner`
- `StaticProvider`

It is intentionally simple.

Its job is to demonstrate the trait shape, not to model a production RPC stack.

```rust
use cow_sdk_core::{
    Address, Amount, BlockInfo, ChainId, ContractCall, ContractHandle, CoreError, HexData,
    Provider, Signer, SigningProvider, TransactionBroadcast, TransactionHash, TransactionReceipt,
    TransactionRequest, TransactionStatus, TypedDataDomain, TypedDataField,
};

#[derive(Debug, Clone)]
struct StaticSigner {
    address: Address,
    receipt_hash: TransactionHash,
    gas_limit: Amount,
}

impl Signer for StaticSigner {
    type Error = CoreError;

    async fn get_address(&self) -> Result<Address, Self::Error> {
        Ok(self.address.clone())
    }

    async fn sign_message(&self, _message: &[u8]) -> Result<String, Self::Error> {
        Ok("0xfeedface".to_owned())
    }

    async fn sign_transaction(&self, _tx: &TransactionRequest) -> Result<String, Self::Error> {
        Ok("0xdeadbeef".to_owned())
    }

    async fn sign_typed_data(
        &self,
        _domain: &TypedDataDomain,
        _fields: &[TypedDataField],
        _value_json: &str,
    ) -> Result<String, Self::Error> {
        Ok("0xtypeddata".to_owned())
    }

    async fn send_transaction(
        &self,
        _tx: &TransactionRequest,
    ) -> Result<TransactionBroadcast, Self::Error> {
        Ok(TransactionBroadcast::new(self.receipt_hash.clone()))
    }

    async fn estimate_gas(&self, _tx: &TransactionRequest) -> Result<Amount, Self::Error> {
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
    type Error = CoreError;

    async fn get_chain_id(&self) -> Result<ChainId, Self::Error> {
        Ok(self.chain_id)
    }

    async fn get_code(&self, _address: &Address) -> Result<Option<HexData>, Self::Error> {
        Ok(None)
    }

    async fn get_transaction_receipt(
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

    async fn get_storage_at(
        &self,
        _address: &Address,
        _slot: &str,
    ) -> Result<HexData, Self::Error> {
        HexData::new("0x00")
    }

    async fn call(&self, _tx: &TransactionRequest) -> Result<HexData, Self::Error> {
        HexData::new("0x")
    }

    async fn read_contract(&self, _request: &ContractCall) -> Result<String, Self::Error> {
        Ok(self.allowance_result.clone())
    }

    async fn get_block(&self, _block_tag: &str) -> Result<BlockInfo, Self::Error> {
        Ok(BlockInfo::new(1, None))
    }

    async fn get_contract(
        &self,
        address: &Address,
        abi_json: &str,
    ) -> Result<ContractHandle, Self::Error> {
        Ok(ContractHandle::new(address.clone(), abi_json.to_owned()))
    }
}

impl SigningProvider for StaticProvider {
    type Signer = StaticSigner;

    async fn create_signer(&self, _signer_hint: &str) -> Result<Self::Signer, Self::Error> {
        Ok(self.signer.clone())
    }
}
```

### What The Example Shows

The example is intentionally small, but it already satisfies the stable
integration contract:

- address resolution comes from the signer
- transaction submission comes from the signer
- transaction observation comes from the provider receipt lookup
- typed contract reads come from the provider
- signer creation is provider-owned through `SigningProvider`
- the provider keeps chain authority

## Using The Adapter With Downstream Helpers

Once your adapter implements the traits, you can pass it into downstream
helpers that are generic over `Provider`, `Signer`, or
`SigningProvider`.

For example, the trading crate exposes allowance helpers over the provider seam:

```rust
use cow_sdk_core::{Address, CowEnv, SupportedChainId};
use cow_sdk_trading::get_cow_protocol_allowance;

async fn read_allowance(
    provider: &StaticProvider,
) -> Result<(), Box<dyn std::error::Error>> {
    let token = Address::new("0xfff9976782d46cc05630d1f6ebab18b2324d6b14")?;
    let owner = Address::new("0x1111111111111111111111111111111111111111")?;

    let allowance = get_cow_protocol_allowance(
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

## Relationship To The Default Facade

The root `cow-sdk` facade stays trading-first.

It does not freeze one provider ecosystem into the default package.

That is why this guide exists as a separate page instead of widening the
onboarding path or the root crate identity.

If you are still choosing crate surfaces, read [Architecture](architecture.md).

If you want the shortest deterministic onboarding path first, read
[Getting Started](getting-started.md).
