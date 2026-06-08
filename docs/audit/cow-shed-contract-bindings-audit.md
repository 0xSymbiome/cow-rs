# COW Shed Contract Bindings Audit

Status: Current
Last reviewed: 2026-06-08
Owning surface: inline COW Shed `alloy::sol!` bindings, proxy creation-code artifacts, version-call evidence, and deployment registry rows
Refresh trigger: Refresh when COW Shed deployments, proxy creation code, factory ABIs, hook type strings, the deployed `VERSION()` return value, or the upstream commit pin for the COW Shed source change.
Related docs:
- [ADR 0049](../adr/0049-cow-shed-account-abstraction-proxy.md)
- [ADR 0050](../adr/0050-eip1271-signature-blob-encoding.md)
- [ADR 0051](../adr/0051-signing-owned-eip1271-signature-provider-trait.md)
- [ADR 0052](../adr/0052-alloy-primitives-canonical-primitive-layer.md)
- [COW Shed App-Data Integration Audit](cow-shed-app-data-integration-audit.md)

## Scope

This audit covers:

- the inline COW Shed `alloy::sol!` bindings that reproduce the upstream
  Solidity surface verbatim, with the upstream source pinned by commit in
  `parity/source-lock.yaml` and proven byte-for-byte by the JSON parity
  fixtures under `parity/fixtures/cow_shed/`;
- the per-version proxy creation-code artifacts and SHA-256 digest
  neighbors;
- the per-chain `VERSION()` call evidence captured in
  `crates/contracts/abi/cow-shed/version-call-results.json`;
- the schema v2 deployment registry rows for the COW Shed factory and
  implementation contracts;
- the Gnosis-only `COWShedForComposableCoW` forwarder gate that enforces
  chain id 100 for the bridge variant;
- the EIP-712 type strings used by the hook structure, including the
  whitespace-free declaration order and the EOA signature byte order
  `r || s || v`.

It does not cover the COW Shed hook metadata schema integration with the
app-data crate; that boundary is governed by the
[COW Shed App-Data Integration Audit](cow-shed-app-data-integration-audit.md).

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Inline bindings | The inline COW Shed `alloy::sol!` bindings (mirroring upstream pinned by commit in `parity/source-lock.yaml`) emit type strings byte-identical to the upstream sources, including no whitespace between commas, proven by the JSON parity fixtures under `parity/fixtures/cow_shed/` | Conforms |
| Proxy creation-code | `v1.0.0.bin` and `v1.0.1.bin` artifacts ship with adjacent `.sha256` digest neighbors validated by `crates/contracts/build.rs` | Conforms |
| Version-call evidence | Every per-chain row in `version-call-results.json` records `decoded_version == "1.0.1"` and `expected_sdk_version == "CowShedVersion::V1_0_1"` | Conforms |
| Deployment registry | COW Shed factory and implementation rows are present in `registry.toml` for every supported chain id; `COWShedForComposableCoW` is present only for chain id 100 | Conforms |
| Gnosis forwarder gate | The Gnosis-only forwarder is reachable only when the caller selects chain id 100; all other chains return the typed `CowShedError::COWShedForComposableCoWGnosisOnly { chain }` variant | Conforms (contract; helper body lands in a later capability landing) |
| Hook type strings | Canonical type strings carry no whitespace between commas in declaration order; the EOA signature byte order is `r || s || v` | Conforms |
| EIP-712 hashing | Domain separator and signing digest are produced by `alloy_sol_types::Eip712Domain::separator` and `<ExecuteHooks as SolStruct>::eip712_signing_hash` respectively; bytes match the reference parity fixtures | Conforms |
| Call type identity | The macro-emitted `Call` declared in the canonical sol! block is the single source of truth for typed-data hashing, ABI calldata building, and both proxy and factory interface signatures; the four representative `executeHooks` calldata rows in the parity fixture catalog the wire-byte contract | Conforms |
| CREATE2 derivation | Proxy address derivation routes through `alloy_primitives::Address::create2` over the per-user salt and the proxy init-code hash; the thirty per-chain, per-user rows in the proxy-address parity fixture catalog the wire-byte contract | Conforms |
| EOA signature byte order | The ERC-2098 compact signature decoder routes through `alloy_primitives::Signature::from_erc2098` and `Signature::as_bytes`, emitting the canonical 65-byte `r \|\| s \|\| v` layout with `v ∈ {27, 28}`; the four representative rows in the EOA signature byte-order parity fixture catalog the wire-byte contract | Conforms |

## Current Contract

### Inline bindings

