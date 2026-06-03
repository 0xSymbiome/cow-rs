# ADR 0037: Compose Native Alloy Provider And Signer In One Client

- Status: Accepted
- Date: 2026-05-06
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: alloy, provider, signer, adapter, native
- Related: [ADR 0008](0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md), [ADR 0024](0024-asyncprovider-asyncsigningprovider-capability-split.md), [ADR 0025](0025-workspace-url-redaction-convention.md), [ADR 0026](0026-alloy-major-release-absorption-plan.md), [ADR 0035](0035-alloy-provider-adapter.md), [ADR 0036](0036-alloy-signer-adapter.md), [ADR 0038](0038-transaction-lifecycle-types.md)

## Decision

The workspace ships `cow-sdk-alloy` as the native composed Alloy adapter.
`AlloyClient` owns an Alloy HTTP provider configured with a local wallet
filler and implements `cow_sdk_core::Provider`,
`cow_sdk_core::LogProvider`, and `cow_sdk_core::SigningProvider`.
`LogProvider::get_logs` issues a single bounded `eth_getLogs` over the composed
provider and reuses the provider leaf's `LogQuery` → filter and Alloy-log →
`RawLog` conversions through the doc-hidden inter-crate seam, so the umbrella
does not fork the reviewed mappings. A consumer therefore fetches event logs
from the same client it trades through, without constructing a second provider
for the same RPC endpoint.

`create_signer` returns an owned `AlloyClientSignerHandle`. The handle keeps an
`Arc` to the client inner state, so it remains usable after the parent client
value is dropped and does not borrow from the client. The handle implements
`Signer`, preserves canonical EIP-712 payload primary types, normalizes
ECDSA recovery bytes through `cow-sdk-contracts`, submits transactions through
the wallet-filler provider, and returns `TransactionBroadcast` with the
broadcast hash read through `*pending.tx_hash()` without waiting for
confirmation.

Raw `sign_transaction` is intentionally unsupported on the umbrella handle. It
returns `UnsupportedTransactionRequest` without sending an HTTP request; callers
that want on-chain execution use `send_transaction`, where nonce, fee, chain,
and broadcast context are available.

The adapter is native-only. Wasm targets fail at compile time and should use
browser-wallet signing plus consumer-supplied EIP-1193 provider reads.

## Why

Native consumers often want a single client that exposes both `Provider`
and `SigningProvider` so trading flows can drive the same client for chain
reads, EIP-712 typed-data signing, and transaction submission. The provider and
signer leaves keep capability boundaries available for users that need only
one side; the umbrella composes them for the common native wallet-provider case
without changing the default facade dependency graph.

The owned handle shape follows the `SigningProvider` trait contract, which
has no lifetime parameter. Returning a borrowed signer would either fail to
compile or expose a fragile lifetime model to downstream users.

`pending.tx_hash()` reads the broadcast hash already captured by Alloy's
pending-transaction builder. Waiting through `pending.watch().await` would
observe confirmation before returning from a method whose contract is only
broadcast acknowledgement. ADR 0038 separates that acknowledgement from mined
receipt observation.

Provider-side receipt conversion uses
`receipt.inner.status_or_post_state().as_eip658()` for status mapping. That
preserves `None` for pre-Byzantium post-state receipts instead of coercing them
to success through Alloy's higher-level `status()` helper.

## Must Remain True

- Public surface: `AlloyClient`, `AlloyClientBuilder`,
  `AlloyClientSignerHandle`, and `AlloyClientError` expose SDK-owned types; the
  upstream Alloy provider, local signer, wallet, transport, and configured URL
  remain private and redacted.
- Builder state: construction requires HTTP transport, private-key source, and
  chain id before `build()` is callable; external callers cannot construct the
  marker states directly.
- Trait coverage: `AlloyClient` implements `Provider`, `LogProvider`, and
  `SigningProvider`; `AlloyClientSignerHandle` implements `Signer`
  and does not implement `Provider`.
