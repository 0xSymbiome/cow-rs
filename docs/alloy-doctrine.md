# Alloy Doctrine

## Status

- Pre-1.0; binding for the v1.0 cut and forward.
- Codifies posture already accepted across the ADR set; introduces no new policy.
- Anchored by: ADR 0010, ADR 0011, ADR 0012, ADR 0014, ADR 0019, ADR 0022, ADR 0024, ADR 0026, ADR 0029, ADR 0035, ADR 0036, ADR 0037, ADR 0040, ADR 0044, ADR 0045, ADR 0050, ADR 0051, ADR 0052.
- Supporting principle layer: [Canonical Contract Bindings](principles.md), [Chain-RPC Runtime Neutrality](principles.md), [Explicit Runtime Boundaries](principles.md), [Strong Typed Public Surfaces](principles.md), [Forward-Compatible Public Surfaces](principles.md).
- Supersedes: nothing. This document is the quotable form of doctrine already distributed across the ADR set and the principle-ADR map at `.github/config/principle-adr-map.yaml`.

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

5. **Is this a runtime-coupling concern (chain-RPC, signer creation, HTTP transport, browser-wallet session)?**
   - Yes → **BOUNDARY-ADAPTER**. Define the trait in `cow-sdk-core`, ship the alloy wrap in a sibling adapter crate (`cow-sdk-alloy-*`), and keep the alloy-runtime family confined to that crate per ADR 0026.
   - No → **COW-OWNED** (last resort). Flag the primitive in the next ADR cycle so future maintainers can recognize the deliberate choice. Unflagged Bucket-2 entries that look like they could be Bucket 1 or 3 are the entropy this doctrine is designed to prevent.

## Bucket 1: ALWAYS-ALLOY

Every primitive in this table uses the alloy symbol directly. No cow-owned re-implementation exists or is permitted in shipped code.

