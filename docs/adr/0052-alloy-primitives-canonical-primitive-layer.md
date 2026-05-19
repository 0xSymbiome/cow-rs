# ADR 0052: Alloy primitives as the canonical primitive layer

- Status: Accepted
- Date: 2026-05-19
- Last reviewed: 2026-05-19
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: alloy-primitives, alloy-sol-types, eip-712, abi, canonical-types
- Related: [ADR 0011](0011-typed-amount-boundary-and-typestate-ready-state-construction.md), [ADR 0012](0012-alloy-sol-bindings-and-registry-authority.md), [ADR 0014](0014-eip1271-verification-cache.md), [ADR 0022](0022-ecdsa-signature-v-normalization.md), [ADR 0026](0026-alloy-major-release-absorption-plan.md), [ADR 0028](0028-account-abstraction-integration-plan.md), [ADR 0039](0039-typescript-callable-wasm-sdk-surface.md), [ADR 0048](0048-composable-conditional-order-framework.md), [ADR 0049](0049-cow-shed-account-abstraction-proxy.md), [ADR 0050](0050-eip1271-signature-blob-encoding.md)

## Decision

`cow_sdk_core` adopts `alloy_primitives` and `alloy_sol_types` as the
canonical primitive and EIP-712 / ABI layer across the workspace.

The cow-named public types `Address`, `Hash32`, `AppDataHash`,
`TransactionHash`, `BlockHash`, `OrderDigest`, `HexData`, `OrderUid`,
`Amount`, `SignedAmount`, and `TypedDataDomain` resolve to re-exports of
`alloy_primitives::{Address, B256, Bytes, FixedBytes<56>, U256, I256}`
and `alloy_sol_types::Eip712Domain` respectively.

Hand-rolled `keccak256`, `domain_separator`, `typed_data_digest`,
EIP-712 type strings, EIP-191 message wrappers, CREATE2 derivation,
hex-prefixed serde, signature byte-pack, IMF-fixdate parsing,
canonical-JSON serialisation, and Multiplexer merkle machinery are
replaced by maintained-crate equivalents (`alloy_primitives::keccak256`,
`Eip712Domain::separator`, `SolStruct::eip712_signing_hash`,
`eip191_hash_message`, `Address::create2`, `Bytes` serde,
`Signature::as_bytes`, `httpdate`, `serde_jcs`, `rs_merkle`).

The alloy-core ABI family (`alloy-primitives`, `alloy-sol-types`,
`alloy-sol-macro`, `alloy-dyn-abi`, `alloy-json-abi`, `alloy-serde`)
becomes an in-scope dependency of `cow-sdk-core`, `cow-sdk-contracts`,
`cow-sdk-signing`, `cow-sdk-app-data`, `cow-sdk-cow-shed`, and
`cow-sdk-composable`. The alloy-runtime family (`alloy-provider`,
`alloy-signer-local`, `alloy-network`, `alloy-consensus`,
`alloy-rpc-types-eth`, `alloy-transport-*`) remains confined to the
native adapter crates per [ADR 0026](0026-alloy-major-release-absorption-plan.md).

`cow-sdk-wasm` continues to forbid direct `alloy*` imports through the
`wasm-imports-grep-gate.yml` workflow; cow-sdk-wasm consumes alloy
types via `cow-sdk-contracts` and `cow-sdk-pure-helpers` re-exports.

## Why

Maintained-crate equivalents for every algorithm in scope eliminate
roughly 2,530 net lines of hand-rolled code across 17 workspace
crates, retire two direct dependencies (`sha3`, `num-bigint`), add
four maintained dependencies (`httpdate`, `serde_jcs`, `rs_merkle`,
`alloy-serde`), reduce the wasm binary size, and remove duplicate
definitions of `keccak256`, `parse_u256`, `encode_address`, and
`Call` (5 + 3 + 3 + 3 = 14 duplicate definitions collapse to one
each). The two `SigningScheme` enums in `cow-sdk-contracts` and
`cow-sdk-orderbook` retain divergent wire formats and are connected
by a typed `From` / `TryFrom` bridge that prevents variant drift.

