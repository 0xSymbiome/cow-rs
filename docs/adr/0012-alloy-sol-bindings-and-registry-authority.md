# ADR 0012: Canonical `alloy::sol!` Bindings And A Single Registry Authority

- Status: Accepted
- Date: 2026-04-21
- Last reviewed: 2026-06-08
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: contracts, bindings, abi, registry, deployments
- Related: [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md), [ADR 0008](0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md), [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md), [ADR 0054](0054-onchain-order-event-decoding-is-fail-closed.md)

## Decision

Every ABI binding in `cow-sdk-contracts` is authored as an inline `alloy::sol!`
interface and proven byte-for-byte against TypeScript-SDK-derived call-data and
EIP-712 digest fixtures under `parity/fixtures/`; the upstream Solidity each
binding mirrors is pinned by commit in `parity/source-lock.yaml`. Every
deployed-address lookup in the workspace resolves through a single typed
`Registry` keyed on the `(ContractId, SupportedChainId, CowEnv)` tuple.
Hand-rolled encoders and hard-coded chain-scoped address constants are not
allowed in shipped crates. The registry manifest is validated at compile time
through `build.rs` before the crate builds, and again at runtime through
`Registry::from_toml_str` for consumers that load their own manifest.

## Why

A protocol SDK that hand-writes ABI encoders alongside the upstream
Solidity is two copies of the same contract that drift every time the
upstream surface gains a field, changes a flag layout, or renames a
parameter. A protocol SDK that hard-codes deployed addresses in per-crate
constants is three or four copies of the deployment table that drift every
time a new chain lands or an environment boundary changes. Funnelling the
binding surface through one canonical generator and the address surface
through one canonical lookup keeps the workspace honest, makes every address
auditable from a single file, and pushes the discovery of a malformed
deployment from first runtime call to the compile-time gate. The inline
`alloy::sol!` interface is the single hand-authored description of each
contract surface — the structural analog of the upstream TypeScript SDK's
hand-authored ABI arrays — and the call-data parity fixtures prove that
description produces upstream-identical bytes.

## Must Remain True

- Public surface: every ABI binding the SDK emits call-data against is the
  output of an `alloy::sol!` invocation inside `cow-sdk-contracts` or an
  equivalent capability crate. Each binding's encoded call-data and hashed
  payloads are proven byte-for-byte against TypeScript-SDK-derived fixtures
  under `parity/fixtures/`, and the upstream Solidity it mirrors is pinned by
  commit in `parity/source-lock.yaml` and cited in the binding's rustdoc.
  `Registry::address(contract_id, chain_id, env)` is the sole production path
  for resolving a deployed contract address, and `Registry::with_override` is
  the sole production path for installing a local-dev or fork-specific
  deployment on top of the embedded manifest. The canonical binding families
  covered by this rule are `GPv2Settlement`, `GPv2VaultRelayer`,
  `CoWSwapEthFlow`, the `CoWSwapOnchainOrders` event surface, the EIP-1967
  proxy slot surface, `IERC20` / `IERC20Permit`, and `IWrappedNativeToken`.
- Runtime and support: native Alloy provider and local-signer dependencies are
  confined by the policy-maintainer allow-list checks rather than by a
  hand-maintained crate enumeration in this ADR. The `alloy::sol!` machinery
  (`alloy-sol-types`, `alloy-sol-macro`, `alloy-primitives`) is wasm-safe and
  carries no tokio-bound network client. Consumers select their own chain-RPC
  runtime through the `Provider` seam in `cow-sdk-core`.
- Validation and review: parity scope is byte-identity on implemented
  surfaces. Every binding has a regression test that asserts the generated
  call-data matches a TypeScript-SDK-derived fixture bit for bit; any new
  `#[sol]` interface follows the same pattern before it lands. `build.rs`
  rejects unsupported schema versions, unsupported chain ids, malformed hex
  addresses, and duplicate `(contract, chain, env)` keys. Runtime consumers
  that load their own manifest through `Registry::from_toml_str` see the same
  failure modes as typed `RegistryError` variants.
- Cost: the workspace pins `alloy-sol-macro` and `alloy-sol-types` at
  matching versions and carries `crates/contracts/registry.toml` as a
  committed source. The `build.rs` validator adds a small compile-time cost in
  exchange for catching a malformed manifest before the binary boots.

## Alternatives Rejected

- Keep the hand-rolled Rust encoders alongside the upstream Solidity:
  tested, but every upstream change becomes a two-copy migration and the
  drift detection is a hand-written fixture diff, not a macro-enforced
  signature match.
- Use a mixed binding idiom (macro for new surfaces, hand-rolled for
  legacy): cheaper in the short term, but preserves the drift class the
  macro was adopted to eliminate and doubles the surface reviewers must
  audit.
- Commit a full byte-identical Solidity mirror of each upstream contract and
  gate it with a bespoke SHA-256 verifier: this attests files the compiler
  never reads — the inline `alloy::sol!` interface is the binding, and it is
  never mechanically checked against the mirror — so the apparatus adds a
  large reviewer-facing surface without proving binding correctness. The
  call-data parity fixtures already prove the binding output directly, and a
  commit-level pin in `parity/source-lock.yaml` records the upstream the
  binding tracks. This matches the upstream TypeScript SDK posture, which
  vendors no Solidity source.
- Keep per-crate deployment-address constants in `cow-sdk-core::config`:
  familiar, but every new chain or environment becomes a three-way edit
  across the constant table, the per-crate accessor, and the deployment
  fixture. The typed registry collapses those three touches into one
  TOML row validated at compile time.
- Use a JSON or YAML manifest for deployments: parseable, but less
  human-editable than TOML and forces a separate serde adapter. TOML
  matches the rest of the workspace configuration vocabulary.

## Links

- [Architecture](../architecture.md)
- [Deployments](../deployments.md)
- [Parity Matrix](../parity.md)
- [Parity Scope](../parity.md)
- [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md)
- [ADR 0008](0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md)

**Proven by:**

- [Contract Bindings Parity Audit](../audit/contract-bindings-parity-audit.md)
- [Deployment Registry Audit](../audit/deployment-registry-audit.md)

## Amendment 2026-06-08: deployment registry collapsed to a const table

The "registry authority" half of this decision is updated: the committed
`crates/contracts/registry.toml` manifest and its compile-time / runtime
validators are retired in favour of a const table of CREATE2 singleton
addresses, per the 2026-06-08 amendment to
[ADR 0032](0032-deployment-authority-machine-readable-provenance.md). The
inline `alloy::sol!` binding decision and the parity-fixture posture recorded
above are unchanged; only the address-resolution backing store moved from a TOML
manifest to committed constants, and `RegistryError` (the runtime TOML-parser
diagnostic) is removed with the parser.
