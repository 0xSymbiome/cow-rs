# Parity And Provenance

This document defines the parity authorities for `cow-rs`, the committed
source-lock contract that pins them, the surface-to-evidence map, and the
in-scope and out-of-scope boundaries for the release.

Authority order:

1. `parity/source-lock.yaml`
2. this document
3. `docs/release-checklist.md`
4. committed parity fixtures and executable tests

## Authority Model

`cow-rs` interoperates with a live protocol, so its parity authorities are the
upstream producers that define the protocol contract on the wire and on-chain.
Parity is byte-identity with those producers on implemented surfaces, not
feature-identity with the upstream TypeScript SDK.

Primary protocol authorities:

- `https://github.com/cowprotocol/services` — the off-chain authority for the
  orderbook HTTP API, the OpenAPI schemas, the wire DTOs, and the
  order-validation and rejection semantics the SDK must match to interoperate.
- `https://github.com/cowprotocol/contracts` — the on-chain authority for
  EIP-712 order hashing, the settlement ABI, and deployment addresses.
- `https://github.com/cowprotocol/ethflowcontract` — the on-chain authority for
  the EthFlow order surface.

Prior art (not a pinned parity source):

- `https://github.com/cowprotocol/cow-sdk` — the upstream TypeScript SDK is
  prior art for the trading consumer-workflow shape (the quote-to-sign-to-post
  orchestration); the slippage convention cow-rs implements faithfully is
  documented in
  [ADR 0066](adr/0066-trading-slippage-and-suggestion-policy.md). It is **not** a
  pinned parity source and is not listed in `parity/source-lock.yaml`: it does
  not define the Rust public API shape (Rust idiom governs that), the wire format
  (services), on-chain shapes (contracts), the app-data schemas
  (`cowprotocol/cow-sdk`), or the subgraph schema (the deployed Graph).

## Source Lock

Pinned sources live in `parity/source-lock.yaml`, the portable authority for
upstream producer commits and paths.

| Producer | Used for |
| --- | --- |
| `cowprotocol/services` | Orderbook HTTP API, OpenAPI schemas (vendored to `parity/openapi/services-orderbook.yml`), wire DTOs, the solver-competition v2 producer, the flash-loan hint shape, and order-validation and rejection semantics |
| `cowprotocol/contracts` | EIP-712 order hashing (including the `GPv2Signing` domain block), settlement ABI, and deployment addresses |
| `cowprotocol/cow-sdk` | Commit pin for the TypeScript SDK contract-address constants behind the staging settlement and vault-relayer deployments resolved by the typed `Registry`, the COW Shed factory ABI and version constants with their CREATE2 goldens, the trading protocol-fee amount-composition goldens, and the canonical app-data JSON Schema families (hooks/flashloan/quote/partnerFee/definitions) the `parity/fixtures/app_data/` fixtures track; this monorepo (published as `@cowprotocol/sdk-app-data`) is their canonical home, and the standalone `cowprotocol/app-data` repo it long ago superseded is deprecated |
| `cowprotocol/ethflowcontract` | Commit pin for the inline `sol!` EthFlow bindings (`CoWSwapEthFlow`, `EthFlowOrder`, `ICoWSwapOnchainOrders`, `CoWSwapOnchainOrders`, `IWrappedNativeToken`) proven by parity fixtures, plus the `ReceiverMustBeSet()` revert-selector evidence |
| `cowdao-grants/cow-shed` | Commit pin (the v1.0.1 tag — the deployed generation the inline `sol!` COW Shed bindings mirror) proven by JSON fixtures, plus the proxy creation-code `.bin` bytes locked by the CREATE2 address-parity test, factory address derivation, hook signature shape, and the per-version deployment record |

Each repository row carries a `# why:` comment in the lock itself; the lock is
the single home for the pinned commits and producer paths.

Off-chain orchestration behavior (for example `cowprotocol/watch-tower`) is
consulted as ecosystem context to define what stays outside the SDK; it is a
boundary statement rather than a pinned source, so it carries no commit pin.

The pinned commits themselves are authoritative in
`parity/source-lock.yaml` and are not duplicated here.

Normal `cow-rs` builds, tests, and publishes never require local checkouts of
the upstream repositories. Local upstream checkout paths are optional validation
inputs; when used, they must be independent git checkouts at the pinned
commits — `cargo xtask parity sync` materializes (or re-detaches) exactly that
layout, one blob-less clone per lock repository under `<root>/<id>`.