| Surface | Alloy crate + symbol | cow-rs consumer crate(s) | ADR authority | Notes |
|---|---|---|---|---|
| Inner-layer address type | `alloy_primitives::Address` | `cow-sdk-core` (via `Address` newtype), `cow-sdk-contracts`, `cow-sdk-signing`, `cow-sdk-orderbook`, `cow-sdk-trading`, `cow-sdk-alloy*`, `cow-sdk-cow-shed`, `cow-sdk-browser-wallet` | ADR 0052 | `repr(transparent)` newtype, bit-for-bit layout, zero-cost conversion via `From::from` and the `as_alloy` / `into_alloy` accessors. |
| 32-byte hash | `alloy_primitives::B256` | `cow-sdk-core` (via `Hash32`, `AppDataHash`) | ADR 0052 | Two cow newtypes around `B256` preserve type-system distinction. |
| Variable bytes | `alloy_primitives::Bytes` | `cow-sdk-core` (via `HexData`) | ADR 0052 | Display/Serialize/Deserialize forward to alloy defaults. |
| Fixed-width 56-byte UID | `alloy_primitives::FixedBytes<56>` | `cow-sdk-core` (via `OrderUid`) | ADR 0052 | UID packing payload byte width is fixed by GPv2; cow owns only the 56-byte packing function (Bucket 2 below). |
| Unsigned 256-bit integer | `alloy_primitives::U256` | `cow-sdk-core` (via `Amount`) | ADR 0052 | The integer is alloy; the strict-decimal `Deserialize` is cow-owned (Bucket 2). |
| Signed 256-bit integer | `alloy_primitives::I256` | `cow-sdk-core` (via `SignedAmount`) | ADR 0052 | Same split as `Amount`. |
| keccak256 hash | `alloy_primitives::keccak256` | `cow-sdk-contracts`, `cow-sdk-signing`, `cow-sdk-app-data`, `cow-sdk-cow-shed` | ADR 0052 | One independent oracle (`sha3::Keccak256::digest`) is retained inside `crates/contracts/src/order/hash.rs` as a test-only parity reference; do not collapse (see the `keccak_word` test-oracle entry in Bucket 2 below). |
| EIP-712 domain separator | `alloy_sol_types::Eip712Domain::separator()` | `cow-sdk-signing`, `cow-sdk-contracts`, `cow-sdk-cow-shed` | ADR 0052 | Bridged from cow `TypedDataDomain` via `into_alloy_domain()` (Bucket 2). |
| EIP-712 struct signing hash | `alloy_sol_types::SolStruct::eip712_signing_hash` | `cow-sdk-contracts`, `cow-sdk-cow-shed`, `cow-sdk-composable` (deferred per ADR 0050) | ADR 0052 | cow-shed routes through `ExecuteHooks { ... }.eip712_signing_hash(domain)` once the inner hashing has collapsed onto `SolStruct::eip712_signing_hash`. |
| EIP-191 personal-sign hash | `alloy_primitives::eip191_hash_message` | `cow-sdk-contracts`, `cow-sdk-signing` | ADR 0052 | EIP-191 prefix ownership is split per ADR 0022 (signing emits raw digest, contracts applies prefix at recovery). |
| ECDSA signature container | `alloy_primitives::Signature::from_raw` + secp256k1 recovery API | `cow-sdk-contracts::Signature::recover_ecdsa_address` | ADR 0022, ADR 0052 | The recovery surface is alloy. The `v ∈ {27, 28}` byte canonicalization that wraps it is cow-owned (Bucket 2). |
| ERC-2098 compact signature | `alloy_primitives::Signature::from_erc2098` | `cow-sdk-contracts` | ADR 0052 | Used at the compact-signature ingress only. |
| CREATE2 derivation | `alloy_primitives::Address::create2` | `cow-sdk-cow-shed`, `cow-sdk-contracts` | ADR 0052 | Replaces hand-rolled `create2`. |
| sol! ABI bindings | `alloy_sol_types::sol!` macro | `cow-sdk-contracts` (canonical home), `cow-sdk-cow-shed`, `cow-sdk-composable` (deferred) | ADR 0012, [Canonical Contract Bindings](principles.md) | Solidity excerpts committed under `crates/contracts/abi/<family>/` for provenance. |
| Function selector | `<MyCall as SolCall>::SELECTOR` | `cow-sdk-contracts` | ADR 0012 | Replaces hand-rolled `function_selector` helper. |
| ABI encode call | `<MyCall as SolCall>::abi_encode` | `cow-sdk-contracts`, `cow-sdk-trading` (allowance), `cow-sdk-cow-shed` | ADR 0012 | Hand-rolled encoders forbidden in shipped crates. |
| ABI decode returns | `<MyCall as SolCall>::abi_decode_returns` | `cow-sdk-contracts`, `cow-sdk-cow-shed` | ADR 0012 | |
| ABI encode tuple | `alloy_sol_types::SolValue::abi_encode` and `abi_encode_sequence` | `cow-sdk-composable` (deferred, ADR 0050) | ADR 0050, ADR 0052 | Shape B (forwarder) emits a tuple via this surface; Shape A (Safe muxer) uses `SolCall::abi_encode` because the selector prefix is load-bearing. |
| Format decimal units | `alloy_primitives::utils::format_units` | `cow-sdk-core::Amount::format_units` | ADR 0052 | Guarded delegation: the `decimals == 0` short-circuit is preserved (avoid the literal drop-in that breaks at `Unit::MAX`). |
| Hex compile-time literal | `alloy_primitives::hex!` macro | `cow-sdk-core::config::chains` | ADR 0052 | Replaces `hex_decode_20` + `decode_nibble` panic-allowlisted helpers; ten `WRAPPED_NATIVE_*_BYTES` constants flip to `hex!("0x...")`. |
| Hex runtime decode/encode | `alloy_primitives::hex::{decode, encode}` | Every first-party crate under `crates/` plus `examples/native/` | ADR 0052 | The upstream `hex` crate is fully retired from the workspace dependency graph. Every production and test callsite resolves through `alloy-primitives → const-hex`. Adding a direct `hex` dep to any new crate is forbidden by this doctrine row. |
| FixedBytes parsing | `alloy_primitives::FixedBytes::<N>::from_str` | `cow-sdk-core::types::identity` | ADR 0052 | Wrapped through a cow-owned classifier (`fn classify_alloy_hex_error`) so the alloy `c: char` payload never leaks past the redaction boundary (`crates/sdk/tests/error_redaction_contract.rs`). |
| Address to 32-byte word | `alloy_primitives::Address::into_word` | NOT used directly — cow keeps `encode_address_word` | (cosmetic, no ADR) | Kept as a `[u8; 32]`-shaped helper because the production callsite at `crates/trading/src/allowance.rs:190` already routes through the cow helper; replacing it with `Address::into_word` is cosmetic at best. |
| Multiplexer merkle proofs | `rs_merkle` (maintained crate adopted via ADR 0052) | `cow-sdk-composable` (deferred) | ADR 0052 | Replaces hand-rolled Multiplexer merkle machinery. |
| RFC 8785 canonical JSON | `serde_jcs` (maintained crate adopted via ADR 0052) | `cow-sdk-app-data` | ADR 0052 | Replaces bytewise key-ordering canonicalisation; one documented behaviour change for non-ASCII keys (ADR 0052). |
| IMF-fixdate parsing | `httpdate::parse_http_date` | `cow-sdk-transport-policy::retry_after` | ADR 0052 | Drives `Retry-After` HTTP header parsing; the `parse_retry_after` *function* is cow-owned (Bucket 2) because alloy's namesake parses JSON-RPC error message strings, not the REST HTTP header. |

The principle binding is binary on every row: cow-rs does not maintain a parallel implementation in shipped crates. The one independent oracle retained in test code (`sha3::Keccak256` in `crates/contracts/src/order/hash.rs`) is preserved on purpose so the parity test does not verify alloy's `keccak256` against itself.

## Bucket 2: COW-OWNED

Every surface in this table is shipped from cow-rs source and may not be swapped for an alloy equivalent. Each row names a binding ADR cite and a one-sentence failure mode for the wrong swap.

