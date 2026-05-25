# ADR 0049: COW Shed Account-Abstraction Proxy

- Status: Accepted (amended)
- Date: 2026-05-15
- Last reviewed: 2026-05-22
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

`cow-sdk-cow-shed` is an additive leaf crate per ADR 0008. The crate is
opt-in behind the facade-level `cow-shed` feature and is never on the default
`cow-sdk` dependency closure.

### Version Forwarding

`CowShedVersion` has variants `V1_0_0` and `V1_0_1` with `V1_0_1` as
`Default::default()`. The SDK signs and derives proxy addresses against the
deployed `VERSION()` return value captured in
`crates/contracts/abi/cow-shed/version-call-results.json`. The version selected
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
The `SignedCowShedHook::signature` field is a fixed-length 65-byte array in
that order; a trybuild fixture enforces it at the type level.

`isDelegateCall = true` is opt-in only via an explicit builder method that
requires a `// SAFETY:` comment in the immediately preceding three lines of
the call site. A compile-fail fixture rejects use without the safety comment.

`executePreSignedHooks` indexes by struct hash without domain prefix because
the proxy deduplicates pre-signed batches inside the implementation, not at
the EIP-712 domain layer.

### Gnosis-Only Forwarder Gate

The `COWShedForComposableCoW` forwarder is deployed on Gnosis Chain (chain id
100) only. Helpers that construct or interact with the forwarder must reject
every other chain id with the typed
`CowShedError::COWShedForComposableCoWGnosisOnly { chain }` variant. The gate
is enforced by the typed Cargo feature `cow-shed-gnosis` and an ENS-related
feature `cow-shed-ens` (default off) that gates ENS-record helpers.

### Crate-Graph Invariants

`cow-sdk-cow-shed` depends on `cow-sdk-core`, `cow-sdk-contracts`,
`cow-sdk-signing`, `cow-sdk-app-data`, and `cow-sdk-pure-helpers`. It MUST
NOT depend on `cow-sdk-trading`, `cow-sdk-orderbook`, `cow-sdk-subgraph`,
`cow-sdk-browser-wallet`, `alloy-provider`, `alloy-signer-local`, `reqwest`,
or `tokio` runtime features. The negative-edge invariant
`cow-sdk-cow-shed ⇏ cow-sdk-trading` is asserted via `cargo metadata` and the
`parity-maintainer check-deps` validator in CI.

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
  `version-call-results.json` artifact carries per-chain rows with
  `decoded_version == "1.0.1"` and `expected_sdk_version == "CowShedVersion::V1_0_1"`.
- Runtime and support: every internal builder forwards the caller-selected
  version. The regression test asserts distinct version variants produce
  distinct proxy addresses.
- Crate graph: `cargo metadata` continues to prove
  `cow-sdk-cow-shed ⇏ cow-sdk-trading`,
  `cow-sdk-cow-shed ⇏ cow-sdk-orderbook`,
  `cow-sdk-cow-shed ⇏ cow-sdk-subgraph`,
  `cow-sdk-cow-shed ⇏ alloy-provider`.
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