Provenance is layered so it is always reproducible from the committed record,
never from a caller-local copy: (1) `parity/source-lock.yaml` pins each producer
to a commit and every fixture's header cites producer paths under one of those
pins; (2) provenance-sensitive verification materializes each pinned repository
as an independent checkout and validates its remote and `HEAD` against the pin;
(3) `cargo xtask parity sync --root <dir>` reproduces the layout for reviewers.

## Validation Modes

Repo-local validation does not require upstream checkouts:

```text
cargo parity-validate
```

That validates the lock by form, every fixture header under
`parity/fixtures/**/*.json` against the pins (cited repositories, the
commit-equality freshness ratchet, refs confined to declared producer paths),
and the vendored OpenAPI stamp against the services pin.

Upstream-root validation is stricter: it deep-checks **every** lock repository
against the checkout at `<root>/<id>` and compares the vendored OpenAPI body
against the blob at the pinned services commit:

```text
cargo xtask parity sync --root <dir>
cargo parity-validate --upstream-root <dir>
```

For each repository the validator requires the git top-level, a remote
matching the expected upstream, `HEAD` at the pinned commit, and all declared
producer paths present and clean relative to `HEAD`. The scheduled
`upstream-drift` workflow runs `cargo xtask parity drift` weekly to report
producer-path movement — and, for any watched directory (`watch_dirs`, e.g. the
app-data schema tree), files added or removed between the pin and the upstream
default branch, so an additively versioned schema cannot land unseen. The
maintainer workflow for refreshing the lock lives in
[parity/README.md](../parity/README.md).

## Surface Matrix

