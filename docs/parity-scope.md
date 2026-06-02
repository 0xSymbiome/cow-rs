# Parity Scope

This document defines the review scope for parity evidence in `cow-rs`.

The source lock is the portable authority for upstream producer commits and
paths.

## Source Lock

Pinned sources live in `parity/source-lock.yaml`.

| Producer | Pinned role | Used for |
| --- | --- | --- |
| `cowprotocol/services` | Primary protocol authority | Orderbook HTTP API, OpenAPI schemas, wire DTOs, and order-validation and rejection semantics |
| `cowprotocol/contracts` | Primary protocol authority | EIP-712 order hashing, settlement ABI, and deployment addresses |
| `cowprotocol/cow-sdk` | Cross-language reference | Consumer-workflow and ergonomic reference (a different language with different idioms); SDK package configuration, COW Shed TypeScript constants, and shared package-level deployment evidence |
| `cowprotocol/composable-cow` | Primary capability evidence | Byte-identical composable-order Solidity mirrors (gated by `cargo parity-verify-sol-provenance` against `parity/source-lock.yaml`), deployment rows, selector fixtures, EIP-1271 payload shapes, and watch-tower boundary evidence |
| `cowprotocol/ethflowcontract` | Primary capability evidence | Byte-identical EthFlow Solidity mirrors (`CoWSwapEthFlow.sol`, `EthFlowOrder.sol`, `ICoWSwapOnchainOrders.sol`, `CoWSwapOnchainOrders.sol`, `IWrappedNativeToken.sol`) and the `ReceiverMustBeSet()` revert-selector provenance |
| `cowdao-grants/cow-shed` | Primary capability evidence | Byte-identical COW Shed Solidity mirrors, proxy creation-code bytes, factory address derivation, hook signature shape, and version-call evidence |
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
leaf manifests, deployment evidence, byte-identical ABI mirrors, parity fixtures, and
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
| Contracts | `cow-sdk-contracts` | `cowprotocol/contracts`, `cowprotocol/ethflowcontract`, `cowprotocol/composable-cow`, and `cowdao-grants/cow-shed` Solidity sources mirrored byte-identically under `crates/contracts/abi/**/*.sol` and gated by `cargo parity-verify-sol-provenance` against SHA-256 rows in `parity/source-lock.yaml`, `alloy::sol!`-generated bindings, the typed `Registry` deployment authority, and selected upstream test fixtures |
| Signing | `cow-sdk-signing` | Order-signing utilities, typed-data helpers, and contract-signing sources |
| App-data | `cow-sdk-app-data` | App-data helpers, schema imports, generated schema references, and schema regression tests |
| Orderbook | `cow-sdk-orderbook` | TypeScript orderbook sources plus selected `cowprotocol/services` OpenAPI and validation references |
| Trading | `cow-sdk-trading` | TypeScript trading workflows and tests |
| Subgraph | `cow-sdk-subgraph` | TypeScript subgraph API, GraphQL, query, and test sources |
| SDK facade | `cow-sdk` | TypeScript SDK root package exports and typedoc entrypoint |
| HTTP transport policy | `cow-sdk-transport-policy` | Retry, rate-limit, cooldown, jitter, and transport-classification behavior shared by typed HTTP clients |
| Native Alloy adapters | `cow-sdk-alloy-provider`, `cow-sdk-alloy-signer`, `cow-sdk-alloy` | Alloy runtime and Alloy Core source-lock pins, adapter contract tests, transaction broadcast / receipt shape invariants, and native examples |
| TypeScript-callable WASM | `cow-sdk-wasm` | Native Rust helper parity for typed-data, UID, digest, app-data, EIP-1271 payloads, orderbook/subgraph/IPFS/trading DTO shape, npm declaration snapshots, and upstream TypeScript SDK EIP-1271 vector coverage |
| Composable orders | `cow-sdk-composable` reserved manifest | Composable-CoW source locks, byte-identical Solidity mirrors, selector and EIP-1271 blob fixtures, handler revert fixtures, and watch-tower boundary documentation |
| COW Shed | `cow-sdk-cow-shed` reserved manifest | COW Shed source locks, byte-identical Solidity mirrors, proxy creation-code bytes, CREATE2 address fixtures, EIP-712 hook fixtures, and version-call evidence |

