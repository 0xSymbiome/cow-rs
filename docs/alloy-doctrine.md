# Alloy Doctrine

## Status

Pre-1.0; binding for the v1.0 cut and forward. This is the quotable form of
posture already distributed across the ADR set and the principle-ADR map
(`.github/config/principle-adr-map.yaml`); it introduces no new policy, and
every bucket row below cites its own binding ADR.
## The three-bucket rule

Every cow-rs primitive belongs to exactly one of three buckets. **ALWAYS-ALLOY**: alloy ships the canonical maintained implementation and cow-rs delegates without re-implementation. **COW-OWNED**: cow-rs deliberately owns the logic because the contract is CoW-protocol-specific or because a binding ADR records a divergence from alloy. **BOUNDARY-ADAPTER**: cow-rs ships a runtime-neutral trait in `cow-sdk-core` plus a sibling adapter crate that wraps alloy. The bucket assignment is a property of the primitive, not of the consumer; the same primitive does not move between buckets across crates.

This rule is the operational form of three principles: Canonical Contract Bindings drives Bucket 1, Strong Typed Public Surfaces drives Bucket 2, Chain-RPC Runtime Neutrality and Explicit Runtime Boundaries drive Bucket 3.

## Decision tree

When a maintainer adds a new primitive, run the tree top-to-bottom:

1. **Does alloy ship a maintained equivalent for this primitive (anywhere in the alloy ecosystem)?**
   - Yes → step 1b.
   - No → step 4.

1b. **Which alloy family does the maintained equivalent live in?**
   - alloy-core ABI family (`alloy-primitives`, `alloy-sol-types`, `alloy-sol-macro`, `alloy-dyn-abi`, `alloy-json-abi`, `alloy-serde`) → step 2.
   - alloy-runtime family (`alloy-provider`, `alloy-signer-local`, `alloy-network`, `alloy-consensus`, `alloy-rpc-types-eth`, `alloy-transport-*`) → step 5 (**BOUNDARY-ADAPTER**). Runtime-family types are forbidden from `cow-sdk-core` and every capability crate per ADR 0026 and ADR 0052.

2. **Does cow-rs have a binding ADR that records a required divergence from alloy on this primitive's wire form, type identity, error grammar, or behavior?**
   - No → **ALWAYS-ALLOY**. Use the alloy symbol directly. No newtype, no wrapper, no parallel implementation.
   - Yes → step 3.

3. **Document the divergence cite in the call site, then classify as COW-OWNED.** The ADR-cited reason is the load-bearing fence; if a future contributor cannot articulate the cite, the divergence is suspect and a follow-up ADR review is warranted.

4. **Is the primitive CoW-protocol-specific (orderbook DTO, settlement UID, app-data CID, composable conditional order, COW Shed envelope, EIP-1271 blob shape, orderbook URL grammar, retry policy, source-lock provenance)?**
   - Yes → **COW-OWNED**. The protocol authority is upstream Solidity, upstream services, and the upstream TypeScript SDK, not alloy.
   - No → step 5.

5. **Is this a runtime-coupling concern (chain-RPC, signer creation, HTTP transport, host wallet callback)?**
   - Yes → **BOUNDARY-ADAPTER**. Define the trait in `cow-sdk-core`, ship the alloy wrap in a sibling adapter crate (`cow-sdk-alloy-*`), and keep the alloy-runtime family confined to that crate per ADR 0026.
   - No → **COW-OWNED** (last resort). Flag the primitive in the next ADR cycle so future maintainers can recognize the deliberate choice. Unflagged Bucket-2 entries that look like they could be Bucket 1 or 3 are the entropy this doctrine is designed to prevent.

## Bucket 1: ALWAYS-ALLOY

Every primitive in this table uses the alloy symbol directly. No cow-owned re-implementation exists or is permitted in shipped code.