| Surface | Primary upstream producers | Rust crates | Committed authority | Primary evidence |
| --- | --- | --- | --- | --- |
| Order creation, signing, and submission | `cowprotocol/services` order-creation and quote DTOs and `cowprotocol/contracts` EIP-712 signing; the slippage layer follows the CoW SDK convention (ADR 0066) | `cow-sdk-signing`, `cow-sdk-orderbook`, `cow-sdk-trading`, `cow-sdk` | `parity/fixtures/orderbook-requests/order_creation.json` | `crates/signing/tests/order_signing_contract.rs`, `crates/orderbook/tests/api_contract.rs`, `crates/trading/tests/post_contract.rs`, `crates/trading/tests/sdk_contract.rs`, `crates/sdk/tests/public_api.rs`, `crates/sdk/tests/public_api_default_features_only.rs`, `crates/sdk/tests/public_api_with_all_features.rs` |
| Contracts parity | `cowprotocol/contracts` | `cow-sdk-contracts`, `cow-sdk-signing` | `parity/fixtures/contracts.json` | `crates/contracts/tests/order_contract.rs`, `crates/contracts/tests/settlement_events_contract.rs`, `crates/contracts/tests/parity_contract.rs`, `crates/signing/tests/eip1271_contract.rs` |
| Codec fuzz corpora | `cowprotocol/contracts` order UID and EIP-712 typed-data helpers | `cow-sdk-contracts`, `cow-sdk-signing` | `parity/fixtures/contracts.json` (fuzz corpus seeds are generated out-of-tree, not committed — see `fuzz/README.md`) | `fuzz/fuzz_targets/fuzz_order_uid_pack_unpack.rs`, `fuzz/fuzz_targets/fuzz_typed_data_digest.rs`, and the seed-class taxonomy in `docs/audit/fuzz-coverage-audit.md` |
| `GPv2Settlement` bindings | `cowprotocol/contracts` settlement surface | `cow-sdk-contracts::settlement` via inline `alloy::sol!` | Inline `sol!` binding proven by `parity/fixtures/contracts.json`, mirroring upstream Solidity pinned by commit in `parity/source-lock.yaml` | `crates/contracts/tests/parity_contract.rs::parity_fixture_cases_hold` |
| `CoWSwapEthFlow` bindings | `cowprotocol/ethflowcontract` surface | `cow-sdk-contracts::eth_flow` via inline `alloy::sol!` | Inline `sol!` binding proven by `parity/fixtures/contracts.json`, mirroring upstream Solidity pinned by commit in `parity/source-lock.yaml` | `crates/contracts/tests/parity_contract.rs::parity_fixture_cases_hold` |
| `CoWSwapOnchainOrders` event decoder | `cowprotocol/ethflowcontract` `CoWSwapOnchainOrders` mixin and interface | `cow-sdk-contracts::onchain_orders` via inline `alloy::sol!` | Inline `sol!` binding proven by selector and order-hash fixtures, mirroring upstream Solidity pinned by commit in `parity/source-lock.yaml` | `crates/contracts/tests/onchain_orders.rs::order_placement_topic0_matches_canonical_hash`, `crates/contracts/tests/onchain_orders.rs::order_hash_matches_canonical_ethflow_foundry_vector` |
| `IWrappedNativeToken` (WETH9-family) bindings | `cowprotocol/ethflowcontract` `IWrappedNativeToken` interface | `cow-sdk-contracts::tokens` via inline `alloy::sol!` | Inline `sol!` binding proven by deposit/withdraw selector fixtures, mirroring upstream Solidity pinned by commit in `parity/source-lock.yaml` | `crates/contracts/tests/tokens_contract.rs::deposit_selector_matches_canonical_keccak`, `crates/contracts/tests/tokens_contract.rs::withdraw_selector_matches_canonical_keccak` |
| ERC-20 bindings | `cowprotocol/contracts` `IERC20` interface (carrying its own OpenZeppelin v3.4.0 lineage in the upstream header) | `cow-sdk-contracts::tokens` via inline `alloy::sol!` | Inline `sol!` binding for `IERC20` proven by `parity/fixtures/contracts.json`, mirroring upstream Solidity pinned by commit in `parity/source-lock.yaml` | `crates/contracts/tests/parity_contract.rs::parity_fixture_cases_hold` |
| Deployment registry authority | `cowprotocol/contracts` deployments record | `cow-sdk-contracts::Registry` const table | `crates/contracts/src/deployments.rs` | `crates/contracts/src/deployments.rs (tests)`, `xtask/tests/registry_confirm.rs` |
| App-data parity | `cowprotocol/cow-sdk` app-data JSON schemas and `cowprotocol/services` app-data hashing | `cow-sdk-app-data`, `cow-sdk-trading` | `parity/fixtures/app_data/` | `crates/app-data/tests/cid_contract.rs`, `crates/app-data/tests/schema_contract.rs`, `crates/app-data/tests/fetch_contract.rs`, `crates/trading/tests/quote_contract.rs` |
| Subgraph support | the deployed CoW Protocol subgraph GraphQL schema, with cow-rs-owned query documents | `cow-sdk-subgraph` | `crates/subgraph/src/query_documents/` | `crates/subgraph/tests/api_contract.rs`, `crates/subgraph/tests/query_contract.rs`, `crates/subgraph/tests/types_contract.rs` |
| Orderbook transport | `cowprotocol/services` orderbook OpenAPI and wire DTOs | `cow-sdk-orderbook` | `parity/fixtures/orderbook-requests/`, `parity/openapi/coverage.yaml` | `crates/orderbook/tests/api_contract.rs`, `crates/orderbook/tests/request_contract.rs`, `crates/orderbook/tests/transform_contract.rs`, `crates/orderbook/tests/types_contract.rs`, `crates/orderbook/tests/wire_contract.rs` |
| WASM target | the cow-rs SDK helper surface compiled to WASM | `cow-sdk`, `cow-sdk-app-data`, `cow-sdk-wasm`, the WASM package examples | committed workflow definitions, example READMEs | `crates/wasm/tests/transport_parity_contract.rs`, `crates/wasm/tests/transport_fetch_contract.rs`, `crates/wasm/tests/transport_fetch_smoke.rs`, `wasm-pack test --headless --firefox`, and the `wasm.yml` compatibility workflow |
| WASM event-log decoders | `cowprotocol/contracts` settlement surface and `cowprotocol/ethflowcontract` mixin | `cow-sdk-wasm` `decodeSettlementLog` / `decodeEthFlowLog` over the `cow-sdk-contracts` decoders | Facade and raw TypeScript declaration snapshots under `crates/wasm/snapshots/` | `crates/wasm/tests/wasm_facade_snapshot_contract.rs::facade_declarations_match_flavour_matrix` |
| Host wallet callback boundary | the EIP-1193 `request` semantics owned by the host JS wallet | `cow-sdk-wasm` typed callbacks (the EIP-1193 request callback) | Facade and raw TypeScript declaration snapshots under `crates/wasm/snapshots/` | `crates/wasm/tests/wasm_facade_snapshot_contract.rs::facade_declarations_match_flavour_matrix` |
| Native Alloy adapters | `alloy` and `alloy-core` crates.io version pins (`2.0.4` / `1.5.7`) plus local trait contracts | `cow-sdk-alloy-provider`, `cow-sdk-alloy-signer`, `cow-sdk-alloy`, `cow-sdk` opt-in features | workspace `Cargo.toml` version pins, `Cargo.lock`, `docs/providers/adapting-alloy.md`, `examples/native/README.md` | `crates/alloy-provider/tests/*`, `crates/alloy-signer/tests/*`, `crates/alloy/tests/*`, `tests/alloy_umbrella_composition.rs` |