| Surface | cow-rs location | Why cow-owned (binding ADR) | Risk if swapped |
|---|---|---|---|
| `Amount::Deserialize` strict-decimal-only wire boundary | `crates/core/src/types/amount.rs` | ADR 0052 — alloy's `Uint::FromStr` sniffs four radices (`0x`, `0o`, `0b`, plus uppercase); cow fails closed on the wire to preserve the JSON-decimal-only contract. | `"0o755"` silently parses as 493 wei; bug invisible until off-chain ledger reconciliation. |
| `Amount::new` / `SignedAmount::new` lenient constructors that reject `0o`/`0b` | `crates/core/src/types/amount.rs` | ADR 0052 | Same failure mode as above — config files, env vars, and CLI flags route through the same prefix-sniffer. |
| `Amount::parse_units` reimplements decimal scaling instead of `alloy_primitives::utils::parse_units` | `crates/core/src/types/amount.rs` | ADR 0011 — the raw alloy call is unsafe for untrusted input: it is fail-OPEN (`parse_units("", d)` returns `Ok(0)`; a leading `-` routes to the `I256` arm whose `Into<U256>` returns a huge two's-complement positive), it PANICS on a non-ASCII input whose fractional-truncation byte offset lands inside a multi-byte char, and its final scaling multiply silently WRAPS over `uint256`. cow does the scaling itself with checked arithmetic (ASCII-digit grammar, `checked_mul`) and uses alloy only for the `Unit::new` decimals bound. | A blank field silently becomes zero, a negative input a near-`2^256` value, untrusted UTF-8 a panic, and an over-`uint256` magnitude a silent wrap — all bypassing the typed boundary. |
| `Address::Display` lowercase emission | `crates/core/src/types/identity.rs` | ADR 0052 — alloy default is EIP-55 mixed-case checksum; cow wire is lowercase. | Every parity fixture diffs; every string-equality tool reports a false mismatch; EIP-712 JSON-stringified payload digests drift. |
| `Address::Serialize` / `Deserialize` | `crates/core/src/types/identity.rs` | ADR 0052 | Same family; inbound deserialize accepts mixed case, outbound serialize always emits lowercase. |
| `TypedDataDomain` JSON wire shape | `crates/core/src/traits/typed_data.rs` | ADR 0052, ADR 0040 — cow shape is the EIP-1193 `eth_signTypedData_v4` wallet payload (required fields, numeric `chainId`, no `salt`); alloy `Eip712Domain` is the hashing-side type with `Option<>` everywhere, `U256` for chainId, and a `salt` field. | Every JS wallet integration fails (`null` fields, hex chainId, unexpected `salt` field). |
| ECDSA `v` byte canonicalization (`0/1 → 27/28`) | `crates/contracts/src/signature.rs` | ADR 0022 | alloy `normalize_v` collapses to a parity bit; Solidity `ecrecover(hash, v, r, s)` expects `v ∈ {27, 28}` and returns `address(0)` on `0/1`. Every smart-contract verification reverts. |
| `SupportedChainId` orderbook support-set enum + `api_path()` URL grammar | `crates/core/src/config/chains.rs` | ADR 0005 (strong domain types, supported-chain semantics), ADR 0011 typestate binding | `alloy_chains::NamedChain` covers 100+ chains and has no concept of CoW orderbook support; would silently accept chains with no backend; `GnosisChain → "xdai"` and `ArbitrumOne → "arbitrum_one"` URL mappings disappear. |
| `hex_decode_20` `const fn` (compile-time wrapped-native-token decoder) | `crates/core/src/config/chains.rs` | ADR 0052 implicit | After the compile-time `hex!` macro adoption lands, this row migrates: hex literals are Bucket 1, the panic-allowlist entry retires, and there is no surface left to "swap". |
| `Amount` decimal-string `Serialize` impl | `crates/core/src/types/amount.rs` | ADR 0052 — alloy's default `Serialize` for `U256` is hex; cow wire is decimal. | Every orderbook DTO field carrying an amount flips to hex; backend rejects; reconciliation breaks. |
| `OrderUid` 56-byte packing helper (encode digest ‖ owner ‖ valid_to) | `cow-sdk-contracts::order::uid` | GPv2 protocol contract (no ADR; CoW-protocol-specific) | The 56-byte packing is the protocol identity of an order; alloy ships no equivalent because this is cow-specific. |
| `cow_shed_eip712_domain` + `execute_hooks_signing_hash` cow-shed envelope | `crates/cow-shed/src/eip712/` | ADR 0049 cow-shed account-abstraction proxy | The envelope payload is GPv2/cow-shed-specific; the *hashing primitive* is alloy (Bucket 1), the *envelope identity* is cow. |
| EIP-1271 signature blob Shape A (Safe muxer, selector-prefixed) | `cow-sdk-composable` (deferred per ADR 0050) | ADR 0050 | Drop the selector → Safe muxer fails to dispatch → on-chain settlement reverts. |
| EIP-1271 signature blob Shape B (raw forwarder, no selector) | `cow-sdk-composable` (deferred) | ADR 0050 | Include the selector → ABI decode fails because every field offset shifts by 4. |
| `Eip1271SignatureProvider` trait | `cow_sdk_signing::eip1271` | ADR 0051 | Custom smart-account signing callback contract; not an alloy concept; placement is signing per ADR 0051 (not trading, not composable). |
| `Eip1271VerificationCache` trait | `cow_sdk_contracts::verify` (defined), re-exported from `cow_sdk_signing::cache` | ADR 0014 | A safe-by-construction positive-only memoization boundary specific to EIP-1271 probes, keyed on `(verifier, digest, signature_hash)`; alloy ships no equivalent. Default-off, explicit-cache-arg contract is the security invariant; the in-memory impl is gated behind the `in-memory-cache` feature. |
| `Redacted<T>` credential wrapper | `cow-sdk-core::redacted` | ADR 0025 (workspace url-redaction convention) + [Credential Redaction by Construction](principles.md) | alloy types do not redact credentials; cow's Debug/Display/Serialize/panic-path renderings must emit only sanitized identity. |
| Address registry (deployment authority) | `cow-sdk-contracts::Registry` (`crates/contracts/registry.toml`) | ADR 0012 | `(ContractId, SupportedChainId, CowEnv)` keyed; alloy ships no deployment authority for the CoW protocol. |
| App-data CID encoding | `cow-sdk-app-data` | (no specific ADR; cow-protocol surface) | CIDv0 / multihash for the app-data SHA-256; cow-protocol-specific. |
| Composable conditional order framework (encoders, decoders, selectors, `PollResult` taxonomy, single-call provider operations, local simulator) | `cow-sdk-composable` (deferred per ADR 0048) | ADR 0048 watch-tower boundary | Service loops, persistence, notifications, auto-posting are explicitly out of scope; alloy ships no composable surface. |
| Subgraph GraphQL transport (typed queries, request shape, schema constants) | `cow-sdk-subgraph` | ADR 0003 (separate read-only subgraph crate) | alloy ships no GraphQL transport; cow uses `HttpTransport` (`cow-sdk-core`) as the seam (Bucket 3). |
| HTTP REST transport seam (`HttpTransport` trait + `TransportError::HttpStatus` carrying headers/body) | `cow-sdk-core::transport` | ADR 0010, ADR 0019 | alloy's transport is `tower::Service<RequestPacket>` over JSON-RPC; cow's is REST; they are not type-compatible. The trait is cow-owned (here); the alloy *wrap of the alloy ecosystem* would be Bucket 3 if any — none ships, because alloy has no REST transport. |
| `parse_retry_after` for the HTTP `Retry-After` header | `crates/transport-policy/src/retry_after.rs` | ADR 0041 | alloy's namesake parses `"try again in 4ms"` JSON-RPC error message strings; swapping silently ignores RFC 7231 §7.1.1.1 (delta-seconds + IMF-fixdate + RFC 850). The IMF-fixdate parse itself (`httpdate::parse_http_date`) is Bucket 1; the *RFC 7231 dispatch policy* around it is Bucket 2. |
| Retry, throttle, error-classification policy | `cow-sdk-transport-policy` | ADR 0041, ADR 0046 | Honours `Retry-After` for 429/503, retries on `408,425,429,500,502,503,504`; not alloy's policy. |
| Browser `FetchTransport` with `AbortController` lifecycle | `crates/transport-wasm/src/fetch.rs` | ADR 0010 | alloy ships no browser-fetch transport; alloy transport stack would pull tokio into a `wasm32-unknown-unknown` build. |
| `JsCallbackHttpTransport` (Node/Deno/Workers callback transport) | `cow-sdk-wasm::exports::JsCallbackHttpTransport` | ADR 0010, ADR 0040 | Runtime-neutral JS callback transport; alloy ships no equivalent. |
| EIP-712 type-string whitespace contract | `crates/contracts/src/order/hash.rs`, every type string literal in `cow-sdk-cow-shed` and (future) `cow-sdk-composable` | ADR 0050 | Any whitespace creep between commas in EIP-712 type strings shifts the struct hash; every signature breaks. Formatter-driven risk. |
| `keccak_word` test oracle (independent `sha3::Keccak256`) | `crates/contracts/src/order/hash.rs` | ADR 0052 implicit | Test-only but load-bearing: collapsing to `alloy_primitives::keccak256` means the parity test verifies alloy against itself. |
| `cow-sdk-wasm` "no direct alloy imports" rule | `crates/wasm/src/` | ADR 0052, enforced by `.github/workflows/wasm-imports-grep-gate.yml` | wasm leaf consumes alloy types via re-exports from `cow-sdk-contracts` and `cow-sdk-pure-helpers`; direct imports are a release-gating CI failure. |
| Source-lock provenance (upstream commit hashes for parity validation) | `parity/source-lock.yaml`, `crates/contracts/deployment-provenance.yaml` | [Evidence-Backed Public Claims](principles.md), ADR 0026, ADR 0030, ADR 0032 | alloy ships no provenance authority; cow's release evidence is repository-visible. |

## Bucket 3: BOUNDARY-ADAPTER

Each entry defines a cow-owned trait in `cow-sdk-core` and ships a separate adapter crate that wraps the alloy runtime. The trait lives in `cow-sdk-core`; the alloy-runtime family (`alloy-provider`, `alloy-signer-local`, `alloy-network`, `alloy-consensus`, `alloy-rpc-types-eth`, `alloy-transport-*`) is forbidden from `cow-sdk-core` and from every capability crate per ADR 0026 and ADR 0052.

| Trait | cow-rs trait file | Adapter crate | Alloy types wrapped | ADR authority |
|---|---|---|---|---|
| `Provider` (read-only chain RPC) | `cow_sdk_core::Provider` | `cow-sdk-alloy-provider` (native, read-only); browser-wallet leaf provides an EIP-1193 impl on `wasm32` | `alloy_provider::DynProvider<Ethereum>`, transport via `reqwest`, redacted URL via `Redacted<reqwest::Url>` | ADR 0024, ADR 0035 |
| `SigningProvider: Provider` (signer creation extension) | `cow_sdk_core::SigningProvider` | `cow-sdk-alloy` (composed read+sign) | `alloy_provider::DynProvider<Ethereum>` with wallet filler | ADR 0024, ADR 0037 |
| `Signer` (EIP-191 + EIP-712 signing) | `cow_sdk_core::Signer` | `cow-sdk-alloy-signer` (native local keystore); `cow-sdk-alloy::AlloyClientSignerHandle` (composed) | `alloy_signer_local::PrivateKeySigner`, `alloy_signer::Signer` | ADR 0024, ADR 0036, ADR 0045 |
| Narrow capability traits (`Owner`, `TypedDataSigner`, `DigestSigner`, `Eip1193`) | `cow_sdk_core::{Owner, TypedDataSigner, DigestSigner, Eip1193}` | Callback-shaped adapters (`cow-sdk-browser-wallet`, `cow-sdk-wasm`) that expose a single signing operation | n/a — these are cow-owned shapes; alloy ships no peer | ADR 0024, ADR 0029, ADR 0045 |
| `HttpTransport` (REST/GraphQL) | `cow_sdk_core::HttpTransport` | `ReqwestTransport` (target-gated inside `cow-sdk-core`); `cow_sdk_transport_wasm::FetchTransport`; `cow_sdk_wasm::exports::JsCallbackHttpTransport` | `reqwest::Client` for native; `web_sys::Fetch` for browser; JS callback for Node/Deno/Workers | ADR 0010, ADR 0013, ADR 0019 |
| `IpfsFetchTransport` | `cow_sdk_app_data::IpfsFetchTransport` (re-exported via `cow-sdk-core` cancellation contract) | `cow-sdk-app-data` native + browser variants | Same underlying transports as `HttpTransport`; the CID-fetch policy is cow-owned | ADR 0010 (cancellation extension to IPFS fetch) |
| Wallet/provider/signer JS callback boundary | `cow_sdk_wasm` typed callbacks (`TypedDataSignerCallback`, `Eip1193RequestCallback`, `DigestSignerCallback`, `CustomEip1271Callback`, `CowFetchCallback`) | `cow-sdk-browser-wallet`, `cow-sdk-wasm` | EIP-1193 provider request semantics owned by the host JS, not by Rust types | ADR 0040, ADR 0045, ADR 0047 |
| Transaction lifecycle types (`TransactionBroadcast`, `TransactionReceipt`) | `cow_sdk_core::transaction` | Implemented by `cow-sdk-alloy-provider`, `cow-sdk-alloy`, `cow-sdk-browser-wallet`, any custom adapter | `alloy_rpc_types_eth::TransactionReceipt` | ADR 0038 |

The trait owns the public contract; the adapter is replaceable. A future post-alloy provider stack (e.g. a hypothetical `cow-sdk-foundry-provider`) would ship as a peer adapter implementing the same `Provider` trait, without touching `cow-sdk-trading`, `cow-sdk-orderbook`, `cow-sdk-signing`, or the default facade. This is the operational form of [Chain-RPC Runtime Neutrality](principles.md).

## How to apply the doctrine to a new primitive

The decision tree is meant to be runnable. The following worked examples walk it end-to-end for primitives a future maintainer is likely to be asked about.

### Example 1 — EIP-2930 access lists

**Step 1:** Does alloy-core ship a maintained equivalent? Yes — `alloy_rpc_types_eth::AccessList` and `alloy_consensus::TxEip2930` exist in the alloy-runtime family.

**Step 1b:** `alloy-rpc-types-eth` and `alloy-consensus` are alloy-runtime family per the runbook list — route to Step 5.

**Step 5:** Is this runtime-coupling? Yes — access lists are a transaction-shape concern that only matters at signer/provider submission time.

**Verdict: BOUNDARY-ADAPTER.** The access-list type stays inside `cow-sdk-alloy*`; the public seam exposes an SDK-owned `TransactionRequest`-style type if needed (ADR 0026 prohibits leaking concrete alloy types into the facade). Per ADR 0038, the broadcast/receipt boundary is already enforced; an access-list extension lives at the same adapter layer.

### Example 2 — Adding a new chain (Linea is already on the list; treat as "add Mantle")

**Step 1:** Does alloy ship a `NamedChain::Mantle`? Probably yes in `alloy_chains`.

**Step 1b:** `alloy-chains` is the alloy-core ABI family — route to Step 2.

**Step 2:** Is there a binding ADR that prohibits swapping to `alloy_chains`? Yes — the `SupportedChainId` never-swap entry in Bucket 2 and the ADR 0005 strong-domain-type rule prohibit substituting `alloy_chains::NamedChain` for `SupportedChainId`. The `api_path()` URL grammar is cow-specific.

**Step 3:** Classify as COW-OWNED. Add a `Mantle` variant to `SupportedChainId` under the `#[non_exhaustive]` carve-out ([Forward-Compatible Public Surfaces](principles.md) / ADR 0031). Add an `api_path()` arm that matches the CoW orderbook URL segment. Add a `WRAPPED_NATIVE_MANTLE_BYTES` constant using `hex!` (Bucket 1). Add a `Registry` row in `crates/contracts/registry.toml` for every contract id deployed on Mantle (Bucket 2 — registry is cow-owned per ADR 0012).

**Do not** add `alloy-chains` as a workspace dependency. The `alloy-chains` workspace-dep ban CI gate at `.github/workflows/never-swap-gates.yml` catches the import.

### Example 3 — A new wallet provider (Frame)

**Step 1:** Does alloy ship a Frame adapter? No.

**Step 4:** Is Frame CoW-protocol-specific? No.

**Step 5:** Is this runtime-coupling? Yes — Frame is an EIP-1193 provider implementation. **Verdict: BOUNDARY-ADAPTER.** Frame exposes `eth_signTypedData_v4` and `eth_sendTransaction` like any EIP-1193 wallet. Routing through the existing `Eip1193RequestCallback` / `TypedDataSignerCallback` (ADR 0040) requires zero Rust changes — Frame is a JS-side concern that the host application wires into the typed callback boundary. No new trait, no new adapter crate, no SDK release.

The cow-rs rule is that EIP-1193 wallet identity does not become an SDK-owned Rust type (ADR 0040). The Rust SDK owns timeout, error typing, recovery-byte normalization, and redaction; the wallet ecosystem is JS-side.

### Example 4 — EIP-4844 blob transactions

**Step 1:** alloy ships `alloy_consensus::TxEip4844` and friends.

**Step 1b:** `alloy-consensus` is alloy-runtime family — route to Step 5.

**Step 5:** Runtime-coupling? Yes (transaction shape + KZG commitment).

**Verdict: BOUNDARY-ADAPTER.** The blob-transaction concept is wholly confined to `cow-sdk-alloy*` (alloy-runtime family). If the protocol ever signs over a blob payload (it currently does not — CoW orders are not blob transactions), an EIP-712 wire shape would land as Bucket 2 (cow-owned envelope) wrapping a Bucket 1 hashing primitive.

In the meantime, do not pull `c-kzg` or alloy blob types into `cow-sdk-core`, `cow-sdk-contracts`, `cow-sdk-signing`, or any capability crate. The blob-transaction adapter is a future capability behind a feature flag on `cow-sdk-alloy*` if ever needed.

### Example 5 — Post-quantum signing (ADR 0027 has the absorption plan)

**Step 1:** Does alloy ship a maintained PQ signer? No. EIP-7212 secp256r1 verification has alloy interest but no stable cow-relevant surface.

**Step 4:** Is this CoW-protocol-specific? Yes — at the wire layer the new signing scheme lands as a `SigningScheme` variant under `#[non_exhaustive]` (ADR 0027), gated by upstream cowprotocol wire-contract definition.

**Verdict: COW-OWNED** (Bucket 2). Per ADR 0027, post-quantum support enters through additive non-exhaustive `Signature` / `SigningScheme` variants with scheme-keyed normalization. The ECDSA `27/28` normalizer (Bucket 2) stays untouched; the new scheme gets its own normalization path; verifier-only schemes route through the existing EIP-1271 path until the protocol gains a dedicated typed variant.

If alloy later ships a maintained PQ signer crate that matches the cow-protocol wire form, classification moves to Bucket 1 for the primitive plus Bucket 2 for the wire-form rejection contract (same split as `Amount`).

### Example 6 — alloy 3.0 changes the U256 API

**Step 0:** This is the alloy-major-release runbook scenario ([Alloy Major Release Runbook](alloy-major-release-runbook.md) per ADR 0026).

**Trigger:** The canary lane (ADR 0026, configurable workflow) flags an upstream change against the pinned alloy refs. The canary runs in informational mode; promotion to PR-blocking requires an explicit policy change.

**Tree application:** Re-run the doctrine across every Bucket 1 row that touches the changed surface. Every cow-named newtype around `U256` is `repr(transparent)` with bit-for-bit layout (ADR 0052), so source-level breaks land at:
- the `From`/`Into` conversion bodies in `cow-sdk-core::types::amount`,
- the `as_u256` / `into_u256` accessors per the ADR 0011 amendment,
- the cow-owned `Display`/`Serialize`/`Deserialize` impls that delegate to alloy internals,
- the cow-owned `Add`/`Sub`/`Mul`/`AddAssign` operator overloads that delegate to the inner integer.

ADR 0026 binds: "Letting alloy types leak into the stable SDK facade would turn those migrations into consumer-facing semver breaks." So the cow newtype layer is the absorption surface; the public facade is unchanged; downstream consumers see no breaking change unless cow chooses to widen the public surface at the same time.

The Bucket 2 rejection contract on `Amount::Deserialize` (rejects `0o`/`0b` per ADR 0052) is re-validated against the new alloy `FromStr` behaviour — the alloy contract change may add new radices that need to be added to the cow rejection list.

## The 9 never-swap exceptions

Eight constraints plus four additional surfaces are verified as `DO NOT SWAP`. The CI grep gates that mechanize each fence live at `.github/workflows/never-swap-gates.yml`; the per-site `// DO NOT SWAP` comments live at the load-bearing call sites in `crates/contracts/`, `crates/core/`, `crates/signing/`, `crates/transport-policy/`, and `crates/transport-wasm/`. One further reimplementation fence — `Amount::parse_units` does the decimal scaling itself with checked arithmetic instead of the fail-open, panic-prone, silently-wrapping raw `alloy_primitives::utils::parse_units` (Bucket 2 row above) — carries its own `// DO NOT SWAP` comment block but no dedicated grep gate; it is held by the `gate-do-not-swap-census` comment-block count rather than a symbol-specific regex, so the census gate locks at eleven `DO NOT SWAP` blocks rather than ten.

The canonical roster is:

1. ECDSA `v` byte canonicalization (ADR 0022).
2. `Amount` / `SignedAmount` `new` radix sniffing (ADR 0052).
3. `Address::Display` lowercase (ADR 0052).
4. `SupportedChainId` + `api_path()` (ADR 0005, ADR 0011).
5. `TypedDataDomain` cow struct (ADR 0052, ADR 0040).
6. EIP-1271 blob Shape A vs B (ADR 0050 — future, composable deferred).
7. `cow-sdk-transport-policy` + `cow-sdk-transport-wasm` (ADR 0010, 0019, 0041, 0046).
8. `encode_address_word` cosmetic helper (no ADR — keep as-is).
9. Plus four additional never-swap surfaces:
   - `api_path()` URL labels (sub-invariant of #4),
   - `hex_decode_20` `const fn` (until the `hex!` macro adoption retires it),
   - `keccak_word` test oracle in `crates/contracts/src/order/hash.rs`,
   - EIP-712 type-string whitespace contract (ADR 0050).

The doctrine treats every entry in the canonical roster above as a binding never-swap fence; the CI grep gates at `.github/workflows/never-swap-gates.yml` enforce them mechanically.

## ADRs this doctrine consolidates

Numbered ADR cites with the load-bearing topics:

- **ADR 0005** — strong domain types as the default public contract.
- **ADR 0010** — runtime-neutral async + transport posture; three `HttpTransport` impls; cancellation seam.
- **ADR 0011** — typed Amount boundary; typestate builders; amendment anchors `Amount`/`SignedAmount` to `alloy_primitives::U256`/`I256`.
- **ADR 0012** — canonical `alloy::sol!` bindings + typed `Registry`; hand-rolled encoders forbidden.
- **ADR 0014** — EIP-1271 verification cache trait + two canonical impls; default-off, explicit-cache-arg contract.
- **ADR 0019** — `HttpTransport` is the sole live-dispatch surface on `OrderbookApi` / `SubgraphApi`.
- **ADR 0022** — `RecoverableSignature` typestate is the single contracts-boundary recoverable-signature value; pre-validates the v byte against the canonical accept set, then delegates byte assembly to `alloy_primitives::Signature::from_bytes_and_parity` / `Signature::as_bytes`; scheme-bundled recovery routes through `Signature::recover_address_from_prehash`.
- **ADR 0024** — `Provider` / `SigningProvider` capability split.
- **ADR 0026** — alloy major-release absorption plan; canary lane; runtime family confined to native adapter crates; amendment records the ADR 0052 widening.
- **ADR 0027** — non-exhaustive signature boundaries for future schemes (post-quantum, EIP-7212 secp256r1).
- **ADR 0029** — trait evolution through extension traits; `Provider` / `SigningProvider` shapes frozen through `0.x.y`.
- **ADR 0035** — `cow-sdk-alloy-provider` read-only adapter.
- **ADR 0036** — `cow-sdk-alloy-signer` local keystore adapter.
- **ADR 0037** — `cow-sdk-alloy` composed adapter (`AlloyClient` + `AlloyClientSignerHandle`).
- **ADR 0040** — wallet/provider callback boundary for JS consumers (five typed callbacks).
- **ADR 0044** — wasm flavor builds (default, orderbook, signing, cloudflare); positioning vs upstream TypeScript SDK.
- **ADR 0045** — async signer trait narrowing by operation.
- **ADR 0050** — EIP-1271 signature blob Shape A (Safe muxer) vs Shape B (raw forwarder); whitespace-free type strings; amendment defers composable + anchors encoder to `alloy_sol_types::SolValue`.
- **ADR 0051** — `Eip1271SignatureProvider` owned by `cow-sdk-signing`; trading consumes via inline `map_err`; compile-fail regression for re-export from trading.
- **ADR 0052** — alloy primitives as the canonical primitive layer (the umbrella ADR for this doctrine).

The principle-ADR map at `.github/config/principle-adr-map.yaml` is the traceability anchor. Every principle in `docs/principles.md` resolves to exactly one primary ADR plus an optional supporting set; ADRs scoped to adapter shape live in `out_of_scope_adrs` with written rationale so they do not require a principle pairing.

## ADRs to update when doctrine evolves

The doctrine is a read-only consolidation by design. Three sources of truth must be kept in sync when this document is amended:

1. **`docs/adr/`** — every entry in the Bucket 2 (COW-OWNED) and Bucket 3 (BOUNDARY-ADAPTER) tables resolves to a binding ADR cite. If a new entry lands without an ADR, the entry is suspect and a follow-up ADR is owed before the doctrine accepts the surface.

2. **`.github/config/principle-adr-map.yaml`** — every ADR referenced by this doctrine appears under a principle's `supporting_adrs` list or under `out_of_scope_adrs` with rationale. A doctrine amendment that introduces a new ADR must add a row in the principle map.

3. **`.github/config/audit-refresh-map.yml`** — each ADR in the bucket tables anchors at least one audit document (`docs/audit/<topic>-audit.md`). The refresh map records the cadence at which the audit is re-validated. Doctrine amendments that introduce a new ADR also extend the audit-refresh schedule with a row that names the audit document and trigger.

The principle map and the audit-refresh map together are the operational mechanism that prevents doctrine drift between this consolidation, the ADRs, and the audit evidence.

### Proposed ADR amendments triggered by writing this doctrine

Two minor ADR-vs-code drifts are flagged here so future amendments to the relevant ADRs can land alongside the corresponding source changes:

- **ADR 0052** — the ADR sentence "the cow `Amount::new` and `SignedAmount::new` constructors remain lenient (accept both decimal and `0x`-prefixed hex)" matches the shipped `Amount::new` but does *not* match the shipped `SignedAmount::new`, which is strict-decimal-only. The amendment lands alongside the strict-decimal `SignedAmount::new` audit refresh.
- **ADR 0049 cow-shed** — the `execute_hooks_signing_hash` collapse onto `SolStruct::eip712_signing_hash` (Bucket 1) should add an amendment block to ADR 0049 recording the swap.

No other ADR-vs-code drift was surfaced.

## Lint and CI enforcement

The eight grep gates that mechanize the never-swap fences live at `.github/workflows/never-swap-gates.yml`. The doctrine does not duplicate the gate regexes; it depends on the workflow remaining the canonical mechanical floor.

In addition to the never-swap CI gates above, the following gates exist or are scheduled:

- **`alloy-provider` allow-list check** — `alloy-provider` may appear in `cow-sdk-alloy-provider` and `cow-sdk-alloy` only (ADR 0026).
- **`alloy-signer-local` allow-list check** — same posture, `cow-sdk-alloy-signer` and `cow-sdk-alloy` only (ADR 0026).
- **`alloy-runtime` family confinement** — `alloy-network`, `alloy-consensus`, `alloy-rpc-types-eth`, `alloy-transport-*` confined to the same three adapter crates (ADR 0052).
- **`wasm-imports-grep-gate.yml`** — `cow-sdk-wasm` forbids direct `alloy*` imports; types are consumed via re-exports through `cow-sdk-contracts` and `cow-sdk-pure-helpers` (ADR 0052).
- **`cargo-metadata` negative-edge invariants** — `cow-sdk-signing ⇏ cow-sdk-trading`, `cow-sdk-composable ⇏ cow-sdk-trading`, `cow-sdk-cow-shed ⇏ cow-sdk-trading`; reverse-edge `cow-sdk-trading ⇒ cow-sdk-signing` (ADR 0051).
- **Compile-fail regression** — `crates/trading/tests/eip1271_signature_provider_no_reexport.rs` fails compile if `Eip1271SignatureProvider` is re-exported from `cow_sdk_trading` (ADR 0051).
- **Workspace resolution invariant test** — `Cargo.lock` resolves each alloy crate to exactly one version per ADR 0026.
- **Source-lock provenance gate** — the upstream commit hash is reproducible from the pinned reference; release validation rejects mutable upstream branches (ADR 0026, ADR 0032).
- **Panic-allowlist gate** — `.github/config/panic-allowlist.yaml` entry count strictly decreases by exactly 1 after the chain hex literal migration lands. The exact pre-migration baseline is recorded in the corresponding migration audit; the doctrine binds the *delta*, not the absolute count.
- **`cargo-semver-checks` lane** — runs against unpublished baseline as drift detection against `main` until the first published release (ADR 0052). Pre-1.0 semver-checks is *drift detection, not a release gate*; breaking changes remain the goal until 1.0 is on the runway.

The lint-and-CI layer is the mechanical enforcement of the doctrine; the doctrine is the human-readable form that explains *why* each gate exists.

## Open questions / future ADR triggers

The doctrine is decisive within the surface that the accepted ADR set covers. The following situations are explicitly not covered and would require a new ADR before the doctrine could classify the relevant primitive:

- **alloy crate splits** — if `alloy-primitives` ever splits into `alloy-primitives-core` plus `alloy-primitives-eip712`, the Bucket 1 table needs to be re-anchored to the new symbol locations. The compatibility matrix at ADR 0026 records exact versions, so the split is visible in the next pinned bump; the migration team should propose an ADR amendment to ADR 0026 alongside the dependency bump.

- **alloy crate yanks** — if alloy yanks a pinned version, `Cargo.lock` resolution fails and the release blocks. The runbook at [Alloy Major Release Runbook](alloy-major-release-runbook.md) (referenced by ADR 0026) covers the operational response; the doctrine inherits the runbook.

- **alloy-major bump (1.5 → 2.x ABI core, 2.0 → 3.x runtime)** — the canary lane (ADR 0026) signals drift; promotion to PR-blocking requires policy change. The doctrine's bucket assignments are independent of alloy major version; the absorption is per ADR 0026 / [Alloy Major Release Runbook](alloy-major-release-runbook.md). Worked Example 6 above is the doctrine's contribution to that runbook.

- **Removal of an upstream cowprotocol primitive** — if upstream cowprotocol retires (for example) the orderbook `xdai` URL label, the cow-owned `api_path()` row in Bucket 2 amends to remove the variant; the `SupportedChainId::GnosisChain` variant follows. This is a Bucket 2 evolution under the `#[non_exhaustive]` carve-out per [Forward-Compatible Public Surfaces](principles.md) / ADR 0031.

- **New alloy-runtime adapter (e.g. `cow-sdk-foundry-provider`)** — the doctrine's Bucket 3 supports peer adapters at the same shape. No doctrine amendment needed; the adapter implements `Provider` and ships as an additive leaf crate per ADR 0008. A peer adapter for a non-EVM runtime (Solana, Stellar, etc.) is currently out of scope for the cow protocol and would require a new ADR (and likely a new trait family) before classification.

- **Post-quantum primitives going Bucket 1** — ADR 0027 currently classifies PQ as Bucket 2 (cow-owned through additive `SigningScheme` variants). If alloy later ships a maintained PQ primitive crate with cow-protocol-aligned wire form, the primitive moves to Bucket 1 and the wire rejection stays Bucket 2 (same split as `Amount`). This is the natural absorption pattern.

- **`cow-sdk-composable` crate becoming a workspace member** — ADR 0050 records that composable is deferred. When the crate roots, the doctrine's Bucket 1 entries for `SolValue::abi_encode` and the Bucket 2 entries for Shape A / Shape B blob encoding become live; no doctrine amendment is needed because the rows are already authored for the deferred state. The migration team should re-validate the bucket assignments against the actual shipped code at the time of crate landing.

- **Browser EIP-1193 ecosystem fragmentation** — if a future EIP standardizes a new wallet-callback signature shape (e.g., post-EIP-1193 v2), the five typed callbacks in ADR 0040 require a new entry (or amendment) per the ADR 0045 narrowing principle. The Bucket 3 trait-plus-callback shape absorbs the new wallet contract without widening the SDK core surface.

The doctrine is decisive on every primitive currently shipped or planned in cow-rs. The open questions above are the explicit boundaries where future ADRs would extend it.

---

**This document is the canonical reading for the question "when does cow-rs use alloy, when does it own logic itself, when does it route through an adapter."** A maintainer who follows the decision tree above and cross-references the ADR cites in each bucket table arrives at the same answer that the existing ADR set already records. The doctrine is read-only over the ADRs by construction; it does not introduce policy that is not already accepted.
