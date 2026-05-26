# ADR 0052: Alloy primitives as the canonical primitive layer

- Status: Accepted (amended)
- Date: 2026-05-19
- Last reviewed: 2026-05-22
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: alloy-primitives, alloy-sol-types, eip-712, abi, canonical-types
- Related: [ADR 0011](0011-typed-amount-boundary-and-typestate-ready-state-construction.md), [ADR 0012](0012-alloy-sol-bindings-and-registry-authority.md), [ADR 0014](0014-eip1271-verification-cache.md), [ADR 0022](0022-ecdsa-signature-v-normalization.md), [ADR 0026](0026-alloy-major-release-absorption-plan.md), [ADR 0028](0028-account-abstraction-integration-plan.md), [ADR 0039](0039-typescript-callable-wasm-sdk-surface.md), [ADR 0048](0048-composable-conditional-order-framework.md), [ADR 0049](0049-cow-shed-account-abstraction-proxy.md), [ADR 0050](0050-eip1271-signature-blob-encoding.md)

## Decision

`cow_sdk_core` adopts `alloy_primitives` and `alloy_sol_types` as the
canonical primitive and EIP-712 / ABI layer across the workspace.

The cow-named identity types `Address`, `Hash32`, `AppDataHash`,
`HexData`, and `OrderUid` and the cow-named numeric types `Amount` and
`SignedAmount` are cow-owned `#[repr(transparent)]` newtypes over the
corresponding `alloy_primitives` type. The type-aliased derivative
hashes `TransactionHash`, `BlockHash`, and `OrderDigest` re-route
through `Hash32`. `TypedDataDomain` is the cow struct that already
ships in the working tree (`name: String`, `version: String`,
`chain_id: ChainId`, `verifying_contract: Address`, no `salt`),
preserved as-is; the cow struct's cow-owned `Serialize` impl emits the
EIP-1193 `eth_signTypedData_v4` wire shape directly (numeric
`chainId`, required `verifyingContract`, no `salt`) and an
`into_alloy_domain()` adapter method converts to
`alloy_sol_types::Eip712Domain` at the EIP-712 hashing seam.
`Address`, `Amount`, and `SignedAmount` carry cow-owned
`Display`/`Serialize`/`Deserialize` impls; the other four byte-typed
identity newtypes forward to alloy defaults that already match the
cow lowercase wire form. The cow newtype layer preserves the Rust
type-system distinction between same-width byte primitives that the
cow capability crates rely on (`Hash32` vs `AppDataHash` vs the
digest-shaped fields embedded in DTOs) and preserves the
strict-decimal-only fail-closed wire-form contract for `Amount` and
`SignedAmount` on the `Deserialize` boundary (the constructors stay
lenient — accepting both decimal and `0x`-prefixed hex — to preserve
the existing observed behavior), while keeping bit-for-bit layout
compatibility with the underlying alloy primitive (zero-cost
conversion at the adapter boundary via `.0` or `From::from(...)`).

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
  existing paths. `Address`, `Hash32`, `AppDataHash`, `HexData`,
  `OrderUid`, `Amount`, and `SignedAmount` are cow-owned
  `#[repr(transparent)]` newtypes around `alloy_primitives::Address`,
  `alloy_primitives::B256`, `alloy_primitives::B256`,
  `alloy_primitives::Bytes`, `alloy_primitives::FixedBytes<56>`,
  `alloy_primitives::U256`, and `alloy_primitives::I256` respectively.
  `Address`, `Amount`, and `SignedAmount` carry cow-owned `Display`,
  `Serialize`, and `Deserialize` impls — `Address` because alloy's
  default `Display` is EIP-55 checksum casing and the cow wire form
  is lowercase, and `Amount`/`SignedAmount` because alloy's default
  `Serialize` for `U256` is hex (not decimal) and alloy's `FromStr`
  for `Uint`/`Signed` prefix-sniffs four radices. The cow
  strict-decimal-only fail-closed contract for `Amount` and
  `SignedAmount` applies to both the `Deserialize` wire boundary AND
  to `SignedAmount::new`, which accepts only the grammar `-?[0-9]+`
  and rejects every `0x`/`0X`/`0o`/`0O`/`0b`/`0B` prefix that the
  alloy `I256::from_str` would otherwise silently accept; the
  `SignedAmount::new` narrowing protects the strict JSON-decimal-only
  signed wire contract. The cow `Amount::new` constructor remains
  lenient (accepts both decimal and `0x`-prefixed hex; explicitly
  rejects `0o`/`0b`) to preserve the existing unsigned-amount
  constructor contract. `Hash32`, `AppDataHash`, `HexData`,
  and `OrderUid` forward `Display`, `Serialize`, and `Deserialize`
  to the underlying alloy primitive whose defaults already match the
  cow lowercase wire form. Each newtype carries cow-defined inherent
  methods for the canonical accessor surface (`new`, `from_bytes`,
  `to_hex_string`, `as_slice`, `as_alloy`, `into_alloy`, `zero`,
  `is_zero`, `byte_length`, plus `to_cid` on `AppDataHash`). The owned hex-string accessor is
  named `to_hex_string(&self) -> String` (following the Rust stdlib
  convention that `to_*` returns owned and `as_*` returns a borrow);
  the prior cached-struct `as_str(&self) -> &str` shape retires and
  the legacy callsites are normalized to the canonical accessor for
  each use case (`Display`, `as_slice`, `is_zero`, `to_string`, or
  `to_hex_string` where the hex form is required explicitly).
  Bit-for-bit layout compatibility with the underlying alloy
  primitive is preserved through the `repr(transparent)`
  representation; conversion at the alloy seam is free at runtime
  through `From::from(addr).into()` (canonical) or `.0` access
  (escape hatch). `TransactionHash`, `BlockHash`, and `OrderDigest`
  are `pub type` aliases over `Hash32`. `TypedDataDomain` is the cow
  struct that already ships in the working tree (`name: String`,
  `version: String`, `chain_id: ChainId`, `verifying_contract:
  Address`, no `salt`); the cow struct's cow-owned `Serialize` impl
  emits the EIP-1193 `eth_signTypedData_v4` wire shape directly
  (numeric `chainId`, required `verifyingContract`, no `salt`) and
  the cow struct carries an `into_alloy_domain(&self) ->
  alloy_sol_types::Eip712Domain` adapter method for the EIP-712
  hashing seam. The cow identity and numeric newtypes carry cow-owned
  `Tsify` derives (via the `tsify` crate at version `0.5`) for
  wasm-bindgen so the TypeScript declaration shape does not depend
  on alloy primitives implementing `Tsify`. The `cow_sdk_core::prelude`
  re-export ships the cow newtypes directly; the prior `AddressExt`,
  `Hash32Ext`, `AppDataHashExt`, `HexDataExt`, `OrderUidExt`,
  `AmountExt`, and `SignedAmountExt` extension traits are retired
  entirely because the cow newtypes carry their accessor surface as
  inherent methods. The `cargo-semver-checks` lane on
  `cow-sdk-core`, `cow-sdk-contracts`, `cow-sdk-signing`,
  `cow-sdk-app-data`, `cow-sdk-orderbook`, `cow-sdk-trading`,
  `cow-sdk-subgraph`, `cow-sdk-browser-wallet`, and
  `cow-sdk-transport-wasm` reports no breaking changes against the
  unpublished baseline (the lane runs as drift-detection against
  `main` until the first published release).
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
- Cost: every cow newtype carries a cow-owned inherent-method
  accessor surface (`new`, `from_bytes`, `to_hex_string`, `as_slice`,
  `as_alloy`, `into_alloy`, `zero`, `is_zero`, `byte_length`, plus
  `to_cid` on `AppDataHash`).
  `Address`, `Amount`, and `SignedAmount` additionally carry a
  cow-owned trait surface (`Display`, `Serialize`, `Deserialize`);
  `Hash32`, `AppDataHash`, `HexData`, and `OrderUid` forward to alloy
  defaults via `#[serde(transparent)]` and a one-line `Display`
  delegate, saving roughly 150-200 lines of re-implementation.
  `Amount` and `SignedAmount` additionally carry cow-owned operator
  overloads (`Add`, `Sub`, `Mul`, `AddAssign`, etc.) that delegate to
  the inner `U256`/`I256` so existing arithmetic callsites work
  verbatim. The `repr(transparent)` representation keeps the layout
  bit-for-bit identical to the underlying alloy primitive, so
  conversion at the alloy boundary is free at runtime through
  `From::from(...).into()` (canonical) or `.0` access (escape hatch).
  The trait + inherent surface costs roughly 70-100 lines per newtype
  on the cow-owned-trait family (`Address`, `Amount`, `SignedAmount`)
  and roughly 30-40 lines per newtype on the alloy-forwarding family
  (`Hash32`, `AppDataHash`, `HexData`, `OrderUid`) — net roughly
  500-600 lines of newtype code in `crates/core/src/types/`. In
  exchange for retiring the historical `String`-backed identity
  newtype layer, the `crates/core/src/types/identity_ext.rs`
  extension-trait surface, the cow-side `crates/core/src/types/hex.rs`
  helpers, the cached `inner + hex` half-state on every identity
  type, the cow-side hex helpers in
  `crates/contracts/src/primitives.rs`, and the cow-alloy conversion
  helpers across `crates/alloy-provider/src/conversion.rs` and
  `crates/alloy/src/conversion.rs`. The cow→alloy `TypedDataDomain`
  adapter at `crates/alloy-signer/src/conversion.rs` simplifies from
  207 lines to a focused `into_alloy_domain()` helper (~30 lines)
  rather than deleting entirely — the cow `TypedDataDomain` stays the
  canonical in-memory shape and owns its `Serialize` impl, but the
  EIP-712 hashing seam still needs a cow→alloy struct adapter and
  that adapter lives in the alloy-signer crate. The workspace surface
  dependency on alloy widens beyond the native adapter crates; the
  cow-rs contracts, signing, and orderbook parity fixture suites must
  run in full as part of any alloy-major rehearsal so the alloy-core
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
- Use `pub type` aliases for the byte-typed identity types without
  `repr(transparent)` newtype wrappers: rejected. The all-aliases
  shape would seamlessly interoperate with the alloy ecosystem but at
  the cost of conflating cow domain types that share an underlying
  byte width — `Hash32` and `AppDataHash` would both become
  `alloy_primitives::B256` at the Rust type level, and `HexData`
  would become `alloy_primitives::Bytes`. The cow codebase relies on
  the Rust type system to distinguish these in function signatures
  and DTO field types across the orderbook, trading, app-data,
  signing, and composable crates (over 750 identity-type occurrences
  across more than 130 files, including 22 multi-parameter
  constructor signatures where compile-time argument-swap detection
  is the safety guarantee). The `repr(transparent)` newtype shape
  preserves the type distinction while keeping bit-for-bit layout
  compatibility (zero-cost conversion at the adapter boundary). The
  all-aliases shape would also require an extension-trait surface to
  attach cow-specific accessor methods (orphan rules forbid inherent
  methods on external types), introducing method-resolution
  ambiguity between extension traits that target the same alloy
  primitive.