## Orderbook Rejection Tags

`OrderbookRejection` models 50 variants including the forward-compatible
`Unknown` fallback. Every `errorType` the orderbook OpenAPI documents is pinned
as a closed set in `parity/fixtures/orderbook/rejection_error_types.json` and
asserted to classify to a typed variant, so an upstream enum addition surfaces
when the vendored OpenAPI is re-stamped at a newer pin. The GET-side
trade-filter and pagination tags below are represented directly and preserve
services wire spelling.

| Services wire tag | Rust variant | Primary upstream producer | Primary evidence |
| --- | --- | --- | --- |
| `InvalidTradeFilter` | `OrderbookRejection::InvalidTradeFilter` | `cowprotocol/services` orderbook trade lookup filters | `crates/orderbook/tests/rejection_contract.rs::every_known_services_tag_parses_to_its_typed_variant` |
| `InvalidLimit` | `OrderbookRejection::InvalidLimit` | `cowprotocol/services` orderbook trade pagination limits | `crates/orderbook/tests/rejection_contract.rs::every_known_services_tag_parses_to_its_typed_variant` |
| `LIMIT_OUT_OF_BOUNDS` | `OrderbookRejection::LimitOutOfBounds` | `cowprotocol/services` user-order lookup pagination limits | `crates/orderbook/tests/rejection_contract.rs::every_known_services_tag_parses_to_its_typed_variant` |

## Defaults

The `metadata.utm` row below is a local Rust SDK attribution policy rather than
an upstream fixture vector. It is asserted by
`crates/trading/tests/quote_contract.rs::default_utm_block_uses_env_cargo_pkg_version`
rather than by a committed parity fixture.

| Surface | Default | Opt-out / opt-in |
| --- | --- | --- |
| `OrderToSignParams::new(...)` `apply_costs_slippage_and_fees` | applied on by default (cost, slippage, partner-fee, and protocol-fee adjustments are folded into the unsigned order amounts) | call `.with_apply_costs_slippage_and_fees(false)` to preserve raw caller amounts |
| `build_app_data` `metadata.utm` | when the caller does not supply `metadata.utm`, the helper stamps an SDK-family attribution block with `utmSource = "cow-sdk"`, `utmMedium = "cow-rs@<crate-version>"`, `utmCampaign = "developer-cohort"`, `utmContent = "wasm"` on `wasm32` targets and `""` otherwise, and `utmTerm = "rs"` so downstream analytics can group CoW SDK traffic while distinguishing the Rust SDK and its published version | supply any `metadata.utm` key in the advanced app-data parameters — partial or full — and the caller-declared block is carried through byte-identical with no defaults merged on top |
| `Order.total_fee` | computed narrowly as the canonical executed-fee component (the typed `executed_fee` directly, defaulting to zero when absent); the legacy wire field `executedFeeAmount` is never folded into the canonical sum | `Order.executed_fee_amount: Amount` surfaces the legacy wire value as a typed read-only sibling so consumers that need the legacy summation compute `executed_fee + executed_fee_amount` explicitly at the call site |

## Wire-Format Invariants

The canonical primitive layer per
[ADR 0052](adr/0052-alloy-primitives-canonical-primitive-layer.md) locks the
byte-identical wire-format contract across the cow newtype family at the
following invariants, each pinned by a parity fixture or a contract test:

- `Address` lowercase-canonical hex encoding regardless of input casing
- `Hash32` mixed-case input acceptance with lowercase-canonical output
- `HexData` odd-length nibble pad rule
- `Amount` strict-decimal-only `Deserialize` rejection of `0x`, `0o`, and
  `0b`-prefixed input that alloy's underlying `ruint::Uint::FromStr` would
  otherwise accept
- the cow `TypedDataDomain` direct emission of the EIP-1193
  `eth_signTypedData_v4` wire shape (numeric `chainId`, required
  `verifyingContract`, no `salt`)
- the cow-shed `ExecuteHooks` calldata path
- the EIP-712 order-digest reference vectors
- the ECDSA `v`-normalization branches across the `{0, 1, 27, 28}` accepted set
- the RFC 8785 canonical-JSON UTF-16 code-unit ordering for non-ASCII keys
- the `Retry-After` HTTP-date IMF-fixdate, legacy RFC 850, and ANSI C `asctime`
  branches