| Surface | Alloy crate + symbol | cow-rs consumer crate(s) | ADR authority | Notes |
|---|---|---|---|---|
| Inner-layer address type | `alloy_primitives::Address` | `cow-sdk-core` (via `Address` newtype), `cow-sdk-contracts`, `cow-sdk-signing`, `cow-sdk-orderbook`, `cow-sdk-trading`, `cow-sdk-alloy*`, `cow-sdk-wasm` | ADR 0052 | `repr(transparent)` newtype, bit-for-bit layout, zero-cost conversion via `From::from` and the `as_alloy` / `into_alloy` accessors. |
| 32-byte hash | `alloy_primitives::B256` | `cow-sdk-core` (via `Hash32`, `AppDataHash`) | ADR 0052 | Two cow newtypes around `B256` preserve type-system distinction. |
| Variable bytes | `alloy_primitives::Bytes` | `cow-sdk-core` (via `HexData`) | ADR 0052 | Display/Serialize/Deserialize forward to alloy defaults. |
| Fixed-width 56-byte UID | `alloy_primitives::FixedBytes<56>` | `cow-sdk-core` (via `OrderUid`) | ADR 0052 | UID packing payload byte width is fixed by GPv2; cow owns only the 56-byte packing function (Bucket 2 below). |
| Unsigned 256-bit integer | `alloy_primitives::U256` | `cow-sdk-core` (via `Amount`) | ADR 0052 | The integer is alloy; the strict-decimal `Deserialize` is cow-owned (Bucket 2). |
| keccak256 hash | `alloy_primitives::keccak256` | `cow-sdk-contracts`, `cow-sdk-signing`, `cow-sdk-app-data` | ADR 0052 | One independent oracle (`sha3::Keccak256::digest`) is retained inside `crates/contracts/src/order.rs` as a test-only parity reference; do not collapse (see the `keccak_word` test-oracle entry in Bucket 2 below). |
| EIP-712 domain separator | `alloy_sol_types::Eip712Domain::separator()` | `cow-sdk-signing`, `cow-sdk-contracts` | ADR 0052 | Bridged from cow `TypedDataDomain` via `to_alloy_domain()` (Bucket 2). |
| EIP-712 struct signing hash | `alloy_sol_types::SolStruct::eip712_signing_hash` | `cow-sdk-contracts` | ADR 0052 | cow-shed routes through `ExecuteHooks { ... }.eip712_signing_hash(domain)` once the inner hashing has collapsed onto `SolStruct::eip712_signing_hash`. |
| EIP-191 personal-sign hash | `alloy_primitives::eip191_hash_message` | `cow-sdk-contracts`, `cow-sdk-signing` | ADR 0052 | EIP-191 prefix ownership is split per ADR 0022 (signing emits raw digest, contracts applies prefix at recovery). |
| ECDSA signature container | `alloy_primitives::Signature::from_bytes_and_parity` + `recover_address_from_prehash` recovery API | `cow-sdk-contracts::Signature::recover_ecdsa_address` | ADR 0022, ADR 0052 | The recovery surface is alloy. Strict `v ∈ {0, 1, 27, 28}` pre-validation feeds `from_bytes_and_parity`; that canonicalization is cow-owned (Bucket 2). `Signature::from_raw` is deliberately **not** used — it delegates to `normalize_v`, which would silently admit EIP-155 `v ≥ 35`; the swap is fence-banned (`ecdsa-v-normalization`; see the rationale comment in `crates/contracts/src/signature.rs`). |
| ERC-2098 compact signature | `alloy_primitives::Signature::from_erc2098` | `cow-sdk-contracts` | ADR 0052 | Used at the compact-signature ingress only. |
| CREATE2 derivation | `alloy_primitives::Address::create2` | `cow-sdk-contracts` | ADR 0052 | Replaces hand-rolled `create2`. |
| sol! ABI bindings | `alloy_sol_types::sol!` macro | `cow-sdk-contracts` (canonical home), including the off-by-default `composable` feature-module | ADR 0012, [Canonical Contract Bindings](principles.md) | Declared inline with `sol!`, mirroring upstream Solidity pinned by commit in `parity/source-lock.yaml`; the GPv2 and eth-flow bindings are proven byte-for-byte by the parity fixtures under `parity/fixtures/`, and the `ComposableCoW` and TWAP bindings by the in-crate contract tests (`PROP-CON-026`). |
| Function selector | `<MyCall as SolCall>::SELECTOR` | `cow-sdk-contracts` | ADR 0012 | Replaces hand-rolled `function_selector` helper. |
| ABI encode call | `<MyCall as SolCall>::abi_encode` | `cow-sdk-contracts`, `cow-sdk-trading` (allowance) | ADR 0012 | Hand-rolled encoders forbidden in shipped crates. |
| ABI decode returns | `<MyCall as SolCall>::abi_decode_returns` | `cow-sdk-contracts` | ADR 0012 | |
| ABI encode tuple | `alloy_sol_types::SolValue::abi_encode` and `abi_encode_sequence` | `cow-sdk-signing` (Shape B, shipped); `cow_sdk_contracts::composable` (the `conditional_order_id` preimage and TWAP static input, shipped) | ADR 0050, ADR 0052 | Shape B (forwarder) ships in `cow_sdk_signing::eip1271` via `abi_encode_sequence`; composable encodes the `ComposableCoW.hash` preimage and the 320-byte `TWAPOrder.Data` through `SolValue::abi_encode`. The Shape A EIP-1271 signature blob (the selector-prefixed Safe muxer) is produced by the watch tower, not this SDK. |
| Format decimal units | `alloy_primitives::utils::format_units` | `cow-sdk-core::Amount::format_units` | ADR 0052 | Guarded delegation: the `decimals == 0` short-circuit is preserved (avoid the literal drop-in that breaks at `Unit::MAX`). |
| Hex compile-time literal | `alloy_primitives::hex!` macro | `cow-sdk-core::config::chains` | ADR 0052 | The ten `WRAPPED_NATIVE_*_BYTES` constants are `hex!("0x...")` literals; the former `hex_decode_20` / `decode_nibble` compile-time helpers are retired. |
| Hex runtime decode/encode | `alloy_primitives::hex::{decode, encode}` | Every first-party crate under `crates/` plus `examples/native/` | ADR 0052 | The upstream `hex` crate is retired from every published crate under `crates/` and from `examples/native/`; each production and test callsite resolves through `alloy-primitives → const-hex`. The sole remaining direct `hex` dependency is in the `xtask` maintenance crate (parity registry tooling), outside the published surface. Adding a direct `hex` dep to any published crate is forbidden by this doctrine row. |
| FixedBytes parsing | `alloy_primitives::FixedBytes::<N>::from_str` | `cow-sdk-core::types::identity` | ADR 0052 | Wrapped through a cow-owned classifier (`fn classify_alloy_hex_error`) so the alloy `c: char` payload never leaks past the redaction boundary (`crates/sdk/tests/error_redaction_contract.rs`). |
| Address to 32-byte word | `alloy_primitives::Address::into_word` | `cow-sdk-contracts` (cow-shed CREATE2 address derivation) | (no ADR) | Used at `crates/contracts/src/cow_shed/address/mod.rs` to right-align the user address into the CREATE2 salt word. Elsewhere the `sol!` ABI encoders handle word layout internally. A hand-shaped `[u8; 32]` oracle is retained only in the `crates/contracts/src/order.rs` test module as an independent EIP-712 parity reference (see the `keccak_word` test-oracle entry). |
| RFC 8785 canonical JSON | `serde_jcs` (maintained crate adopted via ADR 0052) | `cow-sdk-app-data` | ADR 0052 | Replaces bytewise key-ordering canonicalisation; one documented behaviour change for non-ASCII keys (ADR 0052). |
| IMF-fixdate parsing | `httpdate::parse_http_date` | `cow_sdk_core::transport::policy::retry_after` | ADR 0052 | Drives `Retry-After` HTTP header parsing; the `parse_retry_after` *function* is cow-owned (Bucket 2) because alloy's namesake parses JSON-RPC error message strings, not the REST HTTP header. |

