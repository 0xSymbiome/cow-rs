# ADR 0035: Ship A Read-Only Alloy Provider Adapter

- Status: Accepted
- Date: 2026-05-06
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: alloy, provider, adapter, native, dependencies
- Related: [ADR 0008](0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md), [ADR 0010](0010-runtime-neutral-async-and-transport-posture.md), [ADR 0024](0024-asyncprovider-asyncsigningprovider-capability-split.md), [ADR 0025](0025-workspace-url-redaction-convention.md), [ADR 0026](0026-alloy-major-release-absorption-plan.md), [ADR 0033](0033-minimum-viable-panic-surface.md)

## Decision

The workspace ships `cow-sdk-alloy-provider` as a native, read-only Alloy
adapter. `RpcAlloyProvider` wraps an `Arc<alloy_provider::DynProvider<Ethereum>>`
internally and exposes it through `cow_sdk_core::Provider`.

The documented public API remains SDK-owned: callers see the provider,
typestate builder, sealed transport markers, builder error, and provider error
types. Upstream Alloy and `reqwest` values stay private except for the
doc-hidden `__seam` module, which sibling `cow-rs` Alloy adapter crates may use
for shared conversion and transport-classification helpers. That seam is not a
stable consumer API.

The adapter supports native HTTP transport in this release. Wasm targets fail at
compile time and should use the browser-wallet and EIP-1193 provider path.

## Why

Native consumers repeatedly need the same conversion from Alloy RPC responses to
the `cow-sdk-core` domain types. Keeping that conversion in a first-party leaf
crate gives it shared tests, redaction review, cancellation compatibility, and a
single dependency boundary.

ADR 0024 makes the split viable: the read-only `Provider` trait has no
signer creation method. Consumers who need signing can pair this provider with
a `Signer` or use the composed Alloy client crate without forcing signer
dependencies onto read-only users.

## Must Remain True

- Public surface: documented constructors and provider methods expose SDK-owned
  types, not upstream Alloy provider or transport types. The doc-hidden seam is
  reserved for sibling adapter crates and may change without notice.
- Trait coverage: `RpcAlloyProvider` implements every `Provider` method and
  does not implement `SigningProvider` or `Signer`.
- Builder state: `RpcAlloyProviderBuilder::build` is available only after HTTP
  transport has been selected, and the URL-bearing state stores
  `Redacted<reqwest::Url>`.
- Runtime support: native HTTP is the only enabled transport. WS, IPC, pubsub,
  and local-node helper features are deferred until they have complete tests.
- Error posture: `ProviderError` is non-exhaustive, classifies validation,
  transport, remote, cancelled, and internal failures, and keeps transport
  details redacted.
- Opt-in retry: `RpcAlloyProviderBuilder` (and the umbrella `AlloyClientBuilder`)
  accept an SDK-owned `RetryConfig` through a `retry` setter; when set, the
  JSON-RPC client is wrapped in alloy's bounded exponential-backoff layer (off by
  default, so the runtime-neutral posture and the upstream no-retry default hold).
  The REST `TransportPolicy` (ADR 0041) is not reused here — its retry signal is
  keyed on REST status codes, which JSON-RPC-over-HTTP errors do not surface.
- Validation: contract tests cover all provider methods, `read_contract` parity,
  malformed-input failures, redaction, cancellation, dependency boundaries, and
  compile-fail capability exclusions.

## Alternatives Rejected

- Re-export upstream Alloy provider types directly: this would couple the SDK
  surface to Alloy's provider semver.
- Keep only a documentation guide: that leaves every consumer to reimplement and
  retest the same conversion logic.
- Combine provider and signer in one crate: read-only users would pull signer
  dependencies and the capability split from ADR 0024 would be weakened.
- Declare placeholder WS or IPC features: compiling a feature that later fails
  through an unsupported runtime path is less honest than omitting the feature.

## Stability

The `cow_sdk_alloy_provider::__seam` module is a doc-hidden public inter-crate
seam for sibling `cow-rs` adapter crates. It is not a semver-stable consumer
API. Anything inside the seam may change in any minor release without notice.
Consumers who write code against it do so at their own risk; the documented
consumer surface is limited to `RpcAlloyProvider`,
`RpcAlloyProviderBuilder`, `ProviderError`, and the typestate markers
explicitly exported from `lib.rs`.

The same posture applies to `ProviderError::from_alloy_transport`. It is
gated `#[doc(hidden)]` and documented in source as an inter-crate seam
constructor, not as a stable consumer API.

## Links

- [Architecture](../architecture.md)
- [Provider adapters](../providers/README.md)
- [Adapting alloy providers](../providers/adapting-alloy.md)
- [Transport](../transport.md)

**Proven by:**

- [Alloy Provider Adapter Audit](../audit/alloy-provider-adapter-audit.md)
