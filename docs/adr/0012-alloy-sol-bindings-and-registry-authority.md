# ADR 0012: Canonical `alloy::sol!` Bindings And A Single Registry Authority

- Status: Accepted
- Date: 2026-04-21
- Last reviewed: 2026-06-15
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: contracts, bindings, abi, registry, deployments
- Related: [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md), ADR 0008, [ADR 0032](0032-deployment-authority-machine-readable-provenance.md), [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md), [ADR 0054](0054-onchain-order-event-decoding-is-fail-closed.md)

## Decision

Every ABI binding in `cow-sdk-contracts` is authored as an inline `alloy::sol!`
interface and proven byte-for-byte against TypeScript-SDK-derived call-data and
EIP-712 digest fixtures under `parity/fixtures/`; the upstream Solidity each
binding mirrors is pinned by commit in `parity/source-lock.yaml`. Every
deployed-address lookup in the workspace resolves through a single typed
`Registry` of committed CREATE2-singleton address constants. Hand-rolled ABI
encoders and hard-coded per-crate address constants are not allowed in shipped
crates.

## Why

A protocol SDK that hand-writes ABI encoders alongside the upstream Solidity is
two copies of the same contract that drift every time the upstream surface gains
a field, changes a flag layout, or renames a parameter. A protocol SDK that
hard-codes deployed addresses in per-crate constants is three or four copies of
the deployment table that drift every time a new chain lands or an environment
boundary changes. Funnelling the binding surface through one canonical generator
and the address surface through one canonical lookup keeps the workspace honest,
makes every address auditable from a single file, and turns a malformed
deployment into a compile error rather than a first-call runtime surprise —
every deployed address is a checked constant. The inline `alloy::sol!` interface
is the single hand-authored description of each contract surface — the structural
analog of the upstream TypeScript SDK's hand-authored ABI arrays — and the
call-data parity fixtures prove that description produces upstream-identical
bytes.

## Must Remain True

- Public surface: every ABI binding the SDK emits call-data against is the output
  of an `alloy::sol!` invocation inside `cow-sdk-contracts` or an equivalent
  capability crate. Each binding's encoded call-data and hashed payloads are
  proven byte-for-byte against TypeScript-SDK-derived fixtures under
  `parity/fixtures/`, and the upstream Solidity it mirrors is pinned by commit in
  `parity/source-lock.yaml` and cited in the binding's rustdoc.
  `Registry::address(contract_id, chain_id, env)` — accepting
  `impl Into<DeploymentChainId>` and `impl Into<DeploymentEnv>` — is the sole
  production path for resolving a deployed address; it returns `Option`, so an
  unsupported `(contract, chain, env)` triple is `None` rather than a panic.
  Local-dev or fork-specific deployments are installed through consumer-level
  typed setters (`TradingBuilder::settlement_contract_override` /
  `eth_flow_contract_override`), not a `Registry` method. The canonical binding
  families are `GPv2Settlement`, `GPv2VaultRelayer`, `CoWSwapEthFlow`, the
  `CoWSwapOnchainOrders` event surface, the EIP-1967 proxy-slot surface,
  `IERC20`, `IWrappedNativeToken`, the COW Shed bindings (`cow_shed`), and the
  `cow-sdk-signing` EIP-1271 verifier interface.
- Runtime and support: native Alloy provider and local-signer dependencies are
  confined by the xtask policy allow-list checks rather than by a hand-maintained
  crate enumeration in this ADR. The `alloy::sol!` machinery (`alloy-sol-types`,
  `alloy-sol-macro`, `alloy-primitives`) is wasm-safe and carries no tokio-bound
  network client. Consumers select their own chain-RPC runtime through the
  `Provider` seam in `cow-sdk-core`.
- Validation and review: parity scope is byte-identity on implemented surfaces.
  Every binding has a regression test that asserts the generated call-data
  matches a TypeScript-SDK-derived fixture bit for bit; any new `sol!` interface
  follows the same pattern before it lands. The source-lock is validated by form
  — a GitHub `.git` remote, a 40-character lowercase-hex commit, a known role,
  and unique non-traversing producer paths — and `cargo xtask parity sync` /
  `cargo xtask parity drift` materialize the pinned checkouts and report
  producer-path drift against the upstream default branches.
- Cost: the workspace pins `alloy-sol-macro` and `alloy-sol-types` at matching
  versions and carries the deployment address table as committed Rust constants.

## Alternatives Rejected

- Keep the hand-rolled Rust encoders alongside the upstream Solidity: tested, but
  every upstream change becomes a two-copy migration and the drift detection is a
  hand-written fixture diff, not a macro-enforced signature match.
- Use a mixed binding idiom (macro for new surfaces, hand-rolled for legacy):
  cheaper in the short term, but preserves the drift class the macro was adopted
  to eliminate and doubles the surface reviewers must audit.
- Commit a full byte-identical Solidity mirror of each upstream contract and gate
  it with a bespoke SHA-256 verifier: this attests files the compiler never reads
  — the inline `alloy::sol!` interface is the binding, and it is never
  mechanically checked against the mirror — so the apparatus adds a large
  reviewer-facing surface without proving binding correctness. The call-data
  parity fixtures already prove the binding output directly, and a commit-level
  pin in `parity/source-lock.yaml` records the upstream the binding tracks. This
  matches the upstream TypeScript SDK posture, which vendors no Solidity source.
- Scatter deployed-address constants per crate in `cow-sdk-core::config`:
  familiar, but every new chain or environment becomes a multi-way edit across
  the constant table, the per-crate accessor, and the deployment fixture. The
  single typed registry collapses those touches into one place.

## Links

- [Architecture](../architecture.md)
- [Deployments](../deployments.md)
- [Parity And Provenance](../parity.md)
- [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md)
- [ADR 0032](0032-deployment-authority-machine-readable-provenance.md)
- [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md)

**Proven by:**

- [Contract Bindings Parity Audit](../audit/contract-bindings-parity-audit.md)
- [Deployment Registry Audit](../audit/deployment-registry-audit.md)
