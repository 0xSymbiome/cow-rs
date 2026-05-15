# Parity Scope

This document defines the review scope for parity evidence in `cow-rs`.

The source lock is the portable authority for upstream producer commits and
paths.

## Source Lock

Pinned sources live in `parity/source-lock.yaml`.

| Producer | Pinned role | Used for |
| --- | --- | --- |
| `cowprotocol/cow-sdk` | Primary | SDK package configuration, COW Shed TypeScript constants, and shared package-level deployment evidence |
| `cowprotocol/composable-cow` | Primary capability evidence | Composable-order Solidity excerpts, deployment rows, selector fixtures, EIP-1271 payload shapes, and watch-tower boundary evidence |
| `cowdao-grants/cow-shed` | Primary capability evidence | COW Shed Solidity excerpts, proxy creation-code bytes, factory address derivation, hook signature shape, and version-call evidence |
| `cowprotocol/watch-tower` | Reference-only boundary evidence | Off-chain orchestration behavior used to define what remains outside the SDK |

Local upstream checkout paths are optional validation inputs. When they are
used, they must be independent git checkouts or worktrees at the pinned
commits.

Local development snapshots are reference-only and are not commit provenance.
Release validation uses fresh git checkouts at the source-lock-pinned commits,
produced by the `parity-maintainer provision-upstreams` flow whenever
provenance is required.

## Workflow-Defined 0.1.0 Scope

Parity for `cow-rs` 0.1.0 is defined by supported workflows, not by a
percentage of upstream TypeScript methods. The release supports these workflow
buckets:

1. **Deterministic order primitives**: order UID calculation, EIP-712 typed
   data envelopes, and EIP-1271 signature payload generation from wrapped ECDSA
   signatures.
2. **Order signing flows**: typed-data EIP-712 signing, raw EIP-1193 signing,
   EIP-191 digest signing, EIP-1271 wrapping, custom EIP-1271 signatures, and
   cancellation typed data.
3. **Orderbook operations**: quote, signed order submission, raw order-creation
   submission, order lookup, owner order pagination, trade lookup, native price
   lookup, app-data lookup, app-data upload, and signed cancellation
   submission.
4. **Trading orchestration**: quote, quote-sign-post, quote-result reuse, limit
   order posting, native-sell transaction construction, allowance reads, and
   EIP-1271-backed swap posting.
5. **Subgraph reads**: protocol totals, recent daily and hourly volume, and
   arbitrary GraphQL query execution.
6. **App-data tools**: app-data document generation, CID and hash derivation,
   schema validation, CID-to-hex conversion, and hex-to-CID conversion.
7. **IPFS app-data fetch**: fetch by CID and fetch by app-data hash through an
   injected HTTP transport.
8. **Deployment registry**: chain and environment addresses for GPv2,
   composable-order, and COW Shed contract families, with deployment coverage
   records for not-deployed and unsupported chain evidence.
9. **Runtime support**: browser bundlers, Node.js 22 and 24 LTS, Cloudflare
   Workers, and experimental Deno builds.
10. **Cancellation and timeouts**: per-call `signal`, per-call `timeoutMs`, and
    wallet callback `walletConfig.timeoutMs`.

The 0.1.0 scope does not claim total method-for-method parity with the
upstream TypeScript SDK. Composable conditional-order helpers and the COW
Shed account-abstraction proxy ship as first-release readiness: reserved
leaf manifests, deployment evidence, ABI excerpts, parity fixtures, and
governing ADRs are in scope, with full ergonomic helper bodies arriving in
the additive landings that follow. Capability families that are explicitly
deferred for 0.1.0 (cross-chain bridging order construction, hook-trampoline
bytecode chaining, ecosystem provider adapters outside Alloy, and other
items listed under Out-Of-Scope below) should continue to use the upstream
packages until their dedicated `cow-rs` leaf crates land.

## Surface Boundaries

