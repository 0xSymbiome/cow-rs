# COW Shed Contract Bindings Audit

Status: Current
Last reviewed: 2026-06-12
Owning surface: inline COW Shed `alloy::sol!` bindings, proxy creation-code artifacts, deployed-generation address record, and the CREATE2/EIP-712/selector parity evidence
Refresh trigger: Refresh when COW Shed deployments, proxy creation code, factory ABIs, hook type strings, the deployed `VERSION()` constants, or the upstream commit pins for the COW Shed sources change.
Related docs:
- [ADR 0049](../adr/0049-cow-shed-account-abstraction-proxy.md)
- [ADR 0050](../adr/0050-eip1271-signature-blob-encoding.md)
- [ADR 0051](../adr/0051-signing-owned-eip1271-signature-provider-trait.md)
- [ADR 0052](../adr/0052-alloy-primitives-canonical-primitive-layer.md)
- [COW Shed App-Data Integration Audit](cow-shed-app-data-integration-audit.md)

## Scope

This audit covers:

- the inline COW Shed `alloy::sol!` bindings that mirror the **deployed
  v1.0.x generation** — the upstream `cowdao-grants/cow-shed` sources at the
  v1.0.1 tag, pinned by commit in `parity/source-lock.yaml`, cross-checked
  against the deployed-runtime factory ABI shipped by the pinned TypeScript
  arbiter (`packages/cow-shed/src/abi/CowShedFactoryAbi.ts`);
- the per-version proxy creation-code artifacts embedded by the module,
  byte-identical to the arbiter's `COW_SHED_PROXY_INIT_CODE` constants and
  digest-pinned by the proxy-address parity fixture;
- the per-version deployed factory/implementation record
  (`parity/fixtures/cow_shed/deployments.json`) — deterministic CREATE2
  deployments, identical on every supported chain, so the record carries no
  chain axis;
- the selector record (`parity/fixtures/cow_shed/canonical_selectors.json`)
  covering every bound function of both interfaces;
- the EIP-712 type strings, domain/digest hashing, and the EOA signature
  byte order `r || s || v` with the ERC-2098 compact round-trip on
  `RecoverableSignature`.

It does not cover the COW Shed hook metadata schema integration with the
app-data crate; that boundary is governed by the
[COW Shed App-Data Integration Audit](cow-shed-app-data-integration-audit.md).

## Generation posture

Upstream cow-shed has moved past the deployed generation: v2.0.0 purged ENS
(1-arg `initializeProxy(address)`) and added the pre-sign flow; v2.1.0 added
the ComposableCoW forwarder. The only v2-generation deployment recorded
upstream is the Gnosis chain-100 redeploy (factory `0x4f4350bf…`,
implementation `0x62d3a7ff…`, EIP-712 domain version `"2.0.0"`), which is
**outside the supported `CowShedVersion` family**; the v1.0.1 generation
remains deployed on Gnosis at the canonical pair recorded in `networks.json`
at the pinned tag. Per ADR 0049 the SDK binds deployed reality: no v2-only
function, event, or error is bound, and nothing in the module is keyed by
chain. A future upstream v2 rollout lands as new `#[non_exhaustive]`
`CowShedVersion` variants with their own creation-code artifacts and domain
version strings.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Inline bindings | Both `sol!` interfaces declare only functions, events, and errors present in the deployed v1.0.x sources at the pinned tag (factory: `executeHooks`, 2-arg `initializeProxy`, `proxyOf`, `ownerOf`, `implementation`; shed: `executeHooks`, `trustedExecuteHooks`, `claimWithResolver`, admin/nonce/domain reads, `VERSION`, 2-arg `initialize`); the ENS resolver read surface and constructor-only errors are documented exclusions | Conforms |
| Selector record | Every row in `canonical_selectors.json` is triple-checked by `selector_parity_cow_shed_contract.rs`: an independent `sha3::Keccak256` derivation of the canonical signature, the pinned fixture value, and the macro-emitted `SolCall::SELECTOR` constant must agree; row counts pin the bound surface | Conforms |
| Proxy creation-code | `v1.0.0.bin` (881 bytes) and `v1.0.1.bin` (829 bytes) are byte-identical to the TS arbiter's `COW_SHED_PROXY_INIT_CODE` constants and pinned by length + keccak256 in `proxy_addresses.json`, asserted by `proxy_address_parity_contract.rs::creation_code_blobs_are_digest_pinned` | Conforms |
| Deployment record | `deployments.json` pins the per-version factory/implementation pairs and the deployed `VERSION()` domain strings; `deployment_address_parity_contract.rs` locks the version-keyed lookups and `CowShedVersion::version_str` against it | Conforms |
| CREATE2 derivation | `proxy_of`/`proxy_for` route through `alloy_primitives::Address::create2` over the user-word salt and the `keccak256(creationCode ‖ abi.encode(implementation, user))` init-code hash; the parity rows include the TS arbiter's own golden vector and its custom-options mock vector as external anchors | Conforms |
| EIP-712 hashing | Domain separator and signing digest are produced by `alloy_sol_types::Eip712Domain::separator` and `<ExecuteHooks as SolStruct>::eip712_signing_hash`; `domain_separator.json` and `execute_hooks_digest.json` lock the bytes, and the type hashes are re-derived with an independent keccak in the type-hash parity test | Conforms |
| Call type identity | One macro-emitted `Call` backs typed-data hashing, both interfaces, and the calldata builders; the four `execute_hooks_calldata.json` rows lock the factory and proxy `executeHooks` wire bytes | Conforms |
| EOA signature byte order | `r || s || v` with `v ∈ {27, 28}`, the only shape the on-chain `decodeEOASignature` accepts; the ERC-2098 compact pair lives solely on `RecoverableSignature` (`to_erc2098` normalizes to low-s per BIP-62 — a high-s input maps to its canonical twin — and `parse_erc2098` is the inverse), locked by `eoa_signature_byte_order.json` including an explicit high-s normalization row | Conforms |

