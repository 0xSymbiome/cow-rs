# ADR 0035: Alloy Adapter Family (Provider, Signer, Umbrella)

- Status: Accepted (amended)
- Date: 2026-05-06 (consolidated 2026-06-15)
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: alloy, provider, signer, adapter, native, eip712
- Related: [ADR 0008](0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md), [ADR 0010](0010-runtime-neutral-async-and-transport-posture.md), [ADR 0022](0022-ecdsa-signature-v-normalization.md), [ADR 0024](0024-asyncprovider-asyncsigningprovider-capability-split.md), [ADR 0025](0025-workspace-url-redaction-convention.md), [ADR 0026](0026-alloy-major-release-absorption-plan.md), [ADR 0038](0038-transaction-lifecycle-types.md), [ADR 0068](0068-payload-only-typed-data-signing.md)
- Consolidates: [ADR 0036](0036-alloy-signer-adapter.md), [ADR 0037](0037-alloy-umbrella-adapter.md)

## Decision

The workspace ships the native Alloy runtime as a three-crate family, each gated to
native targets (wasm fails closed at compile time, where the `cow-sdk-wasm`
EIP-1193 callback path applies instead):

- **`cow-sdk-alloy-provider`** — read-only RPC. `RpcAlloyProvider` wraps an
  `Arc<alloy_provider::DynProvider<Ethereum>>` and implements
  `cow_sdk_core::Provider` and `LogProvider`. It implements neither
  `SigningProvider` nor `Signer`.
- **`cow-sdk-alloy-signer`** — local keystore signer. `LocalAlloySigner` wraps an
  Alloy private-key signer and implements `cow_sdk_core::Signer` (EIP-191 plus
  payload-only EIP-712, ADR 0068). It implements neither `Provider` nor
  `SigningProvider`; provider-backed transaction methods return `ProviderRequired`.
- **`cow-sdk-alloy`** — composed umbrella. `AlloyClient` owns a wallet-filler
  provider plus a local signer and implements `Provider`, `LogProvider`, and
  `SigningProvider`; `create_signer` returns an owned `AlloyClientSignerHandle`
  (`Signer`).

Three crates, not one: the capability split (ADR 0024) keeps the read leaf free of
the local-keystore signer, and only a crate boundary enforces that hard — a feature
gate unifies graph-globally, leaking the keystore into a read-only consumer under a
mixed-workspace build. The provider and signer leaves serve consumers who need one
side; the umbrella composes them for the native wallet-provider case without
enlarging the default facade dependency graph.

### Shared posture (all three crates)

- **SDK-owned surface.** Documented APIs expose `cow-sdk-core` domain types plus
  per-crate error, builder, and typestate types — never upstream Alloy provider,
  signer, transport, or `reqwest` values.
- **`__seam`.** Each crate exposes a `#[doc(hidden)] pub mod __seam` so sibling
  adapter crates can share conversion, transport-classification, and key-parsing
  helpers from a single source. It is not a semver-stable consumer API and may
  change in any minor release; the same posture covers the `from_alloy_*` error
  constructors.
- **Native-only.** Wasm targets fail closed with a documented compile-time
  diagnostic.
- **Redaction.** Transport URLs are held in `Redacted<…>` and never reach `Debug`,
  `Display`, or serde output (ADR 0025).
- **Error posture.** Each error enum is `#[non_exhaustive]`, classifies
  validation / transport / remote / cancelled / internal failures, and keeps
  transport details redacted.

## Why

Native consumers repeatedly need the same Alloy-to-`cow-sdk-core` conversions for
reads, the same local-key-to-`Signer` bridge for signing, and often a single client
that does both (chain reads + EIP-712 typed-data signing + transaction submission)
for trading flows. First-party leaf crates give those conversions shared tests,
redaction review, and cancellation compatibility behind a single dependency
boundary. ADR 0024's `Provider` / `SigningProvider` split makes the read leaf viable
without forcing signer dependencies onto read-only users; the shared ADR 0022
normalizer keeps recovery-byte handling single-sourced across the signer and
umbrella.

## Must Remain True

**Provider leaf**

- `RpcAlloyProvider` implements every `Provider` method and does not implement
  `SigningProvider` or `Signer`.