- Use `pub type` aliases for the numeric types `Amount` and
  `SignedAmount` (with per-DTO-field
  `#[serde(with = "alloy_serde::displayfromstr")]`): rejected. The
  alias approach would require annotating roughly 100 DTO fields
  across the capability crates; missing the annotation on any single
  field would silently flip the wire form from decimal to
  alloy-default hex. More critically, alloy's underlying
  `ruint::Uint::FromStr` prefix-sniffs four radices (`0x`, `0o`,
  `0b`, plus uppercase variants), so the `displayfromstr` mitigation
  widens the input grammar for both `Amount` and `SignedAmount`
  deserialization — relaxing cow's strict-decimal fail-closed
  contract. Only a cow-owned `Deserialize` impl closes the gap, and
  the cleanest place for that impl is on a cow newtype rather than
  on a per-DTO-field serde helper. The cow newtype approach matches
  the design language already chosen for the byte-typed identity
  family and replaces every per-field annotation with two cow-owned
  trait implementations.
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

## Amendment 2026-05-22: retire `Address::normalized_key`

`Address::normalized_key` previously returned the lowercase 0x-prefixed
hex form. The body was identical to `Address::to_hex_string` because the
cow `Address` already canonicalises every input to its lowercase
representation at construction time, so the two accessors produced
byte-identical output for every input. The duplicate accessor is
retired; every call site routes through `Address::to_hex_string`
directly. The canonical inherent-method surface for the cow newtypes
listed above no longer enumerates a per-type case-insensitive-key
accessor.

