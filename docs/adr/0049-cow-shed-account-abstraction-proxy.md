# ADR 0049: COW Shed Account-Abstraction Proxy

- Status: Accepted
- Date: 2026-05-15
- Last reviewed: 2026-06-15
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: cow-shed, account-abstraction, version-forwarding, proxy-derivation
- Related: [ADR 0008](0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md), [ADR 0012](0012-alloy-sol-bindings-and-registry-authority.md), [ADR 0022](0022-ecdsa-signature-v-normalization.md), [ADR 0050](0050-eip1271-signature-blob-encoding.md), [ADR 0051](0051-signing-owned-eip1271-signature-provider-trait.md), [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md)

## Context

COW Shed is a CREATE2-deployed per-user ERC-1967 proxy that fronts interactions
with a designated implementation contract. The SDK binds the **deployed v1.0.x
generation**: across the eleven supported chains the deployed implementation
returns `"1.0.1"` from `VERSION()`, and the SDK signs and derives proxy addresses
against that deployed reality, not against the advanced upstream Solidity source
HEAD — a signature produced against a later source version would fail
verification on every live proxy.

The COW Shed source has layered authorities that must be ranked because they
disagree: (1) the deployed-runtime factory ABI (the canonical 2-arg
`initializeProxy(address,bool)` selector that matches deployed bytecode), (2) the
upstream Solidity source (a divergent 1-arg form, reference only), (3) the
per-chain `networks.json` deployment registry, and (4) the version-keyed pinning
constants. Layers 1 and 4 agree on the v1.0.x generation; the `networks.json`
HEAD rows for Gnosis describe a *different, unsupported* v2.0.0-generation
redeploy, not a chain divergence within the supported generation — so the
deployed pairs are chain-uniform per version and the derived proxy address is
chain-independent.

The upstream TypeScript SDK has a known bug where the `CoWShedHooks` constructor
silently drops the caller-selected version and defaults every instance to the
latest. The Rust SDK must not mirror it.

## Decision

The COW Shed surface ships as the `cow_sdk_contracts::cow_shed` module, gated
behind the off-by-default `cow-shed` feature of `cow-sdk-contracts` and exposed
through the facade-level `cow-shed` feature as `cow_sdk::cow_shed`. It is an
additive capability per [ADR 0008](0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md)
and never on the default `cow-sdk` dependency closure.

**Version forwarding.** `CowShedVersion` has variants `V1_0_0` and `V1_0_1`
(`V1_0_1` is the default). The caller-selected version threads through every
internal builder; no helper constructs a downstream object without forwarding
it, and the regression test `distinct_versions_derive_distinct_proxies` proves
distinct versions derive distinct proxies. The deployed factory/implementation
pairs and domain strings are pinned in
`parity/fixtures/cow_shed/deployments.json`, and the source-lock pins cow-shed at
the v1.0.1 tag commit (`e15a131d`).

**Hook hashing and signature order.** EIP-712 type strings for the hook
structure carry no whitespace between commas in declaration order (`Call(address
target,uint256 value,bytes callData,bool allowFailure,bool isDelegateCall)` and
the `ExecuteHooks(...)` composite). The signing digest is produced by
`<ExecuteHooks as alloy_sol_types::SolStruct>::eip712_signing_hash(&domain)`
through the single entry `execute_hooks_signing_hash(&domain, &calls, nonce,
deadline) -> B256`, with `cow_shed_eip712_domain(chain, version, proxy)` building
the domain (`cow_shed_domain_separator` is a thin `.separator()` wrapper). EOA
signatures use the `r || s || v` byte order, produced and validated by
`cow_sdk_contracts::RecoverableSignature` (non-65-byte input and recovery bytes
outside `{0, 1, 27, 28}` rejected, [ADR 0022](0022-ecdsa-signature-v-normalization.md));
a smart-account (EIP-1271) owner supplies a variable-length blob, and
`encode_execute_hooks_calldata_with_signature` carries either shape to the
factory unchanged. `isDelegateCall = true` is opt-in only through
`Call::delegate_call`, and each call site carries a `// SAFETY:` justification.