## Current Contract

### Inline bindings

The COW Shed bindings are inline `alloy::sol!` interfaces in
`crates/contracts/src/cow_shed/bindings.rs` mirroring the deployed v1.0.x
generation, pinned by commit (the v1.0.1 tag) under `repositories:` in
`parity/source-lock.yaml` and cross-checked against the deployed-runtime
factory ABI the TypeScript arbiter ships. The mirror is a deliberate,
documented subset: the factory's inherited ENS resolver reads
(`initializeEns`, `addr`, `name`, `baseName`, `baseNode`, the
resolution-node getters, `supportsInterface`) and the constructor-only
`NoCodeAtImplementation` error are out of scope for hook execution and proxy
discovery. Every bound symbol exists byte-for-byte in the deployed runtime;
the v2-only pre-sign family is not bound because no deployed v1.0.x contract
dispatches those selectors. The error sets mirror the deployed sources per
contract, including the library errors that surface through `executeHooks`
(`DeadlineElapsed`, `NonceAlreadyUsed`) so revert decoding through the
generated error enums matches on-chain behavior.

### Selector record

`parity/fixtures/cow_shed/canonical_selectors.json` carries one row per
bound function (5 factory, 11 shed) plus the canonical EIP-712 type strings
and the signature byte order. The contract test derives every selector from
its canonical signature with `sha3::Keccak256` (an independent keccak
implementation, not alloy's), compares it to the pinned fixture value, and
asserts the macro-emitted `SolCall::SELECTOR` constant equals both — so the
fixture, the preimages, and the bindings cannot drift apart silently, and a
fixture row without a binding (or a renamed binding) fails loudly. Group row
counts pin the bound surface size.

### Proxy creation-code

Per-version proxy creation-code artifacts ship at
`crates/contracts/src/cow_shed/address/proxy-creation-code/{v1.0.0,v1.0.1}.bin`,
embedded via `include_bytes!`. They are byte-identical to the TS arbiter's
`COW_SHED_PROXY_INIT_CODE` constants at the pinned cow-sdk commit, and the
proxy-address parity fixture pins each blob by byte length and keccak256.
The blobs store the deployer bytecode prefix only; the full init code is
completed per derivation with `abi.encode(implementation, user)`. The
deployed 2-arg `initialize(address,bool)` selector (`0x400ada75`) is embedded
in both blobs as the pre-initialization call guard, corroborating the
generation match.

### Deployment record

`parity/fixtures/cow_shed/deployments.json` records, per supported version,
the factory, the implementation, and the deployed `VERSION()` constant that
doubles as the EIP-712 domain version. The pairs are deterministic CREATE2
deployments, identical on every chain the generation is deployed to, so the
version-keyed lookups in `crates/contracts/src/cow_shed/address/mod.rs` carry
no chain parameter and the record carries no chain axis. Chain id enters the
COW Shed story only through the EIP-712 signing domain.

### CREATE2 derivation

`proxy_of(version, factory, user)` pairs an explicit factory with the
version's canonical implementation; `proxy_for(version, user)` uses the
canonical factory. Both route through
`alloy_primitives::Address::create2` with the user address as the 32-byte
salt and `keccak256(creationCode ‖ abi.encode(implementation, user))` as the
init-code hash. The parity rows in `proxy_addresses.json` include two
external anchors from the arbiter's own test suite — the canonical v1.0.1
golden vector and the custom-options mock (v1.0.0 creation code with a
non-canonical factory/implementation pair, exercising the explicit
`init_code_hash` + `create2` path) — plus derived regression rows; the
anchors transitively prove the creation-code bytes and the formula against
an authority outside this repository.

### Hook type strings and EIP-712 hashing

The canonical type strings are
`Call(address target,uint256 value,bytes callData,bool allowFailure,bool isDelegateCall)`
and
`ExecuteHooks(Call[] calls,bytes32 nonce,uint256 deadline)Call(address target,uint256 value,bytes callData,bool allowFailure,bool isDelegateCall)`,
whitespace-free between commas in declaration order.
`cow_shed_eip712_domain` builds the `alloy_sol_types::Eip712Domain`
(name `"COWShed"`, the deployed version string, chain id, proxy address, no
salt); `execute_hooks_signing_hash` delegates to
`<ExecuteHooks as SolStruct>::eip712_signing_hash`, composing the canonical
envelope through `alloy_primitives::keccak256` with no cow-owned envelope
code. `domain_separator.json` and `execute_hooks_digest.json` lock the
per-chain bytes (the chain-100 rows use the canonical, chain-independent
proxy with domain version `"1.0.1"`), and the type-hash parity test asserts
the macro accessors equal an independent `sha3::Keccak256` of the canonical
strings.

### EOA signature byte order

The on-chain `decodeEOASignature` accepts exactly 65 bytes read as
`r || s || v`; `RecoverableSignature::parse_bytes` produces and validates
that shape (recovery byte in `{0, 1, 27, 28}`, ADR 0022), and
`encode_execute_hooks_calldata_with_signature` carries either a 65-byte EOA
signature or a variable-length EIP-1271 blob through to the factory's
`bytes` argument unchanged, keeping the proxy's length-based dispatch
reachable for both owner kinds. The ERC-2098 compact representation lives
solely on `RecoverableSignature`: `to_erc2098` (alloy `as_erc2098`)
normalizes `s` to low-s per BIP-62 before packing the parity bit — a high-s
input maps to its canonical twin `(r, n − s, !y_parity)`, the same
(digest, signer) validity under ECDSA malleability — and `parse_erc2098` is
the inverse. `eoa_signature_byte_order.json` locks both directions,
including an explicit high-s row proving normalize-on-encode.

## Evidence

Primary implementation points:

- `crates/contracts/src/cow_shed/bindings.rs`
- `crates/contracts/src/cow_shed/address/mod.rs`
- `crates/contracts/src/cow_shed/address/proxy-creation-code/v1.0.0.bin`
- `crates/contracts/src/cow_shed/address/proxy-creation-code/v1.0.1.bin`
- `crates/contracts/src/cow_shed/calls.rs`
- `crates/contracts/src/signature.rs` (`RecoverableSignature::{to,parse}_erc2098`)
- `parity/source-lock.yaml`
- `parity/fixtures/cow_shed/` (`canonical_selectors`, `deployments`,
  `proxy_addresses`, `domain_separator`, `execute_hooks_digest`,
  `execute_hooks_calldata`, `eoa_signature_byte_order`)

Primary regression coverage:

- `crates/contracts/tests/selector_parity_cow_shed_contract.rs`
- `crates/contracts/tests/deployment_address_parity_contract.rs`
- `crates/contracts/tests/proxy_address_parity_contract.rs`
- `crates/contracts/tests/domain_separator_parity_contract.rs`
- `crates/contracts/tests/eip712_message_hash_parity_contract.rs`
- `crates/contracts/tests/eip712_type_hash_parity_contract.rs`
- `crates/contracts/tests/signed_calldata_parity_contract.rs`
- `crates/contracts/tests/eoa_signature_byte_order_contract.rs`
- `tests/cow_shed_typed_data_digest.rs`

Validation surface:

```text
cargo test -p cow-sdk-contracts --features cow-shed
cargo parity-validate --source-lock parity/source-lock.yaml
```
