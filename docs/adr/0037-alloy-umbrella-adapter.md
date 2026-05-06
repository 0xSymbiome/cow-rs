# ADR 0037: Compose Native Alloy Provider And Signer In One Client

- Status: Accepted
- Date: 2026-05-06
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: alloy, provider, signer, adapter, native
- Related: [ADR 0008](0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md), [ADR 0024](0024-asyncprovider-asyncsigningprovider-capability-split.md), [ADR 0025](0025-workspace-url-redaction-convention.md), [ADR 0026](0026-alloy-major-release-absorption-plan.md), [ADR 0035](0035-alloy-provider-adapter.md), [ADR 0036](0036-alloy-signer-adapter.md)

## Decision

The workspace ships `cow-sdk-alloy` as the native composed Alloy adapter.
`AlloyClient` owns an Alloy HTTP provider configured with a local wallet
filler and implements both `cow_sdk_core::AsyncProvider` and
`cow_sdk_core::AsyncSigningProvider`.

`create_signer` returns an owned `AlloyClientSignerHandle`. The handle keeps an
`Arc` to the client inner state, so it remains usable after the parent client
value is dropped and does not borrow from the client. The handle implements
`AsyncSigner`, preserves canonical EIP-712 payload primary types, normalizes
ECDSA recovery bytes through `cow-sdk-contracts`, submits transactions through
the wallet-filler provider, and returns the broadcast hash from
`pending.watch().await`.

Raw `sign_transaction` is intentionally unsupported on the umbrella handle. It
returns `UnsupportedTransactionRequest` without sending an HTTP request; callers
that want on-chain execution use `send_transaction`, where nonce, fee, chain,
and broadcast context are available.

The adapter is native-only. Wasm targets fail at compile time and should use
browser-wallet signing plus consumer-supplied EIP-1193 provider reads.

## Why

Native consumers often want one ergonomic client that can both read chain state
and submit signed transactions. The provider and signer leaves keep capability
boundaries available for users that need only one side; the umbrella composes
them for the common native wallet-provider case without changing the default
facade dependency graph.

The owned handle shape follows the `AsyncSigningProvider` trait contract, which
has no lifetime parameter. Returning a borrowed signer would either fail to
compile or expose a fragile lifetime model to downstream users.

`pending.watch().await` honestly returns the broadcast transaction hash, which
is the only value carried by the SDK's minimal `TransactionReceipt`. Waiting for
and then discarding a full receipt would imply inclusion semantics the public
type does not represent.

## Must Remain True

- Public surface: `AlloyClient`, `AlloyClientBuilder`,
  `AlloyClientSignerHandle`, and `AlloyClientError` expose SDK-owned types; the
  upstream Alloy provider, local signer, wallet, transport, and configured URL
  remain private and redacted.
- Builder state: construction requires HTTP transport, private-key source, and
  chain id before `build()` is callable; external callers cannot construct the
  marker states directly.
- Trait coverage: `AlloyClient` implements `AsyncProvider` and
  `AsyncSigningProvider`; `AlloyClientSignerHandle` implements `AsyncSigner`
  and does not implement `AsyncProvider` or sync `Signer`.
- Runtime behavior: `send_transaction` uses the Alloy wallet-filler provider and
  `pending.watch().await`; `sign_transaction` returns
  `UnsupportedTransactionRequest` without dispatching HTTP; `estimate_gas`
  delegates directly to the provider.
- Typed-data behavior: `sign_typed_data_payload` preserves the payload primary
  type rather than routing through the legacy flat-fields fallback.
- Support posture: native targets are supported; wasm targets fail closed with
  the documented compile-time diagnostic.
- Validation: tests cover provider delegation, owned signer handles, EIP-712
  vectors, chain coherence, redaction, cancellation, no-broadcast
  `sign_transaction`, compile-fail capability exclusions, examples, and
  TradingSdk integration.

## Alternatives Rejected

- Re-export upstream Alloy provider and signer types directly: this would bind
  the SDK public API to Alloy semver and error shapes.
- Make the provider leaf signer-capable: read-only users would pull local-key
  dependencies and weaken the capability split.
- Route umbrella typed-data signing through the signer leaf at runtime: the
  umbrella already owns the local signer and should preserve the primary type
  directly instead of introducing an adapter-to-adapter call chain.
- Use `pending.get_receipt().await`: the SDK receipt type carries only the
  transaction hash, so receipt waiting would misrepresent the public semantic.

## Links

- [Alloy Provider Adapter ADR](0035-alloy-provider-adapter.md)
- [Alloy Signer Adapter ADR](0036-alloy-signer-adapter.md)
- [Async Provider Capability Split ADR](0024-asyncprovider-asyncsigningprovider-capability-split.md)
- [Architecture](../architecture.md)

**Proven by:**

- [Alloy Umbrella Adapter Audit](../audit/alloy-umbrella-adapter-audit.md)