## Wire-Format Invariants

The canonical primitive layer per
[ADR 0052](adr/0052-alloy-primitives-canonical-primitive-layer.md) locks
the byte-identical wire-format contract across the cow newtype family at the
following invariants, each pinned by a parity fixture or a contract test:

- `Address` lowercase-canonical hex encoding regardless of input casing
- `Hash32` mixed-case input acceptance with lowercase-canonical output
- `HexData` odd-length nibble pad rule
- `Amount` strict-decimal-only `Deserialize` rejection of `0x`, `0o`, and
  `0b`-prefixed input that alloy's underlying `ruint::Uint::FromStr` would
  otherwise accept
- `SignedAmount` negative-value decimal-string round-trip
- the cow `TypedDataDomain` direct emission of the EIP-1193
  `eth_signTypedData_v4` wire shape (numeric `chainId`, required
  `verifyingContract`, no `salt`)
- the cow-shed `ExecuteHooks` calldata path
- the EIP-712 order-digest reference vectors
- the ECDSA `v`-normalization branches across the `{0, 1, 27, 28}` accepted
  set
- the RFC 8785 canonical-JSON UTF-16 code-unit ordering for non-ASCII keys
- the `Retry-After` HTTP-date IMF-fixdate, legacy RFC 850, and ANSI C
  `asctime` branches

The invariants are enforced by the parity fixtures under `parity/fixtures/`
and the regression tests at
`crates/core/tests/wire_format_preservation_contract.rs` and
`crates/browser-wallet/tests/signer_contract.rs`. The composable
multiplexer merkle-proof invariants land alongside the
`cow-sdk-composable` crate when that reserved manifest ships.

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
  manifests, byte-identical contract mirrors, deployment taxonomy rows, fixture artifacts, and
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
`cow-sdk-composable` manifest, deployment evidence, byte-identical ABI mirrors, selector
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
implement the SDK's `Provider`, `SigningProvider`, and `Signer`
trait seams to bridge a custom ecosystem.

### TypeScript-tooling-only packages

The upstream TypeScript SDK includes packages that exist to manage TypeScript
build orchestration (for example `typescript-config`, `config`). These have no
Rust analogue and are not in scope.

## Intentionally Out-of-Scope

Parity in `cow-rs` is byte-identity with the protocol authorities — services
on the wire and the contracts on-chain — on implemented surfaces, not
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
- **`availableBalance` order field** — the services schema marks this field
  deprecated and documents it as unused, always `null`, and slated for removal.
  The cow-rs `Order` response DTO does not model it; a response that still
  carries `availableBalance` deserializes with the value ignored and it is
  never re-emitted. Covered by the order-response wire-shape regression in
  `crates/orderbook/tests/fee_amount_is_not_a_public_builder_setter.rs`.
- **Legacy wire-string `Amount` wrapper** — the Rust SDK consolidated
  the canonical atomic amount to a single cow-owned
  `#[repr(transparent)]` newtype `cow_sdk_core::Amount` over
  `alloy_primitives::U256` per
  [ADR 0052](adr/0052-alloy-primitives-canonical-primitive-layer.md),
  with cow-owned serde that preserves the decimal-string wire format. The retired wire-string wrapper is simply
  absent from the workspace; by design, there is no negative test
  because the type does not exist and the Rust compiler itself enforces
  the exclusion at every call site. Governed by
  [ADR 0011](./adr/0011-typed-amount-boundary-and-typestate-ready-state-construction.md).