The maintained-crate path is byte-identical to the hand-rolled path
on every existing parity fixture, with one documented exception:
`crates/app-data/src/info.rs`'s JSON canonicalisation now uses
RFC 8785 UTF-16 key ordering instead of bytewise ordering, closing a
latent gap with the upstream `cow-sdk` TypeScript implementation for
non-ASCII keys. ASCII-only documents remain byte-identical.

## Must Remain True

- Public surface: the cow-named public types `Address`, `Hash32`,
  `AppDataHash`, `HexData`, `OrderUid`, `Amount`, `SignedAmount`, and
  `TypedDataDomain` resolve through `cow_sdk_core::types::*` at their
  existing paths. `Hash32`, `HexData`, `OrderUid`, `AppDataHash`,
  `Amount`, and `SignedAmount` are `pub type` aliases over
  `alloy_primitives::B256`, `alloy_primitives::Bytes`,
  `alloy_primitives::FixedBytes<56>`, `alloy_primitives::B256`,
  `alloy_primitives::U256`, and `alloy_primitives::I256` respectively.
  `Address` is a `repr(transparent)` newtype around
  `alloy_primitives::Address` whose cow-owned `Display` and
  `Serialize` impls emit lowercase 0x-prefixed hex (see Cost below for
  the wire-format rationale). `TypedDataDomain` resolves to
  `alloy_sol_types::Eip712Domain`. Extension traits (`AddressExt`,
  `OrderUidExt`, `Hash32Ext`, `AppDataHashExt`, `HexDataExt`,
  `AmountExt`, `SignedAmountExt`) carry the cow-specific accessor
  methods (`as_str`, `new`, `to_cid`, `to_str_radix_10`,
  `to_str_radix_16`, `zero`, `is_zero`); the `cow_sdk_core::prelude`
  re-export brings these traits into scope. The blocking
  `cargo-semver-checks` lane on `cow-sdk-core`, `cow-sdk-contracts`,
  `cow-sdk-signing`, `cow-sdk-app-data`, `cow-sdk-orderbook`,
  `cow-sdk-trading`, `cow-sdk-subgraph`, `cow-sdk-browser-wallet`,
  and `cow-sdk-transport-wasm` reports no breaking changes.
- Runtime and support: `cow-sdk-core`, `cow-sdk-contracts`,
  `cow-sdk-signing`, `cow-sdk-app-data`, `cow-sdk-cow-shed`, and
  `cow-sdk-composable` may depend directly on `alloy-primitives`,
  `alloy-sol-types`, `alloy-sol-macro`, `alloy-dyn-abi`,
  `alloy-json-abi`, and `alloy-serde`. `cow-sdk-core` and every
  other capability crate must not depend directly on any alloy-runtime
  crate (`alloy-provider`, `alloy-signer-local`, `alloy-network`,
  `alloy-consensus`, `alloy-rpc-types-eth`, `alloy-transport-*`);
  those remain confined to `cow-sdk-alloy`, `cow-sdk-alloy-provider`,
  and `cow-sdk-alloy-signer` per
  [ADR 0026](0026-alloy-major-release-absorption-plan.md).
  `cow-sdk-wasm` must not carry direct alloy imports in its source;
  the `wasm-imports-grep-gate.yml` workflow continues to enforce this.
- Validation and review: every parity fixture under
  `parity/fixtures/` continues to pass byte-identically. The one
  documented exception is
  `parity/fixtures/app_data/canonical_json_utf16.json` (new), which
  documents the RFC 8785 UTF-16 key ordering for non-ASCII keys;
  ASCII-only fixtures under `parity/fixtures/app_data/` continue
  byte-identical. Seven new parity fixtures land alongside this
  decision:
  `parity/fixtures/cow_shed/execute_hooks_calldata.json` (populated),
  `parity/fixtures/cow_shed/eoa_signature_byte_order.json` (expanded),
  `parity/fixtures/eip712/order_digests.json` (new),
  `parity/fixtures/ecdsa/v_normalization.json` (new),
  `parity/fixtures/app_data/canonical_json_utf16.json` (new — covers
  the UTF-16 ordering gap closure),
  `parity/fixtures/retry_after/imf_fixdate_*.json` (new — covers
  IMF-fixdate accept / reject branches and RFC 850 legacy support),
  and `parity/fixtures/composable/multiplexer_proofs.json` (new —
  covers Multiplexer merkle proofs for trees of size 1, 2, 4, 8, and
  16). `docs/audit/transport-policy-coverage-audit.md` reflects the
  `httpdate` dispatch; `docs/audit/cid-dependency-audit.md` names
  `serde_jcs` among the dependency-coverage rows.