- `RpcAlloyProviderBuilder::build` is available only after HTTP transport is
  selected; the URL-bearing state stores `Redacted<reqwest::Url>`.
- Native HTTP is the only enabled transport; WS, IPC, pubsub, and local-node
  helpers are deferred until they have complete tests.
- Opt-in retry: the builder (and `AlloyClientBuilder`) accept an SDK-owned
  `RetryConfig`; when set, the JSON-RPC client is wrapped in alloy's bounded
  exponential-backoff layer (off by default, preserving the runtime-neutral
  no-retry default). The REST `TransportPolicy` (ADR 0041) is not reused — its
  retry signal is keyed on REST status codes, which JSON-RPC-over-HTTP does not
  surface.

**Signer leaf**

- `LocalAlloySigner` implements `Signer` and does not implement `Provider` or
  `SigningProvider`.
- `build()` is available only after both a private key and a chain id are selected;
  builder markers stay sealed from external construction.
- Message and typed-data signatures normalize through `cow-sdk-contracts`
  (ADR 0022); canonical typed-data payload signing preserves the caller's primary
  type (payload-only, ADR 0068).
- Native local-keystore signing is the only supported runtime; wasm fails closed.

**Umbrella**

- `AlloyClient` implements `Provider`, `LogProvider`, and `SigningProvider`;
  `AlloyClientSignerHandle` implements `Signer` and does not implement `Provider`.
- Construction requires HTTP transport, a private-key source, and a chain id before
  `build()`; external callers cannot construct the marker states directly.
- `send_transaction` uses the wallet-filler provider and returns
  `TransactionBroadcast` with the broadcast hash read through `*pending.tx_hash()`
  without waiting for confirmation (ADR 0038); the read methods, receipt conversion,
  and `LogProvider::get_logs` reuse the provider leaf's reviewed conversions through
  `__seam` so the umbrella never forks them.
- `sign_typed_data_payload` preserves the payload primary type; the owned handle
  keeps an `Arc` to the client inner state, remaining usable after the parent client
  value drops.

## Chain coherence (umbrella)

`AlloyClientBuilder::chain_id(SupportedChainId)` binds the signer to the chain. The
default `build()` is free of network I/O. Flows that require the configured chain to
match the endpoint use `build_checked().await` (one `eth_chainId`, rejecting
mismatch with `AlloyClientBuilderError::ChainMismatch`) or
`AlloyClient::verify_chain_id().await?`. Keeping the check opt-in avoids surprising
long-running clients that re-verify on their own cadence.

## Alternatives Rejected

- Re-export upstream Alloy provider or signer types directly — couples the SDK
  surface to Alloy's semver and error shapes.
- Combine provider and signer in one crate, or fold into `core` — read-only users
  would pull the local-keystore signer; the ADR 0024 split, and the only hard
  isolation, is the crate boundary.
- Keep documentation-only guides — every consumer reimplements and retests the same
  conversions.
- Declare placeholder WS/IPC features or return placeholder transaction signatures —
  compiling an unsupported path, or signing without provider context, is less honest
  than omitting it.
- Use `pending.get_receipt().await` for submission — submission returns a broadcast
  acknowledgement; receipt waiting belongs to provider lookup (ADR 0038).

## Stability

The `__seam` modules and the `from_alloy_transport` / `from_alloy_signer` /
`from_pending_tx_error` constructors are `#[doc(hidden)]` inter-crate seams, not
semver-stable consumer API; anything inside them may change in any minor release.
The documented consumer surface is each crate's public client, builder, error, and
typestate markers (and the umbrella's namespaced `provider` / `signer` re-exports of
the leaf surfaces). The workspace `alloy_read_contract_parity_invariant` test pins
that the umbrella and provider leaf produce byte-identical `read_contract` output.

## Links

- [Architecture](../architecture.md)
- [Provider adapters](../providers/README.md)
- [Adapting alloy providers](../providers/adapting-alloy.md)
- [Transport](../transport.md)

**Proven by:**

- [Alloy Provider Adapter Audit](../audit/alloy-provider-adapter-audit.md)
- [Alloy Signer Adapter Audit](../audit/alloy-signer-adapter-audit.md)
- [Alloy Umbrella Adapter Audit](../audit/alloy-umbrella-adapter-audit.md)
