# ADR 0049: COW Shed Account-Abstraction Proxy

- Status: Accepted (amended)
- Date: 2026-05-15
- Last reviewed: 2026-06-12
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: cow-shed, account-abstraction, version-forwarding, proxy-derivation
- Related: [ADR 0008](0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md), [ADR 0010](0010-runtime-neutral-async-and-transport-posture.md), [ADR 0012](0012-alloy-sol-bindings-and-registry-authority.md), [ADR 0048](0048-composable-conditional-order-framework.md), [ADR 0050](0050-eip1271-signature-blob-encoding.md), [ADR 0051](0051-signing-owned-eip1271-signature-provider-trait.md), [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md)

## Context

The COW Shed account-abstraction proxy is a CREATE2-deployed per-user ERC-1967
proxy that fronts every interaction with a designated implementation contract.
On the canonical eleven supported chains the deployed bytecode of the
implementation returns `"1.0.1"` from `VERSION()` even though the upstream
Solidity source HEAD has been advanced to `"2.1.0"`. The SDK must sign and
derive proxy addresses against deployed reality, not against source HEAD.
Signing against `"2.1.0"` would produce signatures that fail verification on
every live proxy.

The COW Shed surface also has four layered source authorities that must be
ranked, because all four disagree in places: the deployed factory ABI in
`cow-sdk/packages/cow-shed/src/abi/CowShedFactoryAbi.ts` ships the 2-arg
`initializeProxy(address,bool)` selector that matches deployed bytecode; the
upstream Solidity in `cow-shed/src/COWShedFactory.sol` documents a 1-arg
`initializeProxy(address)` form that does not match deployed bytecode; the
`cow-shed/networks.json` deployment registry pins factory and implementation
addresses per chain; and the version-keyed constants in
`cow-sdk/packages/cow-shed/src/const.ts` pin per-version factory and
implementation addresses for legacy snapshots.

A separate Gnosis-only `COWShedForComposableCoW` forwarder bridges the
composable framework to the COW Shed proxy on chain id 100 only. No other
supported chain carries this forwarder, so the SDK must gate the bridge
behavior on chain id.

The upstream TypeScript SDK has a known bug at
`cow-sdk/packages/cow-shed/src/CowShedSdk.ts:172` where `new CoWShedHooks(chainId, customOptions)`
silently drops the caller-selected `version` and defaults every instance to
`COW_SHED_LATEST_VERSION`. The Rust SDK must not mirror this bug.

## Decision

The COW Shed surface ships as the `cow_sdk_contracts::cow_shed` module, gated
behind the off-by-default `cow-shed` feature of `cow-sdk-contracts` and exposed
through the facade-level `cow-shed` feature as `cow_sdk::cow_shed`. It is an
additive capability per ADR 0008 and is never on the default `cow-sdk`
dependency closure. (Originally shipped as the standalone `cow-sdk-cow-shed`
leaf crate; see the 2026-06-09 amendment.)

### Version Forwarding

`CowShedVersion` has variants `V1_0_0` and `V1_0_1` with `V1_0_1` as
`Default::default()`. The SDK signs and derives proxy addresses against the
deployed `VERSION()` return value captured in
`parity/fixtures/cow_shed/version_calls.json`. The version selected
by the caller threads through every internal builder; no helper may construct
a downstream object without forwarding the caller-selected version.

A regression test asserts that distinct `CowShedVersion` variants produce
distinct CREATE2 proxy addresses for the same user. This regression makes the
caller-selected version detectable at every derivation layer.

### Four-Layer Source Authority

The source authority order is, in descending priority:

1. Deployed-runtime factory ABI from
   `cow-sdk/packages/cow-shed/src/abi/CowShedFactoryAbi.ts` (the canonical
   2-arg `initializeProxy(address,bool)` selector that matches deployed
   bytecode).
2. Solidity source-level semantics from `cow-shed/src/*.sol` (documented as
   divergent reference only; the 1-arg form does not match deployed bytecode).