| Surface | Rust crate | Pinned evidence |
| --- | --- | --- |
| Core config and runtime contracts | `cow-sdk-core` | Common adapter, address, token, config, and selected shared type sources from `cowprotocol/cow-sdk` |
| Contracts | `cow-sdk-contracts` | `cowprotocol/contracts` Solidity sources, committed excerpts under `crates/contracts/abi/**/*.sol`, `alloy::sol!`-generated bindings, the typed `Registry` deployment authority, and selected upstream test fixtures |
| Signing | `cow-sdk-signing` | Order-signing utilities, typed-data helpers, and contract-signing sources |
| App-data | `cow-sdk-app-data` | App-data helpers, schema imports, generated schema references, and schema regression tests |
| Orderbook | `cow-sdk-orderbook` | TypeScript orderbook sources plus selected `cowprotocol/services` OpenAPI and validation references |
| Trading | `cow-sdk-trading` | TypeScript trading workflows and tests |
| Subgraph | `cow-sdk-subgraph` | TypeScript subgraph API, GraphQL, query, and test sources |
| SDK facade | `cow-sdk` | TypeScript SDK root package exports and typedoc entrypoint |
| HTTP transport policy | `cow-sdk-transport-policy` | Retry, rate-limit, cooldown, jitter, and transport-classification behavior shared by typed HTTP clients |
| Native Alloy adapters | `cow-sdk-alloy-provider`, `cow-sdk-alloy-signer`, `cow-sdk-alloy` | Alloy runtime and Alloy Core source-lock pins, adapter contract tests, transaction broadcast / receipt shape invariants, and native examples |
| TypeScript-callable WASM | `cow-sdk-wasm` | Native Rust helper parity for typed-data, UID, digest, app-data, EIP-1271 payloads, orderbook/subgraph/IPFS/trading DTO shape, npm declaration snapshots, and upstream TypeScript SDK EIP-1271 vector coverage |
| Composable orders | `cow-sdk-composable` reserved manifest | Composable-CoW source locks, Solidity excerpts, selector and EIP-1271 blob fixtures, handler revert fixtures, and watch-tower boundary documentation |
| COW Shed | `cow-sdk-cow-shed` reserved manifest | COW Shed source locks, Solidity excerpts, proxy creation-code bytes, CREATE2 address fixtures, EIP-712 hook fixtures, and version-call evidence |

## Schema Evidence Policy

Schema-derived evidence is a review aid, not a public API shortcut.

- orderbook schema evidence is tied to `cowprotocol/services`, including
  `crates/orderbook/openapi.yml`
- subgraph evidence is tied to `cowprotocol/cow-sdk`, including
  `packages/subgraph/src/queries.ts`
- canonical subgraph query documents live in
  `crates/subgraph/src/query_documents/`
- test-only subgraph schema and codegen evidence lives in
  `crates/subgraph/tests/schema_evidence/`
- generated or schema-derived Rust mirrors must stay non-public or test-only

## Schema-Derived Artifacts

No generated or schema-derived Rust mirrors are part of the public SDK API.

- orderbook evidence is committed as OpenAPI artifacts, fixtures, contract
  tests, and source-lock references
- subgraph evidence is committed as saved query documents, test-only schema
  snapshots, contract tests, and source-lock references

## First-Release Scope

The Rust SDK ships in scope:

- core domain types and runtime traits (`cow-sdk-core`)
- `alloy::sol!`-generated contract bindings and Registry
  (`cow-sdk-contracts`)
- order signing and EIP-1271 verification (`cow-sdk-signing`)
- app-data encoding and schema (`cow-sdk-app-data`)
- typed orderbook transport (`cow-sdk-orderbook`)
  - `Order` covers the orderbook OpenAPI `Order` schema
    (`OrderCreation` + `OrderMetaData` + `interactions`)
  - `AuctionOrder` covers the orderbook OpenAPI `AuctionOrder` schema as a
    separate Rust type
  - `OrderQuoteResponse`, `Trade`, `StoredOrderQuote`, and
    `OnchainOrderData` cover their OpenAPI schemas as separate typed mirrors
