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
bridging to alloy types is zero-cost via `From::from(addr).into()` or the
`as_alloy` / `into_alloy` accessors. Constructors validate input; the cow public surface preserves the
lowercase wire form for byte-typed identities (`Address`, `Hash32`,
`AppDataHash`, `HexData`, `OrderUid`) and the strict-decimal-only wire form
for the numeric type (`Amount`).

## Shipped Alloy Adapters

The native Alloy family is opt-in:

- `cow-sdk-alloy-provider` implements read-only `Provider`.
- `cow-sdk-alloy-signer` implements local private-key `Signer`.
- `cow-sdk-alloy` composes provider and signer support, implements
  `SigningProvider` for `Trading` helper flows, and implements `LogProvider`
  for single-call event-log fetching.

The root facade exposes matching features named `alloy-provider`,
`alloy-signer`, and `alloy`. These features are native-only and hard-fail on
`wasm32-unknown-unknown`; browser integrations use the `cow-sdk-wasm` package
together with the host app's own wallet stack (viem, wagmi, or any EIP-1193
provider) through the SDK's EIP-1193 request callback.

## Composable Deferral And COW Shed

COW Shed ships as the `cow-sdk-contracts` leaf crate behind the opt-in `cow-shed`
facade feature; the composable-order capability is deferred and recorded only by
[ADR 0048](adr/0048-composable-conditional-order-framework.md), with its
deployment addresses still resolvable through the typed `Registry`. COW Shed
rests on the same provenance and registry foundations, which improve on directly
copying TypeScript package behavior in these concrete ways:

- protocol deployment addresses (settlement, vault relayer, eth-flow,
  composable) resolve through one typed `Registry` const table; the COW Shed
  factory/implementation pairs are version-keyed module constants because each
  deployed generation is a deterministic CREATE2 deployment identical on every
  supported chain — there is no chain axis to register
- not-deployed and unsupported chains live in a coverage manifest instead of
  being mixed into addressable rows
- EIP-1271 custom signature production is owned by `cow-sdk-signing`, so trading
  workflows consume signatures without owning smart-account implementation
  details
- every upstream contract surface is bound through inline `alloy::sol!` and
  proven byte-for-byte by the call-data, EIP-712, and selector fixtures under
  `parity/fixtures/` (with the upstream Solidity pinned by commit in
  `parity/source-lock.yaml`), so hand-written ABI encoders never enter the
  workspace
- source commits and npm package integrity evidence are pinned together, with
  public generated documentation treated only as drift signal
- COW Shed proxy creation-code bytes are digest-pinned (byte length +
  keccak256, byte-identical to the TypeScript arbiter's constants) and the
  address-derivation fixtures anchor on the arbiter's own CREATE2 golden
  vectors
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
  `FetchTransport` from `cow-sdk-core`'s target-gated `transport::fetch`
  module, the browser sibling of `ReqwestTransport`. Custom implementations
  install through the builder's `.transport(Arc::new(...))` setter on
  both `OrderbookApi::builder()` and `SubgraphApi::builder()`. See
  [Transport](transport.md) for the full seam.

`cow-sdk-core` also ships narrower capability traits —
[`TypedDataSigner`] and [`DigestSigner`] per
[ADR 0045](adr/0045-async-signer-trait-narrowing.md) — for callback-shaped
adapters that expose only one signing operation. Owner resolution is served by
`Signer::address`.

## TypeScript And JavaScript Runtime Boundary

For most browser dapps, web apps, and CowSwap-style UIs, the upstream
[`@cowprotocol/cow-sdk`](https://www.npmjs.com/package/@cowprotocol/cow-sdk)
TypeScript SDK is the recommended choice; it is substantially smaller at
equivalent feature subsets. `cow-sdk-wasm` is appropriate for specialized
cases — TypeScript services that need byte-for-byte parity with the Rust
SDK's signing path, single-source-of-truth Rust + TypeScript embedding, and
Cloudflare Workers (size-compatible at the time of measurement; the
`cloudflare` flavor is built and tested end-to-end in CI (Workers Vitest),
within the Workers compressed-size budget).

`cow-sdk-wasm` exposes the SDK to JavaScript through typed callbacks rather
than a bundled wallet or HTTP library. It names five host callbacks:
`TypedDataSignerCallback`, `Eip1193RequestCallback`, `DigestSignerCallback`,
`CustomEip1271Callback`, and `CowFetchCallback`. The callback HTTP transport
uses SDK-owned timeout and a live `AbortSignal`, while the host runtime owns
actual network dispatch.

## Choose the crate or package by runtime

The canonical runtime-to-package routing table lives in the root README:
[When to use cow-rs](../README.md#when-to-use-cow-rs).

`signOrderWithCustomEip1271` is the smart-account integration point when a
JavaScript application owns the account-abstraction client and the SDK should
only consume the resolved EIP-1271 signature.

## Contract Shape

The traits are intentionally narrow.

### `Signer`

An async signer owns:

- address resolution via `address`
- message signing via `sign_message`
- typed-data signing via `sign_typed_data_payload`, which receives the
  canonical EIP-712 payload: domain, full type map, primary-type name, and
  message in one self-contained value
- transaction submission via `send_transaction`, which returns a
  `TransactionBroadcast` carrying the broadcast hash. This is not a mined
  receipt and does not prove block inclusion or execution success.
- gas estimation via `estimate_gas`

For browser integrations, the `cow-sdk-wasm` package bridges `Signer` to the
host wallet through the EIP-1193 request callback. Native key-store adapters
such as `cow-sdk-alloy-signer` implement `Signer` against their own private-key
backend.

### `Provider`

An async provider owns:

- chain id lookup
- code lookup
- transaction-receipt lookup
- generic call execution
- typed contract reads through `read_contract`
- block lookup

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
    Address, Amount, BlockInfo, ChainId, ContractCall, CoreError, HexData,
    Provider, Signer, SigningProvider, TransactionBroadcast, TransactionHash, TransactionReceipt,
    TransactionRequest, TransactionStatus, TypedDataPayload,
};

#[derive(Debug, Clone)]
struct StaticSigner {
    address: Address,
    receipt_hash: TransactionHash,
    gas_limit: Amount,
}

impl Signer for StaticSigner {
    type Error = CoreError;

    async fn address(&self) -> Result<Address, Self::Error> {
        Ok(self.address.clone())
    }

    async fn sign_message(&self, _message: &[u8]) -> Result<String, Self::Error> {
        Ok("0xfeedface".to_owned())
    }

    async fn sign_typed_data_payload(
        &self,
        _payload: &TypedDataPayload,
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

    async fn call(&self, _tx: &TransactionRequest) -> Result<HexData, Self::Error> {
        HexData::new("0x")
    }

    async fn read_contract(&self, _request: &ContractCall) -> Result<String, Self::Error> {
        Ok(self.allowance_result.clone())
    }

    async fn get_block(&self, _block_tag: &str) -> Result<BlockInfo, Self::Error> {
        Ok(BlockInfo::new(1, None))
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
use cow_sdk_trading::cow_protocol_allowance;

async fn read_allowance(
    provider: &StaticProvider,
) -> Result<(), Box<dyn std::error::Error>> {
    let token = Address::new("0xfff9976782d46cc05630d1f6ebab18b2324d6b14")?;
    let owner = Address::new("0x1111111111111111111111111111111111111111")?;

    let allowance = cow_protocol_allowance(
        provider,
        &token,
        &owner,
        SupportedChainId::Sepolia,
        CowEnv::Prod,
        None,
    )
    .await?;

    println!("allowance={allowance}");
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