3. Deployment rows from `cow-shed/networks.json` (per-chain factory and
   implementation addresses).
4. Version-keyed pinning constants from
   `cow-sdk/packages/cow-shed/src/const.ts` (per-version factory and
   implementation addresses).

### Hook Type Strings and Signature Order

EIP-712 type strings for the COW Shed hook structure carry no whitespace
between commas in declaration order. The canonical strings are
`Call(address target,uint256 value,bytes callData,bool allowFailure,bool isDelegateCall)`
and
`ExecuteHooks(Call[] calls,bytes32 nonce,uint256 deadline)Call(address target,uint256 value,bytes callData,bool allowFailure,bool isDelegateCall)`.

EOA signature byte order is `r || s || v` (not the standard `v || r || s`).
The canonical 65-byte `r || s || v` layout is produced and validated by
`cow_sdk_contracts::RecoverableSignature`, whose `parse_bytes` rejects any
non-65-byte input and any recovery byte outside `{0, 1, 27, 28}` (ADR 0022). A
smart-contract (EIP-1271) owner instead supplies a variable-length signature
blob; `encode_execute_hooks_calldata_with_signature` carries either shape
through to the factory's `bytes` argument unchanged.

`isDelegateCall = true` is opt-in only via the explicit `Call::delegate_call`
builder, and each call site must carry a `// SAFETY:` comment in the
immediately preceding three lines justifying the delegatecall.

`executePreSignedHooks` indexes by struct hash without domain prefix because
the proxy deduplicates pre-signed batches inside the implementation, not at
the EIP-712 domain layer.

### Gnosis-Only Forwarder Gate

The `COWShedForComposableCoW` forwarder is deployed on Gnosis Chain (chain id
100) only. Helpers that construct or interact with the forwarder must reject
every other chain id with the typed
`CowShedError::COWShedForComposableCoWGnosisOnly { chain }` variant. The gate
is enforced by the off-by-default `cow-shed-gnosis` Cargo feature.

### Crate-Graph Invariants

The `cow-shed` feature of `cow-sdk-contracts` adds only the `cow-sdk-app-data`
dependency on top of the crate's `cow-sdk-core` foundation. It MUST NOT pull
`cow-sdk-trading`, `cow-sdk-orderbook`, `cow-sdk-subgraph`,
`cow-sdk-browser-wallet`, `alloy-provider`, `alloy-signer-local`, `reqwest`, or
`tokio` runtime features into the `cow-sdk-contracts` closure. The negative-edge
invariant `cow-sdk-contracts[cow-shed] ⇏ cow-sdk-trading` is asserted via
`cargo metadata` and the workspace dependency-invariant checks in CI.

## Why

The deployed-bytecode-first authority order avoids a class of signing failures
that appears only at signature verification time. Source HEAD declares
`"2.1.0"` but the deployed implementation returns `"1.0.1"`; a signature
produced against `"2.1.0"` would fail verification on every live proxy. The
authority order keeps the SDK's signing output trustworthy.

Threading the caller-selected version through every internal builder prevents
the silent-drop bug present in the upstream TypeScript SDK. The regression
test makes the contract checkable at every derivation layer: if a future
helper drops the version, distinct `CowShedVersion` inputs collapse to the
same proxy address and the regression fails.

The Gnosis-only forwarder gate ships as a typed error variant rather than a
runtime panic because wrong-chain calls are a caller bug that the type system
can surface. The chain check happens in the constructor; downstream methods
see only an already-validated forwarder handle.

The negative-edge invariant against `cow-sdk-trading` keeps cow-shed an
additive leaf per ADR 0008. cow-shed and trading are peer leaves; trading is
not a dependency direction for cow-shed.

## Must Remain True

