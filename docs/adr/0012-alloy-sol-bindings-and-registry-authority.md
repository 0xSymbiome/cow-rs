# ADR 0012: Canonical `alloy::sol!` Bindings And A Single Registry Authority

- Status: Accepted (amended)
- Date: 2026-04-21
- Last reviewed: 2026-05-28
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: contracts, bindings, abi, registry, deployments
- Related: [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md), [ADR 0008](0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md), [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md), [ADR 0054](0054-onchain-order-event-decoding-is-fail-closed.md)

## Decision

Every ABI binding in `cow-sdk-contracts` is generated through `alloy::sol!`
from byte-identical Solidity mirrors committed under
`crates/contracts/abi/` and gated by `cargo parity-verify-sol-provenance`
against SHA-256 rows in `parity/source-lock.yaml`, and every deployed-address
lookup in the
workspace resolves through a single typed `Registry` keyed on the
`(ContractId, SupportedChainId, CowEnv)` tuple. Hand-rolled encoders and
hard-coded chain-scoped address constants are not allowed in shipped
crates. The registry manifest is validated at compile time through
`build.rs` before the crate builds, and again at runtime through
`Registry::from_toml_str` for consumers that load their own manifest.

## Why

A protocol SDK that hand-writes ABI encoders alongside the upstream
Solidity is two copies of the same contract that drift every time the
upstream surface gains a field, changes a flag layout, or renames a
parameter. A protocol SDK that hard-codes deployed addresses in per-crate
constants is three or four copies of the deployment table that drift every
time a new chain lands or an environment boundary changes. Funnelling both
surfaces through one canonical generator and one canonical lookup keeps the
workspace honest, makes every address auditable from a single file, and
pushes the discovery of a malformed deployment from first runtime call to
the compile-time gate.

## Must Remain True

- Public surface: every ABI binding the SDK emits call-data against is the
  output of an `alloy::sol!` invocation inside `cow-sdk-contracts` or an
  equivalent capability crate. The byte-identical Solidity mirror used to author the
  binding is committed under `crates/contracts/abi/<family>/` so the
  provenance is reviewable at `HEAD`. `Registry::address(contract_id,
  chain_id, env)` is the sole production path for resolving a deployed
  contract address, and `Registry::with_override` is the sole production
  path for installing a local-dev or fork-specific deployment on top of
  the embedded manifest. The canonical binding families covered by this
  rule are `GPv2Settlement`, `GPv2VaultRelayer`, `CoWSwapEthFlow`, the
  EIP-1967 proxy slot surface, and `IERC20` / `IERC20Permit`.
- Solidity provenance discipline: every `.sol` file under
  `crates/contracts/abi/` is a byte-identical mirror of a single
  upstream source pinned in `parity/source-lock.yaml`, and
  `cargo parity-verify-sol-provenance` enforces the gate before the
  workspace builds. The local path, the upstream path under the
  repository root, and the SHA-256 of the upstream bytes at the pinned
  commit live as a `vendored:` row under the matching repository, and
  the verifier rejects any drift between the on-disk SHA and the
  manifest SHA. The verifier additionally rejects any drift between
  the manifest SHA and the live upstream bytes either via
  `--upstream-root <path>` (local `git show <commit>:<path>`) or via
  `--upstream-github` (fetch each `vendored:` row from
  `https://raw.githubusercontent.com/<owner>/<repo>/<commit>/<upstream-path>`
  and compare); the CI quality-gate runs the GitHub-canonical check on
  every push so the manifest cannot silently drift from upstream
  GitHub content. All thirty-seven shipped files follow this posture:
  there is no documentation-only or excerpt-style `.sol` file in the
  workspace, so a reviewer's audit is `sha256sum` on every file against
  the manifest row, or a single `curl` against the same GitHub URL the
  verifier hits. Every `.sol` is LF-normalised on every host through
  `.gitattributes` so the SHA gate stays byte-stable across Windows,
  macOS, and Linux checkouts.
- Runtime and support: native Alloy provider and local-signer dependencies are
  confined by the policy-maintainer allow-list checks rather than by a
  hand-maintained crate enumeration in this ADR. The `alloy::sol!` machinery
  (`alloy-sol-types`, `alloy-sol-macro`, `alloy-primitives`) is wasm-safe and
  carries no tokio-bound network client. Consumers select their own chain-RPC
  runtime through the `Provider` seam in `cow-sdk-core`.
- Validation and review: parity scope is byte-identity on implemented
  surfaces. Every migrated binding has a regression test that asserts the
  generated call-data matches a TypeScript-SDK-derived fixture bit for
  bit; any new `#[sol]` interface follows the same pattern before it
  lands. `build.rs` rejects unsupported schema versions, unsupported chain
  ids, malformed hex addresses, and duplicate `(contract, chain, env)`
  keys. Runtime consumers that load their own manifest through
  `Registry::from_toml_str` see the same failure modes as typed
  `RegistryError` variants.
- Cost: the workspace pins `alloy-sol-macro` and `alloy-sol-types` at
  matching versions and carries `crates/contracts/registry.toml` plus
  `crates/contracts/abi/**/*.sol` as committed sources. The `build.rs`
  validator adds a small compile-time cost in exchange for catching a
  malformed manifest before the binary boots.

## Alternatives Rejected

- Keep the hand-rolled Rust encoders alongside the upstream Solidity:
  tested, but every upstream change becomes a two-copy migration and the
  drift detection is a hand-written fixture diff, not a macro-enforced
  signature match.
- Use a mixed binding idiom (macro for new surfaces, hand-rolled for
  legacy): cheaper in the short term, but preserves the drift class the
  macro was adopted to eliminate and doubles the surface reviewers must
  audit.
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
- [Parity Matrix](../parity-matrix.md)
- [Parity Scope](../parity-scope.md)
- [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md)
- [ADR 0008](0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md)

**Proven by:**

- [Contract Bindings Parity Audit](../audit/contract-bindings-parity-audit.md)
- [Deployment Registry Audit](../audit/deployment-registry-audit.md)

## Amendment 2026-05-28: on-chain order event bindings and the wrapped-native token

This decision now also governs three additional `alloy::sol!` bindings, each
vendored byte-identically under the `ethflowcontract` repository in
`parity/source-lock.yaml` and gated by the same `parity-verify-sol-provenance`
contract:

- the `CoWSwapOnchainOrders` event surface (`OrderPlacement` /
  `OrderInvalidation`), whose fail-closed, provider-free log decoder is governed
  by [ADR 0054](0054-onchain-order-event-decoding-is-fail-closed.md);
- the `IWrappedNativeToken` (WETH9-family) `deposit` / `withdraw` surface, with
  wrap / unwrap helpers that emit the canonical settlement interaction.

The canonical binding families covered by this rule are therefore
`GPv2Settlement`, `GPv2VaultRelayer`, `CoWSwapEthFlow`, `CoWSwapOnchainOrders`,
the EIP-1967 proxy slot surface, `IERC20` / `IERC20Permit`, and
`IWrappedNativeToken`. The committed Solidity mirror corpus moves from
thirty-seven to forty files; every added mirror is a byte-identical pin
verified on every push against GitHub-canonical content.