The invariants are enforced by the parity fixtures under `parity/fixtures/` and
the regression tests at
`crates/core/tests/wire_format_preservation_contract.rs`. The composable multiplexer
merkle-proof invariants land with the deferred composable capability recorded
by [ADR 0048](adr/0048-composable-conditional-order-framework.md).

## Schema Evidence Policy

Schema-derived evidence is a review aid, not a public API shortcut. No generated
or schema-derived Rust mirror is part of the public SDK API.

- orderbook schema evidence is tied to `cowprotocol/services`, including
  `parity/openapi/services-orderbook.yml`, and is committed as OpenAPI artifacts,
  fixtures, contract tests, and source-lock references
- subgraph schema evidence is tied to the deployed CoW Protocol subgraph GraphQL
  schema; the query documents are cow-rs-owned and live in
  `crates/subgraph/src/query_documents/`, with response-shape parity exercised by
  the subgraph contract tests
- generated or schema-derived Rust mirrors stay non-public or test-only

## 0.1.0 Scope

Parity for `cow-rs` 0.1.0 is defined by supported workflows, not by a percentage
of upstream TypeScript methods. The release supports these workflow buckets:

1. **Deterministic order primitives**: order UID calculation, EIP-712 typed data
   envelopes, and EIP-1271 signature payload generation from wrapped ECDSA
   signatures.
2. **Order signing flows**: typed-data EIP-712 signing, raw EIP-1193 signing,
   EIP-191 digest signing, EIP-1271 wrapping, custom EIP-1271 signatures, and
   cancellation typed data.
3. **Orderbook operations**: quote, signed order submission, raw order-creation
   submission, order lookup, owner order pagination, trade lookup, native price
   lookup, app-data lookup, app-data upload, and signed cancellation submission.
4. **Trading orchestration**: quote, quote-sign-post, quote-result reuse, limit
   order posting, native-sell transaction construction, allowance reads, and
   EIP-1271-backed swap posting.
5. **Subgraph reads**: protocol totals, recent daily and hourly volume, and
   arbitrary GraphQL query execution.
6. **App-data tools**: app-data document generation, CID and hash derivation,
   metadata validation, CID-to-hex conversion, and hex-to-CID conversion.
7. **IPFS app-data fetch**: fetch by CID and fetch by app-data hash through an
   injected HTTP transport.
8. **Deployment registry**: production and staging addresses for the GPv2
   contract families (settlement, vault relayer, eth-flow) plus the COW Shed
   per-version address tables, with typed misses for chains where a contract
   is not deployed.
9. **Runtime support**: browser bundlers, Node.js 22 and 24 LTS, Cloudflare
   Workers, Deno, and Vercel Edge through the shipped `trading` flavour's web (edge) build.
10. **Cancellation and timeouts**: per-call `signal`, per-call `timeoutMs`, and
    wallet callback `walletConfig.timeoutMs`.

The 0.1.0 scope does not claim total method-for-method parity with the upstream
TypeScript SDK. The COW Shed account-abstraction proxy ships its full helper body
in 0.1.0 — the `cow-sdk-contracts` leaf crate behind the opt-in `cow-shed` facade
feature. Composable conditional-order helpers are deferred and recorded only by
[ADR 0048](adr/0048-composable-conditional-order-framework.md); the helper
surface and its upstream pin land additively in a later release. Capability families that are
explicitly deferred for 0.1.0 (cross-chain bridging order construction,
hook-trampoline bytecode chaining, ecosystem provider adapters outside Alloy, and
other items listed under Out-of-Scope below) should continue to use the upstream
packages until their dedicated `cow-rs` leaf crates land.

## First-Release Scope

The Rust SDK ships in scope:

- core domain types and runtime traits (`cow-sdk-core`)
- `alloy::sol!`-generated contract bindings and Registry (`cow-sdk-contracts`)
- order signing and EIP-1271 verification (`cow-sdk-signing`)
- app-data encoding and validation (`cow-sdk-app-data`)
- typed orderbook transport (`cow-sdk-orderbook`)
  - `Order` covers the orderbook OpenAPI `Order` schema
    (`OrderCreation` + `OrderMetaData` + `interactions`)
  - `OrderQuoteResponse`, `Trade`, `StoredOrderQuote`, and `OnchainOrderData`
    cover their OpenAPI schemas as separate typed mirrors