**Orchestrator.** `CowShedHooks::new(chain)` → `sign(&signer, &calls, nonce,
deadline)` resolves the owner from the owned `Signer`, derives the proxy via
`proxy_for`, signs the `ExecuteHooks` payload through `sign_typed_data_payload`,
and returns a `SignedCowShedCall { shed_account, factory, factory_calldata }`
that submits directly or becomes an app-data hook via `to_app_data_hook`.
`CowShedVersion::ALL` (current generation first) supports multi-version proxy
discovery.

**Scope (deployed v1.0.x only).** The bindings mirror the deployed v1.0.x surface
exactly — `initialize(address,bool)`, `VERSION()`, `claimWithResolver`, and the
`OnlyTrustedExecutor` / `OnlyAdminOrTrustedExecutorOrSelf` / `DeadlineElapsed` /
`NonceAlreadyUsed` errors — as a documented subset excluding the ENS resolver
reads. The v2-only pre-sign family (`executePreSignedHooks` and friends) and the
Gnosis-only `COWShedForComposableCoW` forwarder are **not bound**: they belong to
the v2 generation and revert on every supported deployment. They return with a
real `V2_x` version family when upstream rolls v2 out beyond Gnosis.

**Crate graph.** The `cow-shed` feature of `cow-sdk-contracts` adds only the
`cow-sdk-app-data` dependency on top of `cow-sdk-core`. It MUST NOT pull
`cow-sdk-trading`, `cow-sdk-orderbook`, `cow-sdk-subgraph`,
`alloy-provider`, `alloy-signer-local`, `reqwest`, or `tokio`.

## Why

Deployed-bytecode-first authority avoids signing failures that appear only at
verification time: a signature produced against a source-HEAD version would fail
on every live v1.0.x proxy. Threading the caller-selected version through every
builder prevents the upstream silent-drop bug, and the regression test makes it
checkable. Binding only the deployed v1.0.x surface keeps every shipped selector
callable — the v2 pre-sign and forwarder families revert on the supported
deployments, so binding them would ship functions that always fail. The negative
edge against `cow-sdk-trading` keeps cow-shed an additive peer leaf per ADR 0008.

## Must Remain True

- Public surface: `CowShedVersion::V1_0_1` is the default;
  `parity/fixtures/cow_shed/deployments.json` pins the per-version
  factory/implementation pairs and the deployed `VERSION()` domain strings.
  Deployed pairs are chain-uniform per version, so the derived proxy address is
  chain-independent.
- Version forwarding: every internal builder forwards the caller-selected
  version; `distinct_versions_derive_distinct_proxies` proves distinct versions
  derive distinct proxies.
- Scope: only the deployed v1.0.x surface is bound; the pre-sign family and the
  `COWShedForComposableCoW` forwarder stay unbound until a v2 generation lands.
  ERC-2098 compact helpers live on `RecoverableSignature` (no module-local
  duplicate).
- Crate graph: `cargo metadata` proves the `cow-shed` feature closure excludes
  `cow-sdk-trading`, `cow-sdk-orderbook`, `cow-sdk-subgraph`, and `alloy-provider`.
- Validation: the COW Shed contract-bindings and app-data-integration audits
  cross-link this ADR and stay `Current` when the surface moves.

## Alternatives Rejected

- Sign against source HEAD: every signature fails verification on live v1.0.x
  proxies.
- Mirror the upstream constructor without version forwarding: reproduces the
  known silent-drop bug.
- Bind the 1-arg `initializeProxy(address)` Solidity selector: the deployed
  bytecode targets the 2-arg form.
- Bind the v2 pre-sign family or the Gnosis forwarder against v1.0.x: ships
  selectors that revert on every supported deployment.
- Statically cache `keccak256(PROXY_CREATION_CODE)` once: the init-code hash is
  per-`(implementation, who)`, so one cache would collapse every user to a single
  proxy address.

## Links

- [Architecture](../architecture.md)
- [Principles](../principles.md)
- [ADR 0008](0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md)
- [ADR 0022](0022-ecdsa-signature-v-normalization.md)
- [ADR 0050](0050-eip1271-signature-blob-encoding.md)
- [ADR 0051](0051-signing-owned-eip1271-signature-provider-trait.md)

**Proven by:**

- [COW Shed Contract Bindings Audit](../audit/cow-shed-contract-bindings-audit.md)
- [COW Shed App Data Integration Audit](../audit/cow-shed-app-data-integration-audit.md)