- typed subgraph transport (`cow-sdk-subgraph`)
- quote-to-order trading workflows (`cow-sdk-trading`)
- browser-runtime wallet integration (`cow-sdk-browser-wallet`)
- browser-target HTTP transport (`cow-sdk-transport-wasm`)
- opt-in native Alloy provider, signer, and composed provider-plus-signer
  adapters (`cow-sdk-alloy-provider`, `cow-sdk-alloy-signer`,
  `cow-sdk-alloy`)
- TypeScript-callable wasm-bindgen bindings (`cow-sdk-wasm`) with typed
  JavaScript callbacks for wallet, signer, EIP-1271, and HTTP dispatch
- composable-order and COW Shed readiness evidence, including reserved crate
  manifests, contract excerpts, deployment taxonomy rows, fixture artifacts, and
  audit records

Native Alloy transaction parity is scoped to the SDK trait contract, not to
re-exporting Alloy's full transaction surface. The composed signer returns
`TransactionBroadcast` from the hash Alloy has already accepted for broadcast,
and provider receipt lookup populates `TransactionReceipt` fields that the SDK
models: status, block number, block hash, gas used, sender, and recipient.

The first release does **not** ship every helper crate body below. Reserved
manifests, provenance, and compatibility fixtures are in scope where listed,
while full ergonomic helper APIs remain additive under ADR 0008.

### Bridging

Cross-chain order construction equivalent to the upstream TypeScript
`bridging` capability. Deferred; not in scope for the first release. A future
leaf crate `cow-sdk-bridging` is a candidate when the upstream contract and
API surface stabilises.

### Composable orders

Composable-CoW readiness is in scope through the reserved
`cow-sdk-composable` manifest, deployment evidence, ABI excerpts, selector
fixtures, handler fixtures, and EIP-1271 signature blob fixtures. Full
order-construction helpers remain additive.

### Cow-shed

COW Shed readiness is in scope through the reserved `cow-sdk-cow-shed`
manifest, deployment evidence, proxy creation-code hash validation,
CREATE2 address fixtures, hook digest fixtures, and version-call evidence.
Full delegated proxy account helpers remain additive.

### Flash loans

The flashloan metadata sub-field is supported in `cow-sdk-app-data`. A
flashloan helper utility surface is deferred; not in scope for the first
release.

### Weiroll

Hook-trampoline bytecode chaining. Deferred; not in scope for the first
release.

### Additional provider ecosystems

Additional provider ecosystems beyond the native Alloy adapter and the
browser-wallet leaf are not in scope for the first release. Consumers can
implement the SDK's `AsyncProvider`, `AsyncSigningProvider`, and `AsyncSigner`
trait seams to bridge a custom ecosystem.

### TypeScript-tooling-only packages

The upstream TypeScript SDK includes packages that exist to manage TypeScript
build orchestration (for example `typescript-config`, `config`). These have no
Rust analogue and are not in scope.

## Intentionally Out-of-Scope

Parity in `cow-rs` is byte-identity on implemented surfaces, not
feature-identity with the TypeScript SDK. The following upstream surfaces
are intentionally excluded from the Rust SDK because they carry no
pre-release user value, re-introduce known protocol footguns, or have
been superseded by a clearer typed boundary. Every exclusion below is
enforced in code (via a negative test or by removing the surface
entirely) so future contributors cannot quietly reintroduce the upstream
shape on the assumption that a missing positive fixture implies a gap.

See [ADR 0011](./adr/0011-typed-amount-boundary-and-typestate-ready-state-construction.md)
for the canonical typed-amount decision. The governing parity-scope
discipline is the four-layer defense documented in the architecture
record: a negative test that fails closed, a scope-doc entry that names
the exclusion, a cross-link to the owning ADR, and a risk-register
entry for anyone who later considers reintroducing the surface.