The principle binding is binary on every row: cow-rs does not maintain a parallel implementation in shipped crates. The one independent oracle retained in test code (`sha3::Keccak256` in `crates/contracts/src/order.rs`) is preserved on purpose so the parity test does not verify alloy's `keccak256` against itself.

## Bucket 2: COW-OWNED

Every surface in this table is shipped from cow-rs source and may not be swapped for an alloy equivalent. Each row names a binding ADR cite and a one-sentence failure mode for the wrong swap.

| Surface | cow-rs location | Why cow-owned (binding ADR) | Risk if swapped |
|---|---|---|---|
| `Amount::Deserialize` strict-decimal-only wire boundary | `crates/core/src/types/amount.rs` | ADR 0052 — alloy's `Uint::FromStr` sniffs four radices (`0x`, `0o`, `0b`, plus uppercase); cow fails closed on the wire to preserve the JSON-decimal-only contract. | `"0o755"` silently parses as 493 wei; bug invisible until off-chain ledger reconciliation. |
| `Amount::new` lenient constructor that rejects `0o`/`0b` | `crates/core/src/types/amount.rs` | ADR 0052 | Same failure mode as above — config files, env vars, and CLI flags route through the same prefix-sniffer. |
| `Amount::parse_units` reimplements decimal scaling instead of `alloy_primitives::utils::parse_units` | `crates/core/src/types/amount.rs` | ADR 0011 — the raw alloy call is unsafe for untrusted input: it is fail-OPEN (`parse_units("", d)` returns `Ok(0)`; a leading `-` routes to the `I256` arm whose `Into<U256>` returns a huge two's-complement positive), it PANICS on a non-ASCII input whose fractional-truncation byte offset lands inside a multi-byte char, and its final scaling multiply silently WRAPS over `uint256`. cow does the scaling itself with checked arithmetic (ASCII-digit grammar, `checked_mul`) and uses alloy only for the `Unit::new` decimals bound. | A blank field silently becomes zero, a negative input a near-`2^256` value, untrusted UTF-8 a panic, and an over-`uint256` magnitude a silent wrap — all bypassing the typed boundary. |
| `Address::Display` lowercase emission | `crates/core/src/types/identity.rs` | ADR 0052 — alloy default is EIP-55 mixed-case checksum; cow wire is lowercase. | Every parity fixture diffs; every string-equality tool reports a false mismatch; EIP-712 JSON-stringified payload digests drift. |
| `Address::Serialize` / `Deserialize` | `crates/core/src/types/identity.rs` | ADR 0052 | Same family; inbound deserialize accepts mixed case, outbound serialize always emits lowercase. |
| `TypedDataDomain` JSON wire shape | `crates/core/src/traits/typed_data.rs` | ADR 0052, ADR 0040 — cow shape is the EIP-1193 `eth_signTypedData_v4` wallet payload (required fields, numeric `chainId`, no `salt`); alloy `Eip712Domain` is the hashing-side type with `Option<>` everywhere, `U256` for chainId, and a `salt` field. | Every JS wallet integration fails (`null` fields, hex chainId, unexpected `salt` field). |
| ECDSA `v` byte canonicalization (`0/1 → 27/28`) | `crates/contracts/src/signature.rs` | ADR 0022 | alloy `normalize_v` collapses to a parity bit; Solidity `ecrecover(hash, v, r, s)` expects `v ∈ {27, 28}` and returns `address(0)` on `0/1`. Every smart-contract verification reverts. |
| `SupportedChainId` orderbook support-set enum + `api_path()` URL grammar | `crates/core/src/config/chains.rs` | ADR 0005 (strong domain types, supported-chain semantics), ADR 0011 typestate binding | `alloy_chains::NamedChain` covers 100+ chains and has no concept of CoW orderbook support; would silently accept chains with no backend; `GnosisChain → "xdai"` and `ArbitrumOne → "arbitrum_one"` URL mappings disappear. |
| `Amount` decimal-string `Serialize` impl | `crates/core/src/types/amount.rs` | ADR 0052 — alloy's default `Serialize` for `U256` is hex; cow wire is decimal. | Every orderbook DTO field carrying an amount flips to hex; backend rejects; reconciliation breaks. |
| `OrderUid` 56-byte packing helper (encode digest ‖ owner ‖ valid_to) | `cow-sdk-contracts::order` | GPv2 protocol contract (no ADR; CoW-protocol-specific) | The 56-byte packing is the protocol identity of an order; alloy ships no equivalent because this is cow-specific. |
| `cow_shed_eip712_domain` + `execute_hooks_signing_hash` cow-shed envelope | `crates/contracts/src/cow_shed/eip712.rs` | ADR 0049 cow-shed account-abstraction proxy | The envelope payload is GPv2/cow-shed-specific; the *hashing primitive* is alloy (Bucket 1), the *envelope identity* is cow. |
| EIP-1271 signature blob Shape A (Safe muxer, selector-prefixed) | not shipped — produced by the watch tower | ADR 0050 | Drop the selector → Safe muxer fails to dispatch → on-chain settlement reverts. |
| EIP-1271 signature blob Shape B (raw forwarder, no selector) | `cow_sdk_signing::eip1271` (`OrderAndSignature`, shipped) | ADR 0050 | Include the selector → ABI decode fails because every field offset shifts by 4. |
| `Eip1271Signer` trait | `cow_sdk_signing::eip1271` | ADR 0051 | Custom smart-account signing callback contract; not an alloy concept; placement is signing per ADR 0051 (not trading, not composable). |
| `Eip1271Cache` trait | `cow_sdk_contracts::verify` (defined), re-exported from `cow_sdk_signing::cache` | ADR 0014 | A safe-by-construction positive-only memoization boundary specific to EIP-1271 probes, keyed on `(verifier, digest, signature_hash)`; alloy ships no equivalent. Default-off, explicit-cache-arg contract is the security invariant; the SDK ships the trait and `NoopEip1271Cache`, and a consumer implements the trait to memoize. |
| `Redacted<T>` credential wrapper | `cow-sdk-core::redacted` | ADR 0025 (workspace url-redaction convention) + [Credential Redaction by Construction](principles.md) | alloy types do not redact credentials; cow's Debug/Display/Serialize/panic-path renderings must emit only sanitized identity. |
| Address registry (deployment authority) | `cow-sdk-contracts::Registry` (const table in `crates/contracts/src/deployments.rs`) | ADR 0012 | `(ContractId, DeploymentChainId, DeploymentEnv)` keyed; alloy ships no deployment authority for the CoW protocol. |
| App-data CID encoding | `cow-sdk-app-data` | (no specific ADR; cow-protocol surface) | CIDv0 / multihash for the app-data SHA-256; cow-protocol-specific. |
| Composable TWAP encoders, `conditional_order_id`, the hand-rolled `Multiplexer` merkle, and the pure `timing_at` classifier | `cow_sdk_contracts::composable` (off-by-default `composable` feature) | ADR 0048 watch-tower boundary | Ships as a feature-module of `cow-sdk-contracts`, not a separate crate. The contract-canonical merkle (double-hashed leaf, sorted-pair root) is hand-rolled because a generic merkle crate diverges from `ComposableCoW._auth`; alloy ships no merkle. Conditional-order decoders, selectors, a `PollResult` taxonomy, provider operations, a local simulator, service loops, persistence, notifications, and auto-posting stay out of scope. |
| Subgraph GraphQL transport (typed queries, request shape, schema constants) | `cow-sdk-subgraph` | ADR 0003 (separate read-only subgraph crate) | alloy ships no GraphQL transport; cow uses `HttpTransport` (`cow-sdk-core`) as the seam (Bucket 3). |
| HTTP REST transport seam (`HttpTransport` trait + `TransportError::HttpStatus` carrying headers/body) | `cow-sdk-core::transport` | ADR 0010, ADR 0013 | alloy's transport is `tower::Service<RequestPacket>` over JSON-RPC; cow's is REST; they are not type-compatible. The trait is cow-owned (here); the alloy *wrap of the alloy ecosystem* would be Bucket 3 if any — none ships, because alloy has no REST transport. |
| `parse_retry_after` for the HTTP `Retry-After` header | `crates/core/src/transport/policy/retry_after.rs` | ADR 0041 | alloy's namesake parses `"try again in 4ms"` JSON-RPC error message strings; swapping silently ignores RFC 7231 §7.1.1.1 (delta-seconds + IMF-fixdate + RFC 850). The IMF-fixdate parse itself (`httpdate::parse_http_date`) is Bucket 1; the *RFC 7231 dispatch policy* around it is Bucket 2. |
| Retry, throttle, error-classification policy | `cow_sdk_core::transport::policy` | ADR 0041, ADR 0060 | Honours `Retry-After` for 429/503, retries on `408,425,429,500,502,503,504`; not alloy's policy. |
| Browser `FetchTransport` with `AbortController` lifecycle | `crates/core/src/transport/fetch.rs` | ADR 0010 | alloy ships no browser-fetch transport; alloy transport stack would pull tokio into a `wasm32-unknown-unknown` build. |
| `JsCallbackHttpTransport` (Node/Deno/Workers callback transport) | `cow-sdk-wasm::exports::JsCallbackHttpTransport` | ADR 0010, ADR 0040 | Runtime-neutral JS callback transport; alloy ships no equivalent. |
| EIP-712 type-string whitespace contract | `crates/contracts/src/order.rs`, every type string literal in `cow-sdk-contracts`, including the `composable` feature-module | ADR 0050 | Any whitespace creep between commas in EIP-712 type strings shifts the struct hash; every signature breaks. Formatter-driven risk. |
| `keccak_word` test oracle (independent `sha3::Keccak256`) | `crates/contracts/src/order.rs` | ADR 0052 implicit | Test-only but load-bearing: collapsing to `alloy_primitives::keccak256` means the parity test verifies alloy against itself. |
| `cow-sdk-wasm` "no native alloy adapter" rule | `crates/wasm/src/` | ADR 0052, enforced by the `wasm-no-alloy-family` fence in `cargo check-source-fences` | wasm must not reference the native alloy adapter crates (`cow-sdk-alloy*`); doing so is a release-gating CI failure. It consumes alloy-core primitives (`alloy_primitives`, `alloy_sol_types`) directly for ABI and event decoding — the fence does not forbid those. |
| Source-lock provenance (upstream commit hashes for parity validation) | `parity/source-lock.yaml` (the single per-repository commit pin behind every `cow-sdk-contracts::Registry` address) | [Evidence-Backed Public Claims](principles.md), ADR 0026, ADR 0030, ADR 0032 | alloy ships no provenance authority; cow's release evidence is repository-visible. |

## Bucket 3: BOUNDARY-ADAPTER

Each entry defines a cow-owned trait in `cow-sdk-core` and ships a separate adapter crate that wraps the alloy runtime. The trait lives in `cow-sdk-core`; the alloy-runtime family (`alloy-provider`, `alloy-signer-local`, `alloy-network`, `alloy-consensus`, `alloy-rpc-types-eth`, `alloy-transport-*`) is forbidden from `cow-sdk-core` and from every capability crate per ADR 0026 and ADR 0052.

| Trait | cow-rs trait file | Adapter crate | Alloy types wrapped | ADR authority |
|---|---|---|---|---|
| `Provider` (read-only chain RPC) | `cow_sdk_core::Provider` | `cow-sdk-alloy-provider` (native, read-only); on `wasm32`, the host wallet's EIP-1193 provider is reached through the `cow-sdk-wasm` contract-read callback | `alloy_provider::DynProvider<Ethereum>`, transport via `reqwest`, redacted URL via `Redacted<reqwest::Url>` | ADR 0024, ADR 0035 |
| `SigningProvider: Provider` (signer creation extension) | `cow_sdk_core::SigningProvider` | `cow-sdk-alloy` (composed read+sign) | `alloy_provider::DynProvider<Ethereum>` with wallet filler | ADR 0024, ADR 0035 |
| `Signer` (EIP-191 + EIP-712 signing) | `cow_sdk_core::Signer` | `cow-sdk-alloy-signer` (native local private-key); `cow-sdk-alloy::AlloyClientSignerHandle` (composed) | `alloy_signer_local::PrivateKeySigner`, `alloy_signer::Signer` | ADR 0024, ADR 0035, ADR 0045 |
| Narrow capability traits (`TypedDataSigner`, `DigestSigner`) | `cow_sdk_core::{TypedDataSigner, DigestSigner}` | Callback-shaped adapters (`cow-sdk-wasm`) that expose a single signing operation | n/a — these are cow-owned shapes; alloy ships no peer | ADR 0024, ADR 0045 |
| `HttpTransport` (REST/GraphQL) | `cow_sdk_core::HttpTransport` | `ReqwestTransport` and `FetchTransport`, both target-gated inside `cow-sdk-core`; `cow_sdk_wasm::exports::JsCallbackHttpTransport` | `reqwest::Client` for native; `web_sys` global `fetch` for browser; JS callback for Node/Deno/Workers | ADR 0010, ADR 0013 |
| `IpfsFetchTransport` | `cow_sdk_app_data::IpfsFetchTransport` (re-exported via `cow-sdk-core` cancellation contract) | `cow-sdk-app-data` native + browser variants | Same underlying transports as `HttpTransport`; the CID-fetch policy is cow-owned | ADR 0010 (cancellation extension to IPFS fetch) |
| Wallet/provider/signer JS callback boundary | `cow_sdk_wasm` typed callbacks (`TypedDataSignerCallback`, `DigestSignerCallback`, `CustomEip1271Callback`, `ContractReadCallback`, `CowFetchCallback`) | `cow-sdk-wasm` | Wallet/provider semantics owned by the host JS — an EIP-1193 provider wraps into the typed-data callback — not by Rust types | ADR 0040, ADR 0045 |
| Transaction lifecycle types (`TransactionBroadcast`, `TransactionReceipt`) | `cow_sdk_core::transaction` | Implemented by `cow-sdk-alloy-provider`, `cow-sdk-alloy`, any custom adapter | `alloy_rpc_types_eth::TransactionReceipt` | ADR 0038 |

The trait owns the public contract; the adapter is replaceable. A future post-alloy provider stack (e.g. a hypothetical `cow-sdk-foundry-provider`) would ship as a peer adapter implementing the same `Provider` trait, without touching `cow-sdk-trading`, `cow-sdk-orderbook`, `cow-sdk-signing`, or the default facade. This is the operational form of [Chain-RPC Runtime Neutrality](principles.md).

## Applying the tree

The tree is runnable as-is. Two traces that show the non-obvious verdicts:

- **Add a chain (e.g. Mantle).** `alloy_chains::NamedChain` exists (Step 1b →
  alloy-core → Step 2), but the `SupportedChainId` never-swap entry and ADR 0005
  forbid the swap — `alloy_chains` knows nothing of CoW orderbook support or the
  `api_path()` URL grammar. **COW-OWNED**: add a `#[non_exhaustive]`
  `SupportedChainId` variant, an `api_path()` arm, and a `WRAPPED_NATIVE_*` `hex!`
  constant; the `Registry` CREATE2 singletons resolve unchanged. Do not add
  `alloy-chains` (the `check-alloy-family-pins` policy rejects it).
- **Add a wallet provider (e.g. Frame).** Not protocol-specific (Step 4 → no),
  runtime-coupling (Step 5 → yes): **BOUNDARY-ADAPTER**. An EIP-1193 wallet routes
  through the `TypedDataSignerCallback` — the host wraps its provider's
  `eth_signTypedData_v4` (ADR 0040) — with zero Rust changes; wallet identity
  never becomes an SDK-owned Rust type; the host application wires it in JS-side.

## Traceability and evolution

Every Bucket 2/3 row resolves to a binding ADR cite; the principle-ADR map
(`.github/config/principle-adr-map.yaml`) is the traceability anchor, and each
bucket ADR anchors at least one `docs/audit/` document. The doctrine is a
read-only consolidation: when an ADR changes, update the matching bucket row, the
principle map, and the audit-refresh map (`.github/config/audit-refresh-map.yml`)
in step. A new bucket entry that lands without an ADR cite is suspect and owes a
follow-up ADR.

## Enforcement

Each Bucket 2 divergence is mechanized by a source guard in
`xtask/src/policy/fences.rs` (run via `cargo check-source-fences`) that rejects
the forbidden alloy symbol on the protected surface; the most security-sensitive
call sites also carry a short rationale comment naming the ADR. Allow-list policy
confines the alloy-runtime family
(`alloy-provider`, `alloy-signer-local`, `alloy-network`, `alloy-consensus`,
`alloy-rpc-types-eth`, `alloy-transport-*`) to the three adapter crates
(`cow-sdk-alloy-provider`, `cow-sdk-alloy-signer`, `cow-sdk-alloy`) per ADR 0026
and ADR 0052; `Cargo.lock` resolves each alloy crate to one version; and no
`cargo-semver-checks` lane runs pre-1.0 (ADR 0030). The lint layer is the
mechanical floor; the doctrine explains why each gate exists.

A primitive outside the shipped or planned surface — a non-EVM runtime, a
post-EIP-1193 wallet shape, an alloy crate split or yank — needs a new ADR before
the doctrine can classify it. Until then the tree's Step 5 "last-resort
COW-OWNED, flag for the next ADR cycle" rule governs.

---

**This document answers "when does cow-rs use alloy, when does it own logic, and
when does it route through an adapter."** Following the decision tree and the
per-row ADR cites lands on the same answer the ADR set already records.