- Public surface: `CowShedVersion::V1_0_1` is the default; the
  `parity/fixtures/cow_shed/deployments.json` artifact pins the per-version
  factory/implementation pairs and the deployed `VERSION()` domain strings
  (see the 2026-06-12 amendment).
- Runtime and support: every internal builder forwards the caller-selected
  version. The regression test asserts distinct version variants produce
  distinct proxy addresses.
- Crate graph: `cargo metadata` continues to prove the `cow-sdk-contracts`
  `cow-shed` feature closure excludes `cow-sdk-trading`, `cow-sdk-orderbook`,
  `cow-sdk-subgraph`, and `alloy-provider`.
- Validation and review: the COW Shed contract bindings audit and the COW
  Shed app-data integration audit cross-link this ADR. Both stay `Current`
  whenever the audited surface moves.
- Cost: the SDK must not mirror the upstream version-drop bug; any future
  helper that drops the caller-selected version is a regression of this ADR.

## Alternatives Rejected

- Sign against source HEAD `"2.1.0"`: every signature would fail verification
  on live proxies that return `"1.0.1"`.
- Mirror the upstream `CoWShedHooks` constructor without version forwarding:
  this would mirror a known upstream bug and break any caller that needs
  per-version signing.
- Bind the SDK to the 1-arg `initializeProxy(address)` Solidity selector:
  the deployed bytecode targets the 2-arg form; the 1-arg form would fail
  every deployment call.
- Statically cache `keccak256(PROXY_CREATION_CODE)` once and reuse it across
  users: the init-code hash is per-`(implementation, who)` and depends on
  `abi.encode(implementation, who)` constructor arguments. A static cache
  would produce a single proxy address for every user.
- Allow `COWShedForComposableCoW` on every chain: only Gnosis Chain has the
  forwarder deployed; allowing every chain would produce silent address-zero
  calls.

## Links

- [Architecture](../architecture.md)
- [Principles](../principles.md)
- [COW Shed Contract Bindings Audit](../audit/cow-shed-contract-bindings-audit.md)
- [COW Shed App Data Integration Audit](../audit/cow-shed-app-data-integration-audit.md)
- [ADR 0048](0048-composable-conditional-order-framework.md)
- [ADR 0050](0050-eip1271-signature-blob-encoding.md)
- [ADR 0051](0051-signing-owned-eip1271-signature-provider-trait.md)

**Proven by:**

- [COW Shed Contract Bindings Audit](../audit/cow-shed-contract-bindings-audit.md)
- [COW Shed App Data Integration Audit](../audit/cow-shed-app-data-integration-audit.md)

## Amendment 2026-05-22: canonical primitive layer (per ADR 0052)

The cow-shed EIP-712 typed-data structures (`Call` with
`target/value/callData/allowFailure/isDelegateCall` fields and
`ExecuteHooks` over `Call[]`, `nonce`, `deadline`) are macro-emitted by
`alloy_sol_types::sol!` per
[ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md). The
proxy address derivation routes through `alloy_primitives::Address`
plus `alloy_primitives::keccak256` for the CREATE2 init-code hash. The
EOA signature byte order on the cow-shed hook signature payload is
`r || s || v` (assembled through `alloy_primitives::Signature::from_erc2098`
plus `Signature::as_bytes`); the whitespace-free EIP-712 type strings
between commas, the `isDelegateCall = true` safety-comment-gated
opt-in builder, and the version-forwarding contract on
`CowShedVersion` stay unchanged.

## Amendment 2026-05-25: EIP-712 envelope consolidation (per ADR 0052)