- typed subgraph transport (`cow-sdk-subgraph`)
- quote-to-order trading workflows (`cow-sdk-trading`)
- browser-target HTTP transport (`FetchTransport`, the target-gated
  `transport::fetch` module of `cow-sdk-core`)
- opt-in native Alloy provider, signer, and composed provider-plus-signer
  adapters (`cow-sdk-alloy-provider`, `cow-sdk-alloy-signer`, `cow-sdk-alloy`)
- TypeScript-callable wasm-bindgen bindings (`cow-sdk-wasm`) with typed
  JavaScript callbacks for wallet, signer, EIP-1271, and HTTP dispatch
- the shipped COW Shed account-abstraction helper crate (`cow-sdk-contracts`),
  backed by inline `sol!` bindings, deployment taxonomy rows, JSON fixtures, and
  audit records

Native Alloy transaction parity is scoped to the SDK trait contract, not to
re-exporting Alloy's full transaction surface. The composed signer returns
`TransactionBroadcast` from the hash Alloy has already accepted for broadcast,
and provider receipt lookup populates `TransactionReceipt` fields that the SDK
models: status, block number, block hash, gas used, sender, and recipient.

The first release does **not** ship every helper crate body below. Deployment
registry rows and compatibility fixtures are in scope where listed, while full
ergonomic helper APIs remain additive under
[ADR 0001](adr/0001-multi-crate-sdk-family-with-thin-facade.md).

### Bridging

Cross-chain order construction equivalent to the upstream `bridging` capability.
Deferred; not in scope for the first release. The planned home is a future
`cow-sdk-bridging` leaf crate.

### Composable orders

Composable-CoW order construction is deferred and recorded only by
[ADR 0048](adr/0048-composable-conditional-order-framework.md). No
`cow-sdk-composable` crate ships; composable deployment addresses already
resolve through the typed `Registry`, and the order-construction helpers land
additively in a later release. Until then, use the upstream composable surface.

### Cow-shed