- **`TypedOrder` alias on `cow-sdk-signing`** — the canonical
  signed-order payload is `cow_sdk_core::OrderData` (the name mirrors
  the upstream services `OrderData`); the former `TypedOrder`
  backward-compatibility alias is absent from the workspace. As with the
  retired wire-string `Amount` wrapper, there is no negative test because
  the type does not exist and the Rust compiler itself enforces the
  exclusion at every call site.
- **Legacy free-function constructors on `OrderbookApi` and
  `SubgraphApi`** — the shipped construction seam for both clients is
  the typestate builder (`OrderbookApi::builder()` and
  `SubgraphApi::builder()`, governed by
  [ADR 0013](./adr/0013-http-transport-injection-and-typestate-builders.md)).
  The earlier family of free-function constructors (for example
  `from_shared_client`, `new_with_transport_policy`, `new_with_base_url`
  on the orderbook client and the matching set on the subgraph client)
  is absent from the workspace; the Rust compiler itself enforces the
  exclusion at every call site. Separately, on `wasm32` the
  default-transport `.build()` is `cfg`-gated off, so a browser consumer
  must inject a `FetchTransport` before `.build()` is reachable; compiling
  the crate for `wasm32` in CI guards that gate.
- **Auction-retrieval method (`get_auction`), the `Auction` response wrapper,
  and the `AuctionOrder` mirror** — `/api/v1/auction` is not reachable for
  public clients and is treated upstream as a liveness probe rather than a
  consumer data feed, so the SDK exposes neither a `get_auction` method nor an
  `Auction` response type. Because no public endpoint produces an auction
  snapshot, the `AuctionOrder` mirror and its auction-side `quote: Quote` had no
  reachable producer and are not modeled either; the order-shaped response
  surface is the single `Order` type. As with the other retired surfaces above,
  there is no negative test because the items do not exist and the Rust compiler
  enforces the exclusion at every call site. Auction retrieval and the
  `AuctionOrder` mirror can return as an additive change if the endpoint becomes
  publicly consumable. Governed by
  [ADR 0031](adr/0031-wire-dto-openapi-driven-with-order-auction-order-split.md).
- **Strict OpenAPI-optionality coverage for `SolverCompetitionResponse`** — the
  vendored `/api/v2/solver_competition/*` schema omits a `required:` block, so
  the `openapi-coverage --validate` optionality check would force every field —
  including the always-present `auctionId`, the block deadlines, and `auction` —
  to `Option<T>`. The upstream producer (the `Response` struct in `services`
  `solver_competition_v2.rs`, serialized behind that route) instead models the
  identity and collection fields as required and only `txHash` / `referenceScore`
  as optional, and the SDK's typed `SolverCompetitionResponse` mirrors that
  producer contract exactly. The type is therefore covered by a producer-pinned
  round-trip fixture (`parity/fixtures/orderbook/solver_competition_response.json`
  exercised by `crates/orderbook/tests/transform_contract.rs`) rather than the
  OpenAPI-optionality manifest, which would degrade the typed boundary against
  the verified producer. Governed by
  [ADR 0031](adr/0031-wire-dto-openapi-driven-with-order-auction-order-split.md).
- **Hand-rolled ABI encoders in `cow-sdk-contracts`** — every binding
  shipped by the contracts crate is generated through `alloy::sol!` from
  the byte-identical Solidity mirrors committed under
  `crates/contracts/abi/` and gated by `cargo parity-verify-sol-provenance`
  against the SHA-256 rows in `parity/source-lock.yaml` (governed by
  [ADR 0012](./adr/0012-alloy-sol-bindings-and-registry-authority.md)).
  Hand-rolled encoder helpers for `GPv2Settlement`, `GPv2VaultRelayer`,
  `CoWSwapEthFlow`, `CoWSwapOnchainOrders`, the wrapped-native token, the
  EIP-1967 proxy, and ERC-20 / ERC-20 Permit are absent from the workspace; byte-identity parity with the upstream
  Solidity surface is proven by the regression contract at
  `crates/contracts/tests/parity_contract.rs`.