The COW Shed signing-digest path collapses onto
`<ExecuteHooks as alloy_sol_types::SolStruct>::eip712_signing_hash(&domain)`
per [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md).
The previous three-function flow (`cow_shed_domain_separator` →
`execute_hooks_message_hash` → `hash_to_sign`) is replaced by a single
public entry point `execute_hooks_signing_hash(&domain, &calls, nonce,
deadline) -> B256` plus the new domain builder
`cow_shed_eip712_domain(chain, version, proxy) -> Eip712Domain`.
`cow_shed_domain_separator` is retained as a thin wrapper that returns
the same domain's `.separator()` byte for callers that only need the
per-proxy separator. The cow-owned hand-rolled 66-byte envelope is
removed; the canonical envelope is now produced entirely by the
macro-emitted `SolStruct` impl, which composes the standard
`keccak256(0x19 || 0x01 || domain_separator || hashStruct(message))`
through `alloy_primitives::keccak256`. The
`parity/fixtures/cow_shed/execute_hooks_digest.json` rows confirm
byte-identical output across every supported chain and version row.

## Amendment 2026-06-04: realized public surface

The `cow-sdk-cow-shed` body ships with the building blocks plus a high-level
`CowShedHooks` orchestrator: `CowShedHooks::new(chain)` (accepting a
`SupportedChainId` or `DeploymentChainId`) → `sign(&signer, &calls, nonce,
deadline)` resolves the owner from the owned `Signer`, derives the proxy via
`proxy_for`, signs the `ExecuteHooks` payload through `sign_typed_data_payload`,
and returns a `SignedCowShedCall { shed_account, factory, factory_calldata }`
that submits directly or becomes an app-data hook via `to_app_data_hook`. The
`executeHooks` encoder is owner-agnostic: `encode_execute_hooks_calldata_with_signature`
accepts a 65-byte EOA signature or an EIP-1271 contract-signature blob, and
`encode_execute_hooks_calldata_signed` is the typed EOA convenience. The chain
keyed `cow_shed_factory` / `cow_shed_implementation` / `proxy_for` lookups and
`CowShedVersion::ALL` (current generation first) support multi-version proxy
discovery. The `Call` hook-call builders (`new`, `allow_failure`,
`delegate_call`) are inherent `const fn` methods on `Call`. The version
forwarding regression asserting distinct `CowShedVersion` variants derive
distinct proxies ships as `distinct_versions_derive_distinct_proxies` in
`crates/contracts/src/cow_shed/address/mod.rs`.

## Amendment 2026-06-09: folded into `cow-sdk-contracts`

The COW Shed surface moved from the standalone `cow-sdk-cow-shed` crate into the
`cow_sdk_contracts::cow_shed` module, gated behind the off-by-default `cow-shed`
feature of `cow-sdk-contracts` (with `cow-shed-gnosis` lifting the Gnosis
forwarder). The public types are unchanged and are reached through
`cow_sdk_contracts::cow_shed::*`; the facade still re-exports them as
`cow_sdk::cow_shed` behind its `cow-shed` feature. With the feature off, the
default `cow-sdk-contracts` surface and dependency closure are unchanged, so the
capability stays off the default `cow-sdk` closure exactly as before. The
reserved `cow-shed-ens` feature and its `COWShedFactoryEns` binding — never
deployed and never consumed — were dropped in the same change, and the module's
`sol!` ABI definitions are consolidated into `cow_shed/bindings.rs`. This sheds
one published crate without altering any COW Shed contract.

## Amendment 2026-06-12: deployed-generation re-scope (corrects the record)

A full-surface review against the vendored upstream git history (tags
v1.0.0/v1.0.1/v2.0.0/v2.1.0) found that parts of this ADR's context and of
the shipped module described a **chimera of two upstream generations**, and
corrected both. What changed:

- **The "eleven chains return 1.0.1" context claim was wrong for Gnosis.**
  The chain-100 rows in upstream `networks.json` HEAD (factory `0x4f4350bf…`,
  implementation `0x62d3a7ff…`) are the **v2.0.0-generation** redeploy shipped
  with the composable-cow work; that implementation's `VERSION()` is
  `"2.0.0"`. The fixture row claiming it decodes `"1.0.1"` was never actually
  probed. The real v1.0.1 Gnosis deployment is the canonical pair
  (`0x312f92fe…`/`0xa2704cf5…`, recorded in `networks.json` at the v1.0.1
  tag) — identical to every other chain. The Gnosis special case in the
  address module (wrong creation code **and** wrong domain version for the v2
  factory) is deleted; the deployed pairs are chain-uniform per version, so
  `cow_shed_factory`/`cow_shed_implementation`/`proxy_for` are keyed by
  version alone and the derived proxy address is chain-independent.