**Proven by:**

- [Shared Logic Reviewability Audit](../audit/shared-logic-reviewability-audit.md)

## Amendment 2026-05-26: retire direct `hex` dependency from contracts and signing

The canonical hex API for the cow workspace is `alloy_primitives::hex::*`,
which resolves to the `const-hex` crate re-exported through
`alloy-primitives 1.5.x`. The doctrinal boundary that previously closed
on `cow-sdk-core` now closes on `cow-sdk-contracts` and `cow-sdk-signing`
as well: every production `hex::encode` and `hex::decode` callsite under
`crates/contracts/src/**` and `crates/signing/src/**` routes through
`alloy_primitives::hex::{encode, decode}`, and both crates retire their
`[dependencies]` declaration of the upstream `hex` crate.

The `ContractsError::DecodeHex { source }` variant carries the typed
`alloy_primitives::hex::FromHexError` value (a re-export of
`const_hex::FromHexError`) so the production error surface no longer
references the upstream `hex` crate's error type. The variant remains
`#[non_exhaustive]` through the enum-level marker.

A permanent carve-out remains for `[dev-dependencies]`: any cow crate
whose integration tests parse hex fixtures may continue to declare
`hex.workspace = true` under `[dev-dependencies]` without violating the
canonical-primitive-layer mandate. The carve-out applies to test
fixture parsing only and does not extend to production code.

**Proven by:**

- [Dependency Gate Audit](../audit/dependency-gate-audit.md)