COW Shed ships in 0.1.0 as the `cow-sdk-contracts` leaf crate, opt-in through the
off-by-default `cow-shed` facade feature (re-exported as `cow_sdk::cow_shed`) and
never on the default `cow-sdk` closure. The crate covers deterministic proxy
derivation (`proxy_of` / `proxy_for` — chain-independent, since each deployed
generation's factory and implementation are identical on every supported chain),
EIP-712 domain + signing hash, the `ExecuteHooks` typed-data payload, factory
calldata encoding for both externally-owned and EIP-1271 smart-contract owners,
and the `CowShedHooks` sign-and-encode orchestrator, all backed by proxy
creation-code digest pinning (byte-identical to the TypeScript arbiter's
constants), CREATE2 address fixtures anchored on the arbiter's own golden
vectors, hook digest fixtures, and the per-version deployed-generation record.
The bindings mirror the deployed v1.0.x generation (cow-shed pinned at the
v1.0.1 tag); the v2.x source generations (ENS purge, pre-sign flow, composable
forwarder) are deployed only as the out-of-family Gnosis redeploy and land
later as explicit new `CowShedVersion` variants. The lock marks this pin with a
`hold:` reason, so `parity drift` reports its upstream movement for visibility
without counting it as actionable drift and `parity sync --update` never
auto-advances it. ENS-record helpers remain additive.

### Flash loans

The flashloan metadata sub-field is supported in `cow-sdk-app-data`. A flashloan
helper utility surface is deferred; not in scope for the first release.

### Weiroll

Hook-trampoline bytecode chaining. Deferred; not in scope for the first release.

### Additional provider ecosystems

Additional provider ecosystems beyond the native Alloy adapter and the
host-wallet EIP-1193 callback served by `cow-sdk-wasm` are not in scope for the
first release. Consumers can implement the SDK's `Provider`, `SigningProvider`,
and `Signer` trait seams to bridge a custom ecosystem.

### TypeScript-tooling-only packages

The upstream TypeScript SDK includes packages that exist to manage TypeScript
build orchestration (for example `typescript-config`, `config`). These have no
Rust analogue and are not in scope.

## Intentionally Out-of-Scope

The following upstream surfaces are intentionally excluded from the Rust SDK
because they carry no pre-release user value, re-introduce known protocol
footguns, or have been superseded by a clearer typed boundary. Every exclusion
below is enforced in code (via a negative test or by removing the surface
entirely) so future contributors cannot quietly reintroduce the upstream shape on
the assumption that a missing positive fixture implies a gap.

See [ADR 0011](./adr/0011-typed-amount-boundary-and-typestate-ready-state-construction.md)
for the canonical typed-amount decision. The governing parity-scope discipline is
the four-layer defense: a negative test that fails closed, a scope entry that
names the exclusion, a cross-link to the owning ADR, and a risk-register entry
for anyone who later considers reintroducing the surface.

- **CIDv0 (`Qm...`) encoding and decoding** — the cow-protocol services backend
  enforces CIDv1 with the raw multicodec (`0x55`) over a keccak-256 multihash
  (`0x1b`) as the only supported CID shape; legacy CIDv0 (dag-pb + sha2-256) paths
  carry no pre-release user value. Consumers that need to resolve historical
  `Qm`-prefixed values use a general-purpose `cid` crate directly. Negative test:
  `crates/app-data/tests/cid_contract.rs::unsupported_and_malformed_cids_are_rejected`.
- **Order-level `fee_amount` as a public builder setter or DTO field** — the
  cow-protocol services backend rejects orders that carry a non-zero order-level
  fee, so the submission path always wires `"feeAmount": "0"` and there is no
  reason to let a caller construct a non-zero value locally. The internal
  serializer preserves `"feeAmount": "0"` for EIP-712 struct-hash compatibility.
  Negative test:
  `crates/orderbook/tests/fee_amount_is_not_a_public_builder_setter.rs`.
- **Legacy quote-response fee descriptors `executedFeeAmount` and
  `fullFeeAmount`** — the current services schema surfaces executed fees through
  the canonical `executedFee` component and quote-response protocol fees through
  `protocolFeeBps`. The retired descriptors are not re-emitted on the cow-rs wire.
  Covered by the same order-response wire-shape regressions in
  `crates/orderbook/tests/fee_amount_is_not_a_public_builder_setter.rs`.
- **`availableBalance` order field** — the services schema marks this field
  deprecated and documents it as unused, always `null`, and slated for removal.
  The cow-rs `Order` response DTO does not model it; a response that still carries
  `availableBalance` deserializes with the value ignored and it is never
  re-emitted. Covered by the order-response wire-shape regression in
  `crates/orderbook/tests/fee_amount_is_not_a_public_builder_setter.rs`.
- **Legacy wire-string `Amount` wrapper** — the Rust SDK consolidated the
  canonical atomic amount to a single cow-owned `#[repr(transparent)]` newtype
  `cow_sdk_core::Amount` over `alloy_primitives::U256` per
  [ADR 0052](adr/0052-alloy-primitives-canonical-primitive-layer.md), with
  cow-owned serde that preserves the decimal-string wire format. The retired
  wire-string wrapper is simply absent from the workspace; by design, there is no
  negative test because the type does not exist and the Rust compiler itself
  enforces the exclusion at every call site. Governed by
  [ADR 0011](./adr/0011-typed-amount-boundary-and-typestate-ready-state-construction.md).
- **`TypedOrder` alias on `cow-sdk-signing`** — the canonical signed-order payload
  is `cow_sdk_core::OrderData` (the name mirrors the upstream services
  `OrderData`); the former `TypedOrder` backward-compatibility alias is absent
  from the workspace. As with the retired wire-string `Amount` wrapper, there is
  no negative test because the type does not exist and the Rust compiler itself
  enforces the exclusion at every call site.
- **Legacy free-function constructors on `OrderbookApi` and `SubgraphApi`** — the
  shipped construction seam for both clients is the typestate builder
  (`OrderbookApi::builder()` and `SubgraphApi::builder()`, governed by
  [ADR 0013](./adr/0013-http-transport-injection-and-typestate-builders.md)). The
  earlier family of free-function constructors (for example `from_shared_client`,
  `new_with_transport_policy`, `new_with_base_url` on the orderbook client and the
  matching set on the subgraph client) is absent from the workspace; the Rust
  compiler itself enforces the exclusion at every call site. Separately, on
  `wasm32` the default-transport `.build()` is `cfg`-gated off, so a browser
  consumer must inject a `FetchTransport` before `.build()` is reachable;
  compiling the crate for `wasm32` in CI guards that gate.
- **Auction-retrieval method (`get_auction`), the `Auction` response wrapper, and
  the `AuctionOrder` mirror** — `/api/v1/auction` is not reachable for public
  clients and is treated upstream as a liveness probe rather than a consumer data
  feed, so the SDK exposes neither a `get_auction` method nor an `Auction`
  response type. Because no public endpoint produces an auction snapshot, the
  `AuctionOrder` mirror and its auction-side `quote: Quote` had no reachable
  producer and are not modeled either; the order-shaped response surface is the
  single `Order` type. As with the other retired surfaces above, there is no
  negative test because the items do not exist and the Rust compiler enforces the
  exclusion at every call site. Auction retrieval and the `AuctionOrder` mirror
  can return as an additive change if the endpoint becomes publicly consumable.
  Governed by
  [ADR 0031](adr/0031-wire-dto-openapi-driven-with-order-auction-order-split.md).
- **Strict OpenAPI-optionality coverage for `SolverCompetitionResponse`** — the
  vendored `/api/v2/solver_competition/*` schema omits a `required:` block, so the
  `openapi-coverage` optionality check would force every field —
  including the always-present `auctionId`, the block deadlines, and `auction` —
  to `Option<T>`. The upstream producer (the `Response` struct in `services`
  `solver_competition_v2.rs`, serialized behind that route) instead models the
  identity and collection fields as required and only `txHash` / `referenceScore`
  as optional, and the SDK's typed `SolverCompetitionResponse` mirrors that
  producer contract exactly. The type is therefore covered by a producer-pinned
  round-trip fixture (`parity/fixtures/orderbook/solver_competition_response.json`
  exercised by `crates/orderbook/tests/transform_contract.rs`) rather than the
  OpenAPI-optionality manifest, which would degrade the typed boundary against the
  verified producer. Governed by
  [ADR 0031](adr/0031-wire-dto-openapi-driven-with-order-auction-order-split.md).
- **Hand-rolled ABI encoders in `cow-sdk-contracts`** — every binding shipped by
  the contracts crate is declared inline with `alloy::sol!` and proven
  byte-for-byte by the call-data and selector fixtures under `parity/fixtures/`,
  mirroring upstream Solidity pinned by commit in `parity/source-lock.yaml`
  (governed by
  [ADR 0012](./adr/0012-alloy-sol-bindings-and-registry-authority.md)).
  Hand-rolled encoder helpers for `GPv2Settlement`, `CoWSwapEthFlow`,
  `CoWSwapOnchainOrders`, the wrapped-native token, and ERC-20 / ERC-20 Permit
  are absent from the workspace; byte-identity parity with the upstream Solidity
  surface is proven by the regression contract at
  `crates/contracts/tests/parity_contract.rs`.

## Supported Networks

The Rust SDK supports the CoW Protocol chains enumerated by
`cow_sdk_core::config::SupportedChainId`. Per-chain numeric ids, deployment
provenance, services-generated metadata, TypeScript SDK support, and
wrapped-native-token evidence are maintained in the
[Deployment Registry Audit](audit/deployment-registry-audit.md#per-chain-provenance)
rather than repeated here. Endpoint discovery via the `OrderbookApi::builder()`
and `SubgraphApi::builder()` typestate chains — each given the chain id and
environment — honors production versus staging environment selection through the
typed API context.

## Publish Order

The published crate-family dry-run and publish order is maintained in the
[Release Checklist](release-checklist.md).

## Provenance Rule

Only repositories listed in `parity/source-lock.yaml` are parity sources.
Repositories that are not listed there are not fixture provenance, source-lock
inputs, or justification for copied literals or defaults.

## Maintenance Rules

- do not point parity evidence at floating upstream `main`
- update pinned SHAs only in dedicated parity refresh changes
- keep fixture provenance explicit in every `parity/fixtures/**/*.json` header
  (`cargo parity-validate` fails closed on a fixture without one)
- keep fixture `sources` commits aligned with `parity/source-lock.yaml` — the
  validator enforces equality, so a pin bump names every fixture that still
  needs re-verification
- keep the `parity/fixtures/app_data/schemas/` mirrors synchronized from
  the `cow-sdk` repository pinned in `parity/source-lock.yaml` (every schema
  mirror, including the flash-loan one, cites the `cow-sdk` producer in its
  header; only the data fixture `flashloan_v1.7.0.json` tracks the `services`
  producer): the per-family mirrors, plus the root-document manifest mirror
  (`app-data-document-v*.json`) that anchors the emitted document version and
  the in-force family versions — the `schema_drift_contract` correspondence
  tests fail until `LATEST_APP_DATA_VERSION` and the typed bounds match the
  refreshed mirrors
- keep local upstream roots out of the normal repository contract

## See Also

- [Verification](verification.md) — proof classes, the crate evidence matrix, and gates
- [Release Checklist](release-checklist.md) — packaging and release verification
- [parity/README.md](../parity/README.md) — the maintainer refresh workflow