- Runtime behavior: `send_transaction` uses the Alloy wallet-filler provider,
  reads the broadcast hash through `*pending.tx_hash()`, and returns
  `TransactionBroadcast`; `sign_transaction` returns
  `UnsupportedTransactionRequest` without dispatching HTTP; `estimate_gas`
  delegates directly to the provider. `get_transaction_receipt` delegates to
  the provider crate's rich receipt conversion.
- Typed-data behavior: `sign_typed_data_payload` preserves the payload primary
  type rather than routing through the legacy flat-fields fallback.
- Support posture: native targets are supported; wasm targets fail closed with
  the documented compile-time diagnostic.
- Validation: tests cover provider delegation, owned signer handles, EIP-712
  vectors, chain-coherence verification through `build_checked()` or
  `verify_chain_id().await`, redaction, cancellation, no-broadcast
  `sign_transaction`, compile-fail capability exclusions, examples, and
  Trading integration.

## Alternatives Rejected

- Re-export upstream Alloy provider and signer types directly: this would bind
  the SDK public API to Alloy semver and error shapes.
- Make the provider leaf signer-capable: read-only users would pull local-key
  dependencies and weaken the capability split.
- Route umbrella typed-data signing through the signer leaf at runtime: the
  umbrella already owns the local signer and should preserve the primary type
  directly instead of introducing an adapter-to-adapter call chain.
- Use `pending.get_receipt().await`: transaction submission should return only
  a broadcast acknowledgement; receipt waiting belongs to provider lookup or a
  higher-level wait helper.

## Chain Coherence

`AlloyClientBuilder::chain_id(SupportedChainId)` binds the local signer to the
configured chain id. The default `build()` path is free of network I/O; the
configured chain id is not verified against the RPC endpoint at construction
time.

Trading flows that require the configured chain id to match the RPC endpoint
should use `build_checked().await`, which dispatches one `eth_chainId` call and
rejects mismatch with `AlloyClientBuilderError::ChainMismatch`, or call
`AlloyClient::verify_chain_id().await?` after construction. Keeping this check
opt-in avoids surprising long-running clients that re-verify on their own
cadence while making the chain-coherence guarantee available where it matters.

## Stability

The public
`AlloyClientError::{from_alloy_transport, from_alloy_signer, from_pending_tx_error}`
constructors are gated `#[doc(hidden)]` and documented in source as
inter-crate seam constructors. They exist so sibling adapter crates can lift
Alloy error types into the umbrella's typed error surface. They are not
semver-stable consumer API.

The documented consumer surface is limited to `AlloyClient`,
`AlloyClientBuilder`, `AlloyClientSignerHandle`, `AlloyClientError`, the
typestate markers explicitly exported from `lib.rs`, and the namespaced
provider and signer re-exports of the leaf adapter public surfaces.

The umbrella consumes the read-contract and typed-data conversion modules
from the leaf adapters through their `#[doc(hidden)] __seam` entries. The
provider leaf owns the `execute_read_contract` entry point and the
JSON-RPC request, block-tag, receipt, block-info, and log-query/raw-log
conversions; the signer leaf owns the EIP-712 typed-data conversion and
signature normalization. The workspace `alloy_read_contract_parity_invariant`
integration test continues to assert byte-for-byte equality between the
umbrella's `AlloyClient::read_contract` output and the leaf provider's
`RpcAlloyProvider::read_contract` output for pinned ABI fixtures, even
though both call sites now converge on the same function body — the test
remains a regression pin against any future re-fork.

## Links

- [Alloy Provider Adapter ADR](0035-alloy-provider-adapter.md)
- [Alloy Signer Adapter ADR](0036-alloy-signer-adapter.md)
- [Transaction Lifecycle Types ADR](0038-transaction-lifecycle-types.md)
- [Async Provider Capability Split ADR](0024-asyncprovider-asyncsigningprovider-capability-split.md)
- [Architecture](../architecture.md)

**Proven by:**

- [Alloy Umbrella Adapter Audit](../audit/alloy-umbrella-adapter-audit.md)
