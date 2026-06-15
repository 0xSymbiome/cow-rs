# ADR 0052: Alloy primitives as the canonical primitive layer

- Status: Accepted
- Date: 2026-05-19
- Last reviewed: 2026-06-15
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: alloy-primitives, alloy-sol-types, eip-712, abi, canonical-types
- Related: [ADR 0011](0011-typed-amount-boundary-and-typestate-ready-state-construction.md), [ADR 0012](0012-alloy-sol-bindings-and-registry-authority.md), [ADR 0022](0022-ecdsa-signature-v-normalization.md), [ADR 0026](0026-alloy-major-release-absorption-plan.md), [Alloy Doctrine](../alloy-doctrine.md)

## Decision

`cow-sdk-core` adopts `alloy-primitives` and `alloy-sol-types` as the canonical
primitive and EIP-712 / ABI layer for the workspace. cow-rs does not maintain a
parallel implementation of any primitive that alloy already ships.

Six cow-named types remain as `#[repr(transparent)]` newtypes over an alloy base,
each with a **sealed (private) inner field** so a value is constructible only
through a validating constructor:

| cow type | alloy base | wire form |
| --- | --- | --- |
| `Address` | `alloy_primitives::Address` | cow-owned `Display`/`Serialize`/`Deserialize` — **lowercase** |
| `Amount` | `alloy_primitives::U256` | cow-owned `Display`/`Serialize`/`Deserialize` — **decimal**, strict-decimal fail-closed `Deserialize` |
| `Hash32` | `alloy_primitives::B256` | alloy default (lowercase hex) |
| `AppDataHash` | `alloy_primitives::B256` | alloy default (lowercase hex) |
| `HexData` | `alloy_primitives::Bytes` | alloy default (lowercase hex) |
| `OrderUid` | `alloy_primitives::FixedBytes<56>` | alloy default (lowercase hex) |

The newtypes exist for two reasons a bare `pub type` alias cannot serve. First,
they keep the Rust type system distinguishing same-width domain types — `Hash32`
and `AppDataHash` are both `B256`, `OrderUid` is a `FixedBytes<56>` — across the
capability crates, where multi-argument constructors rely on compile-time
argument-swap detection as a safety guarantee. Second, they let cow own the wire
form where it diverges from an alloy default: `Address` emits lowercase (alloy
defaults to EIP-55 mixed-case checksum) and `Amount` emits and accepts decimal
(alloy serializes `U256` as hex, and alloy's `FromStr` sniffs four radix
prefixes). `#[repr(transparent)]` keeps the layout bit-for-bit identical to the
alloy base, so crossing the alloy seam is free through `From` and the typed
`as_alloy` / `into_alloy` (and `as_u256` / `into_u256`) accessors — never the
inner field.

`Amount` exposes **no operator overloads**. Arithmetic is explicit and
overflow-checked through `checked_add` / `checked_sub` / `checked_mul` and the
`saturating_*` family, so a silent `uint256` wrap cannot occur at a call site.

Hand-rolled cryptographic and encoding primitives — keccak256, EIP-712 domain
separation and struct hashing, EIP-191 message hashing, CREATE2 derivation,
signature byte assembly, hex serde, and canonical JSON — delegate to the
maintained alloy surface (`keccak256`, `Eip712Domain::separator`,
`SolStruct::eip712_signing_hash`, `eip191_hash_message`, `Address::create2`,
`Signature`) and a small set of maintained companion crates. cow-rs retains an
implementation only where a binding ADR records a required divergence from the
alloy behaviour; those exceptions are enumerated in the
[Alloy Doctrine](../alloy-doctrine.md) and pinned at their call sites by
`cargo check-source-fences`.