- **CIDv0 (`Qm...`) encoding and decoding** — the cow-protocol services
  backend enforces CIDv1 with the raw multicodec (`0x55`) over a
  keccak-256 multihash (`0x1b`) as the only supported CID shape; legacy
  CIDv0 (dag-pb + sha2-256) paths carry no pre-release user value.
  Consumers that need to resolve historical `Qm`-prefixed values use a
  general-purpose `cid` crate directly. Negative test:
  `crates/app-data/tests/v0_cid_is_out_of_scope.rs`.
- **Order-level `fee_amount` as a public builder setter or DTO field**
  — the cow-protocol services backend rejects orders that carry a
  non-zero order-level fee, so the submission path always wires
  `"feeAmount": "0"` and there is no reason to let a caller construct
  a non-zero value locally. The internal serializer preserves
  `"feeAmount": "0"` for EIP-712 struct-hash compatibility. Negative
  test:
  `crates/orderbook/tests/fee_amount_is_not_a_public_builder_setter.rs`.
- **Legacy quote-response fee descriptors `executedFeeAmount` and
  `fullFeeAmount`** — the current services schema surfaces executed
  fees through the canonical `executedFee` component and quote-response
  protocol fees through `protocolFeeBps`. The retired descriptors are
  not re-emitted on the cow-rs wire. Covered by the same order-response
  wire-shape regressions in
  `crates/orderbook/tests/fee_amount_is_not_a_public_builder_setter.rs`.
- **Legacy wire-string `Amount` wrapper** — the Rust SDK consolidated
  the canonical atomic amount to a single typed newtype
  `cow_sdk_core::Amount(BigUint)` with custom serde that preserves the
  decimal-string wire format. The retired wire-string wrapper is simply
  absent from the workspace; by design, there is no negative test
  because the type does not exist and the Rust compiler itself enforces
  the exclusion at every call site. Governed by
  [ADR 0011](./adr/0011-typed-amount-boundary-and-typestate-ready-state-construction.md).
- **`TypedOrder` alias on `cow-sdk-signing`** — the canonical
  pre-signature order state is `cow_sdk_core::UnsignedOrder`; the
  former `TypedOrder = UnsignedOrder` backward-compatibility alias is
  absent from the workspace. As with the retired wire-string `Amount`
  wrapper, there is no negative test because the type does not exist
  and the Rust compiler itself enforces the exclusion at every call
  site.
- **Legacy free-function constructors on `OrderBookApi` and
  `SubgraphApi`** — the shipped construction seam for both clients is
  the typestate builder (`OrderBookApi::builder()` and
  `SubgraphApi::builder()`, governed by
  [ADR 0013](./adr/0013-http-transport-injection-and-typestate-builders.md)).
  The earlier family of free-function constructors (for example
  `from_shared_client`, `new_with_transport_policy`, `new_with_base_url`
  on the orderbook client and the matching set on the subgraph client)
  is absent from the workspace; the Rust compiler itself enforces the
  exclusion at every call site, and the `trybuild` UI harness at
  `crates/subgraph/tests/ui/builder_wasm32_missing_transport.rs`
  captures the compile error a browser consumer sees when `.build()`
  is attempted without `.transport(...)`.
- **Hand-rolled ABI encoders in `cow-sdk-contracts`** — every binding
  shipped by the contracts crate is generated through `alloy::sol!` from
  the Solidity excerpts committed under `crates/contracts/abi/`
  (governed by
  [ADR 0012](./adr/0012-alloy-sol-bindings-and-registry-authority.md)).
  Hand-rolled encoder helpers for `GPv2Settlement`, `GPv2VaultRelayer`,
  `CoWSwapEthFlow`, the EIP-1967 proxy, and ERC-20 / ERC-20 Permit are
  absent from the workspace; byte-identity parity with the upstream
  Solidity surface is proven by the regression contract at
  `crates/contracts/tests/parity_contract.rs`.