- **The pre-sign family is v2-only and is no longer bound.** v1.0.x deploys
  no `executePreSignedHooks`/`preSignHooks`/`setPreSignStorage`/… selectors,
  so the previous bindings (and the `encode_execute_pre_signed_hooks_calldata`
  encoder) targeted functions that revert on every supported deployment. The
  bindings now mirror the deployed v1.0.x surface exactly (2-arg
  `initialize(address,bool)`, `VERSION()`, `claimWithResolver`,
  `OnlyTrustedExecutor`/`OnlyAdminOrTrustedExecutorOrSelf`/`DeadlineElapsed`/
  `NonceAlreadyUsed` errors), as a documented subset excluding the ENS
  resolver reads. The `executePreSignedHooks` design prose in this ADR's
  Decision section is superseded accordingly; the surface returns with a real
  `V2_x` version family when upstream rolls v2 out beyond Gnosis.
- **The Gnosis-only forwarder gate section is superseded.** The
  `COWShedForComposableCoW` binding, the `cow-shed-gnosis` feature, and the
  `CowShedError::COWShedForComposableCoWGnosisOnly` variant are removed: the
  forwarder belongs to the v2 generation, no helper ever constructed the
  gate variant (the documented enforcement was phantom), and the TS arbiter
  ships no composable surface.
- **ERC-2098 compact helpers are consolidated on `RecoverableSignature`.**
  The module-local `compact_signature`/`eoa_signature_from_compact`/
  compact-form encoder trio duplicated `RecoverableSignature::{to,parse}_erc2098`
  and the hand-rolled packer corrupted high-s signatures (it OR-ed the parity
  bit into raw `s` without BIP-62 low-s normalization). The canonical pair —
  which normalizes on encode, mapping a high-s input to its canonical twin —
  is the only compact surface; the on-chain decoder accepts exactly the
  65-byte `r || s || v` form either way.
- **Provenance re-pin.** `parity/source-lock.yaml` pins cow-shed at the
  v1.0.1 tag commit (`e15a131d…`), the generation the bindings mirror, and
  the cow-sdk row gains the cow-shed package producer paths
  (deployed-runtime ABI, per-version constants including the proxy creation
  code, and the CREATE2 golden vectors). The fabricated selector values in
  `canonical_selectors.json` (4/4 `factory_methods` rows) were corrected from
  independent keccak derivations, and the selector parity test now asserts
  fixture == independent keccak == `SolCall::SELECTOR` for every bound
  function instead of literal-asserting fixture values against themselves.
  `version_calls.json` is replaced by `deployments.json` (the honest record:
  pinned constants, not unperformed eth_calls), and the proxy-address fixture
  drops its information-free chain axis in favor of the TS arbiter's two
  external anchor vectors plus derived regression rows.
- **Dead public surface removed.** The consumer-facing `Deadline`/`Nonce`
  strategy enums, the `ProxyAddress` alias, the `SigSource` enum, and the
  14 never-constructed on-chain-mirror `CowShedError` variants are deleted;
  `CowShedError` now carries exactly the signing-path variants the
  orchestrator produces, and on-chain revert taxonomies stay on the `sol!`
  interfaces' generated error enums.

The four-layer source authority order stands unchanged — this amendment is
that order applied correctly: layer 1 (deployed-runtime ABI) and layer 4
(version-keyed constants) agree on the v1.0.x generation, and the layer-3
`networks.json` HEAD rows for Gnosis describe a different, unsupported
generation rather than a chain divergence within the supported one.