- Cost: the `Address` lowercase wire-format invariant
  (`docs/performance.md` § Address Equality, `docs/deployments.md`
  doctests) is preserved through the `repr(transparent)` newtype's
  cow-owned `Display` and `Serialize` impls.
  `alloy_primitives::Address::Display` defaults to EIP-55 checksum
  casing, so a plain type alias would silently break the wire
  contract; the `repr(transparent)` newtype keeps the bit-for-bit
  layout of `alloy_primitives::Address` while owning the trait
  surface that emits the canonical lowercase 0x-prefixed hex. The
  `Amount` and `SignedAmount` decimal-string wire format is
  preserved through `#[serde(with = "alloy_serde::displayfromstr")]`
  on every cow DTO field of these types; per-field annotation is the
  canonical pattern across the workspace because it keeps the wire
  shape reviewable at the field declaration site. The `AmountExt`
  accessor surface ships its final shape directly: `to_str_radix_10()`
  and `to_str_radix_16()` are the radix accessors, with no parametric
  `to_str_radix(N)` variant. The workspace surface dependency on
  alloy widens beyond the native adapter crates; the cow-rs
  contracts, signing, and orderbook parity fixture suites must run
  in full as part of any alloy-major rehearsal so the alloy-core
  surface is re-validated at every major-version absorption.

## Alternatives Rejected

- Keep the hand-rolled string newtypes: rejected. The string-newtype
  layer forces every consuming crate to round-trip through hex
  parsing on every accessor read and to maintain parallel
  implementations of the same primitive operations across the
  workspace (`keccak256` wrappers, U256 and quantity parsers,
  address-encoding helpers, and the cow-to-alloy conversion modules
  in the alloy-adapter crates). The shared-logic reviewability
  boundary captured in
  [`docs/audit/shared-logic-reviewability-audit.md`](../audit/shared-logic-reviewability-audit.md)
  requires every shared primitive to have one canonical invocation
  path; the string-newtype layer is the structural reason the
  workspace cannot satisfy that requirement without typed
  re-exports. Replacing the newtypes with `alloy_primitives`
  re-exports collapses the parallel implementations onto canonical
  alloy entry points and resolves the reviewability boundary at the
  type system.
- Wrap alloy types in a second cow-named newtype layer for every
  cow-named type: rejected. Wrapping every type reproduces the
  maintenance cost of the original string newtypes (every accessor
  method must be rewritten on the wrapper). The decision adopts
  `pub type` aliases for `Hash32`, `HexData`, `OrderUid`,
  `AppDataHash`, `Amount`, and `SignedAmount`, and a
  `repr(transparent)` newtype for `Address` because the lowercase
  `Display` wire contract requires a cow-owned `Display` /
  `Serialize` impl that the type-alias path cannot provide.
- Use `alloy-primitives` directly without re-export: rejected.
  Public-API stability requires the cow-named import path
  `cow_sdk_core::Address` to continue resolving. Re-export via
  `pub type` aliases preserves both the path and the type identity.
- Use a single coordinated change set for the full migration:
  rejected. The incremental decomposition spreads the work across
  multiple commits; each change set's parity-fixture gate catches
  regression locally; rollback at any change-set boundary is
  feasible.
- Wait for a future alloy major release before adopting alloy
  primitives: rejected. The current workspace pins `alloy-core 1.5.7`
  and `alloy-runtime 2.0.4` and stays on those pins. This decision is
  orthogonal to alloy-major bumps; future bumps are handled by
  `docs/alloy-major-release-runbook.md`.

## Links

- [Architecture](../architecture.md)
- [Principles](../principles.md)
- [ADR 0011](0011-typed-amount-boundary-and-typestate-ready-state-construction.md)
- [ADR 0012](0012-alloy-sol-bindings-and-registry-authority.md)
- [ADR 0026](0026-alloy-major-release-absorption-plan.md)
- [Alloy major release runbook](../alloy-major-release-runbook.md)

**Proven by:**

- [Shared Logic Reviewability Audit](../audit/shared-logic-reviewability-audit.md)