The alloy-core ABI family (`alloy-primitives`, `alloy-sol-types`,
`alloy-sol-macro`, `alloy-dyn-abi`, `alloy-json-abi`, `alloy-serde`) is an
in-scope dependency of `cow-sdk-core`, `cow-sdk-contracts`, `cow-sdk-signing`,
and `cow-sdk-app-data`. The alloy-runtime family (`alloy-provider`,
`alloy-signer-local`, `alloy-network`, `alloy-consensus`, `alloy-rpc-types-eth`,
`alloy-transport-*`) stays confined to the native adapter crates per
[ADR 0026](0026-alloy-major-release-absorption-plan.md). `cow-sdk-wasm` consumes
alloy-core primitives (`alloy_primitives`, `alloy_sol_types`) directly for ABI
and event decoding, but takes no dependency on the native alloy adapter crates
(`cow-sdk-alloy*`) or the alloy-runtime family; the `wasm-no-alloy-family` fence
enforces that exclusion.

## Why

A single canonical implementation per primitive is the structural precondition
for the workspace's shared-logic reviewability boundary: every shared primitive
operation has exactly one invocation path, so a reviewer audits keccak256,
EIP-712 hashing, or signature assembly once instead of chasing parallel
re-implementations across crates. Delegating to maintained alloy code also
absorbs upstream fixes and shrinks the wasm binary.

The delegation is byte-identical to the prior hand-rolled path on every parity
fixture under `parity/fixtures/`, with one deliberate exception: app-data JSON
canonicalisation now follows RFC 8785 UTF-16 key ordering, closing a latent gap
with the upstream TypeScript SDK for non-ASCII keys. ASCII-only documents are
unchanged.

## Must Remain True

- The six public types resolve at `cow_sdk_core::types::*` as
  `#[repr(transparent)]` newtypes with private inner fields.
- `Address` serializes and displays lowercase; `Amount` serializes decimal and
  its `Deserialize` rejects every radix prefix. These two divergences from alloy
  defaults are the reason the types are owned rather than aliased.
- Layout stays bit-for-bit identical to the alloy base; the alloy seam is crossed
  only through `From` / `as_alloy` / `into_alloy` / `as_u256` / `into_u256`.
- `Amount` exposes no operator overloads; arithmetic is `checked_*` / `saturating_*`.
- No capability crate depends on an alloy-runtime crate, and `cow-sdk-wasm` takes
  no dependency on the `cow-sdk-alloy*` adapter crates.
- Every parity fixture continues to pass byte-identically, the RFC 8785 UTF-16
  app-data case excepted.

## Alternatives Rejected

- **Keep the hand-rolled string newtypes.** Forces every accessor to re-parse hex
  and every crate to carry parallel primitive implementations — the structural
  reason the shared-logic reviewability boundary could not be met.
- **Use bare `pub type` aliases over the alloy types.** Erases the domain
  distinction between same-width types (`Hash32` vs `AppDataHash`) that the
  capability crates rely on for compile-time argument-swap detection, and forces
  an extension-trait surface for cow accessors (orphan rules forbid inherent
  methods on alloy types).
- **Alias the numeric type with a per-field `#[serde(with = …)]` helper.**
  Requires annotating every amount-bearing DTO field; a single miss silently
  flips an amount from decimal to hex, and alloy's `FromStr` still widens the
  accepted radix set. A cow-owned `Deserialize` on one newtype closes the gap
  once.
- **Depend on `alloy-primitives` without the cow re-export.** Breaks the stable
  `cow_sdk_core::Address` import path that downstream code resolves against.

## Links

- [Alloy Doctrine](../alloy-doctrine.md) — the operational bucket tables and
  never-swap fences this decision anchors
- [Architecture](../architecture.md)
- [Principles](../principles.md)
- [ADR 0011](0011-typed-amount-boundary-and-typestate-ready-state-construction.md),
  [ADR 0012](0012-alloy-sol-bindings-and-registry-authority.md),
  [ADR 0022](0022-ecdsa-signature-v-normalization.md),
  [ADR 0026](0026-alloy-major-release-absorption-plan.md)