The COW Shed bindings are inline `alloy::sol!` interfaces that reproduce
the upstream Solidity surface verbatim. The upstream `cowdao-grants/cow-shed`
source they mirror is pinned by commit under `repositories:` in
`parity/source-lock.yaml`, and the JSON parity fixtures under
`parity/fixtures/cow_shed/` prove the bindings produce byte-identical
wire bytes for the proxy, factory, `COWShed`, `COWShedForComposableCoW`,
forwarder, and hook surfaces. The EIP-712 type strings the bindings emit
carry no whitespace between commas in declaration order; any future
amendment that adds whitespace is a regression caught by the type-string
parity contract test.

### Proxy creation-code

Per-version proxy creation-code artifacts ship at
`crates/contracts/abi/cow-shed/proxy-creation-code/v1.0.0.bin` and
`v1.0.1.bin` with adjacent `.sha256` digest neighbors. The build script
`crates/contracts/build.rs` reads each `.bin` file, computes SHA-256, and
compares to the digest neighbor; a mismatch fails the build. The
init-code hash used at CREATE2 derivation time is computed per call as
`keccak256(PROXY_CREATION_CODE || abi.encode(implementation, who))`; the
`.bin` files store the deployer bytecode prefix and never the full init
code, so derivation works correctly for any user address.

### Version-call evidence

The per-chain `VERSION()` call evidence at
`crates/contracts/abi/cow-shed/version-call-results.json` records the
deployed implementation address, the factory address, and the decoded
version string per chain id. Every row records
`decoded_version == "1.0.1"` and `expected_sdk_version ==
"CowShedVersion::V1_0_1"`, anchoring the SDK's default version to deployed
reality.

### Gnosis forwarder gate

The `COWShedForComposableCoW` contract is deployed only on Gnosis Chain
(chain id 100). The forwarder gate is anchored by the typed
`CowShedError::COWShedForComposableCoWGnosisOnly { chain }` variant; any
constructor or interaction helper that targets the forwarder on a
non-Gnosis chain id must return this variant. The ENS-related helpers gate
behind the `cow-shed-ens` Cargo feature (default off) so non-Gnosis builds
do not pull in the ENS resolver surface.

### Hook type strings

The canonical EIP-712 type strings are
`Call(address target,uint256 value,bytes callData,bool allowFailure,bool isDelegateCall)`
and
`ExecuteHooks(Call[] calls,bytes32 nonce,uint256 deadline)Call(address target,uint256 value,bytes callData,bool allowFailure,bool isDelegateCall)`.
The EOA signature byte order is `r || s || v` (not the standard
`v || r || s`); the canonical 65-byte layout is produced and validated by
`cow_sdk_contracts::RecoverableSignature`, whose `parse_bytes` rejects any
non-65-byte input and any recovery byte outside `{0, 1, 27, 28}` (ADR 0022). A
smart-contract (EIP-1271) owner instead supplies a variable-length signature
blob; `encode_execute_hooks_calldata_with_signature` carries either shape
through to the factory's `bytes` argument. The `isDelegateCall = true` setting
is opt-in only via the `Call::delegate_call` builder, which requires a
`// SAFETY:` comment in the preceding three lines of the call site.

### EIP-712 hashing

The COW Shed EIP-712 hashing path delegates to alloy primitives
end-to-end. The `Call` and `ExecuteHooks` typed-data structs are
declared via the `alloy_sol_types::sol!` macro in
`crates/cow-shed/src/eip712/sol_types.rs`; the macro emits the canonical
type strings at expansion time and rejects any whitespace insertion or
declaration-order swap at macro expansion. `cow_shed_eip712_domain`
constructs an `alloy_sol_types::Eip712Domain` (name `"COWShed"`, the
deployed version string, the caller-supplied chain id, the proxy
address, and no salt) for callers that need the typed-data domain
value; `cow_shed_domain_separator` is the thin convenience wrapper that
returns the same domain's `.separator()` byte for callers that only
need the per-proxy separator. `execute_hooks_signing_hash` builds the
`ExecuteHooks` struct from the input slice and delegates to
`<ExecuteHooks as SolStruct>::eip712_signing_hash(&domain)`, which
composes the canonical EIP-712 envelope (`keccak256(0x19 || 0x01 ||
domain_separator || hashStruct(message))`) end-to-end through
`alloy_primitives::keccak256` with no cow-owned envelope code. Callers
that need the EIP-712 type-hash bytes call
`<T as SolStruct>::eip712_type_hash` on the matching struct. The
`parity/fixtures/cow_shed/domain_separator.json` and
`parity/fixtures/cow_shed/execute_hooks_digest.json` fixtures lock the
wire-byte contract. The type-hash parity contract test asserts the
macro-emitted accessors equal keccak of the canonical type strings via
a hand-rolled `sha3::Keccak256` helper, so the assertion runs against an
independent keccak path rather than the alloy crate's own.

### Call type identity

The COW Shed crate carries one `Call` type definition. The
macro-emitted `Call` in `crates/cow-shed/src/eip712/sol_types.rs` is the
single source of truth: the same sol! block declares the canonical
`ExecuteHooks` typed-data envelope plus the `COWShed` proxy and
`COWShedFactory` factory interfaces, so the `Call[]` arguments on every
hook-bearing function (`executeHooks`, `executePreSignedHooks`,
`isPreSignedHooks`, `preSignHooks`, `trustedExecuteHooks` on the proxy,
and the factory `executeHooks`) reference the same generated Rust type.
The `crates/cow-shed/src/bindings/shed.rs` and
`crates/cow-shed/src/bindings/factory.rs` modules re-export the
canonical interfaces under
`cow_sdk_cow_shed::bindings::shed::COWShed` and
`cow_sdk_cow_shed::bindings::factory::COWShedFactory`, and
`crates/cow-shed/src/types/call.rs` re-exports the canonical struct as
the crate-level `cow_sdk_cow_shed::Call` alias. The ergonomic builder
helpers (`Call::new(target, value, call_data)`, `Call::allow_failure()`,
`Call::delegate_call()`) are inherent `const fn` methods on `Call`, so
call-site code reads in snake-case while the sol-generated struct keeps its
camelCase Solidity field names. The four
representative rows in
`parity/fixtures/cow_shed/execute_hooks_calldata.json` (single-call,
three-call medium fan-out, five-call max fan-out, and empty-`callData`
edge case) lock the wire-byte contract for both the factory
`executeHooks` and the proxy `executeHooks` ABI calldata paths.

### CREATE2 derivation

Proxy address derivation in
`crates/cow-shed/src/address/mod.rs::proxy_of` routes through
[`alloy_primitives::Address::create2`], which assembles the canonical
EIP-1014 preimage (`0xff || factory || salt || init_code_hash`) and
keccak256-hashes it internally. The salt is the user address left-padded
with twelve zero bytes to fill a 32-byte word via
[`alloy_primitives::Address::into_word`]; the init-code hash concatenates
the embedded per-version proxy creation code with the canonical ABI
encoding of the `(implementation, user)` constructor tuple via
[`alloy_sol_types::SolValue::abi_encode`] and hashes the result with
[`alloy_primitives::keccak256`]. The implementation address is selected
by `implementation_for(version, factory)`, which returns the Gnosis
implementation when `version = 1.0.1` and the factory equals the
deployed Gnosis factory address, and the default implementation in
every other case. The thirty rows in
`parity/fixtures/cow_shed/proxy_addresses.json` (five users across two
deployed versions across three chains) lock the per-row salt,
init-code-hash, and proxy-address byte contract.

### EOA signature byte order

The ERC-2098 compact signature decoder
`cow_sdk_cow_shed::eoa_signature_from_compact` concatenates the
caller-supplied `r_compact` and `vs` 32-byte arrays into the 64-byte
ERC-2098 input and routes through
[`alloy_primitives::Signature::from_erc2098`], which extracts the
`y_parity` bit from the high bit of `vs[0]`, masks it out of the
recovered `s`, and constructs the canonical
[`alloy_primitives::Signature`].
[`alloy_primitives::Signature::as_bytes`] then emits the 65-byte
`r || s || v` layout with `v = 27 + y_parity ∈ {27, 28}`. The four
representative rows in
`parity/fixtures/cow_shed/eoa_signature_byte_order.json`
(`v_27_low_bit`, `v_28_high_bit`, `edge_max_s_value`, and a
real-shaped `v = 28` signature) carry the matched ERC-2098 compact
input and the canonical packed signature for each case, locking the
wire-byte contract end-to-end.

## Evidence

Primary implementation points:

- `crates/cow-shed/src/eip712/sol_types.rs`
- `crates/cow-shed/src/bindings/`
- `parity/source-lock.yaml`
- `crates/contracts/abi/cow-shed/proxy-creation-code/v1.0.0.bin`
- `crates/contracts/abi/cow-shed/proxy-creation-code/v1.0.0.bin.sha256`
- `crates/contracts/abi/cow-shed/proxy-creation-code/v1.0.1.bin`
- `crates/contracts/abi/cow-shed/proxy-creation-code/v1.0.1.bin.sha256`
- `crates/contracts/abi/cow-shed/version-call-results.json`
- `crates/contracts/registry.toml`
- `crates/contracts/build.rs` (`validate_cow_shed_proxy_artifacts`)
- `parity/cow-shed-invariants.md`
- `parity/fixtures/cow_shed/`

Primary regression coverage:

- `crates/contracts/tests/schema_v2_success.rs`
- `crates/contracts/tests/schema_v2_rejection.rs`

Validation surface:

```text
cargo test -p cow-sdk-contracts --all-features
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- validate --source-lock parity/source-lock.yaml
```
