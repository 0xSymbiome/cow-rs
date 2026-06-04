# Deployment Registry Audit

Status: Current
Last reviewed: 2026-06-01
Re-review by: 2026-08-02
Owning surface: `cow-sdk-contracts` deployment registry and provenance manifest
Refresh trigger: Changes to `crates/contracts/registry.toml`, `crates/contracts/deployment-provenance.yaml`, the compile-time validator in `build.rs`, the `registry-confirm` presence probe, deployed addresses, or supported chains
Related docs:
- [ADR 0012](../adr/0012-alloy-sol-bindings-and-registry-authority.md)
- [ADR 0032](../adr/0032-deployment-authority-machine-readable-provenance.md)
- [Deployments](../deployments.md)
- [Architecture](../architecture.md)
- [Parity Matrix](../parity.md)

## Scope

This audit covers:

- the embedded `registry.toml` address manifest and typed `Registry` lookup surface
- the `deployment-provenance.yaml` source-provenance manifest
- compile-time agreement checks between registry rows and provenance rows
- live bytecode confirmation through `validation-smoke registry-confirm`

It does not cover binding generation, partner API routing, arbitrary consumer RPC configuration, or future contract upgrades after the recorded confirmation time.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Registry completeness | Every `(ContractId, SupportedChainId, CowEnv)` registry row has one provenance row | Conforms |
| Chain provenance | Every `SupportedChainId` variant has a source-cited services, TypeScript SDK, deployment-provenance, and wrapped-native-token row | Conforms |
| Runtime lookup matrix | Every supported `(ContractId, SupportedChainId, CowEnv)` tuple is either a typed deployed address or an explicit unsupported lookup without silent fallback | Conforms |
| Source authority | Each provenance row records primary or secondary upstream authority at a pinned source commit | Conforms |
| Compile-time validation | `build.rs` rejects missing, duplicate, extra, malformed, or address-mismatched provenance rows | Conforms |
| Live presence | A live `eth_getCode` probe confirms on-chain bytecode presence for every probed row | Conforms |
| Release probe | `registry-confirm --mode release` confirms presence read-only, failing closed on a missing production-chain RPC or an absent deployment | Conforms |

## Current Contract

### Registry And Provenance

`crates/contracts/registry.toml` remains the runtime address source of truth. `crates/contracts/deployment-provenance.yaml` is the auditable evidence layer keyed by the same `(contract_id, chain_id, env)` tuple. The provenance file carries one row for every registry row: 177 rows spanning 14 contract families across the 12 deployment chains (the 11 runtime-supported chains plus Lens, which is deployment-only) and the `prod`, `staging`, and `environment_agnostic` environments.

The provenance row records:

- `address`
- `source_repo`
- `source_commit`
- `source_path`
- `source_symbol`
- `verification` (`status` + `source`)

The runtime lookup regression enumerates every shipped contract id across each
supported chain and environment. Tuples present in the embedded manifest must
resolve to the same non-zero address as their manifest row; unsupported tuples
must stay typed misses rather than falling back to another chain, environment,
or contract family.

### Verification Status

Every row carries a `verification.status` recording how strongly the committed
address is backed. The distribution across the 177 rows is:

| `verification.status` | Rows | Meaning |
| --- | ---: | --- |
| `code_hash_verified` | 153 | The deployed bytecode is code-hash-verified at the pinned upstream manifest (upstream deployments are explorer/Sourcify-verified); no locally committed digest |
| `external_verified` | 8 | A third-party verifier or explorer attested the bytecode |
| `readme_table_unverified` | 8 | Sourced from an upstream README table; not independently probed |
| `canonical_unverified` | 8 | Canonical source evidence with no committed hash or external attestation |

## Per-chain Provenance

The table below is the canonical supported-chain provenance view for the
release-facing registry. It intentionally lives in this deployment-registry
audit so chain support, deployed contract provenance, services-generated
metadata, TypeScript SDK support, and wrapped-native-token evidence have one
reviewed authority. It lists the 11 runtime-supported `SupportedChainId`
variants; Lens (chain 232) is deployment-only and so appears in the registry and
provenance manifest but not in this `SupportedChainId` view. Provenance rows are
authoritatively keyed by `(contract_id, chain_id, env)`; because the manifest is
machine-generated, the line anchors below are indicative of the row's last review
rather than pinned offsets.

| Chain | `SupportedChainId` variant | Numeric chain id | Deployment provenance | Services metadata | TypeScript SDK source | Wrapped native token | Last reviewed |
| --- | --- | ---: | --- | --- | --- | --- | --- |
| Ethereum Mainnet | `Mainnet` | 1 | `crates/contracts/deployment-provenance.yaml:5` | `services/contracts/generated/contracts-generated/gpv2settlement/src/lib.rs:5075` | `packages/config/src/chains/const/chainIds.ts:21`; `README.md:19` | `0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2` (`crates/core/src/config/chains.rs:11`) | 2026-05-04 |
| BNB Smart Chain | `Bnb` | 56 | `crates/contracts/deployment-provenance.yaml:40` | `services/contracts/generated/contracts-generated/gpv2settlement/src/lib.rs:5083` | `packages/config/src/chains/const/chainIds.ts:27`; `README.md:20` | `0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c` (`crates/core/src/config/chains.rs:25`) | 2026-05-04 |
| Gnosis Chain | `GnosisChain` | 100 | `crates/contracts/deployment-provenance.yaml:75` | `services/contracts/generated/contracts-generated/gpv2settlement/src/lib.rs:5087` | `packages/config/src/chains/const/chainIds.ts:22`; `README.md:21` | `0xe91D153E0b41518A2Ce8Dd3D7944Fa863463a97d` (`crates/core/src/config/chains.rs:13`) | 2026-05-04 |
| Polygon PoS | `Polygon` | 137 | `crates/contracts/deployment-provenance.yaml:110` | `services/contracts/generated/contracts-generated/gpv2settlement/src/lib.rs:5091` | `packages/config/src/chains/const/chainIds.ts:26`; `README.md:22` | `0x0d500b1d8e8ef31e21c99d1db9a6444d3adf1270` (`crates/core/src/config/chains.rs:21`) | 2026-05-04 |
| Base | `Base` | 8453 | `crates/contracts/deployment-provenance.yaml:145` | `services/contracts/generated/contracts-generated/gpv2settlement/src/lib.rs:5095` | `packages/config/src/chains/const/chainIds.ts:24`; `README.md:23` | `0x4200000000000000000000000000000000000006` (`crates/core/src/config/chains.rs:17`) | 2026-05-04 |
| Plasma | `Plasma` | 9745 | `crates/contracts/deployment-provenance.yaml:180` | `services/contracts/generated/contracts-generated/gpv2settlement/src/lib.rs:5099` | `packages/config/src/chains/const/chainIds.ts:28`; `README.md:24` | `0x6100e367285b01f48d07953803a2d8dca5d19873` (`crates/core/src/config/chains.rs:27`) | 2026-05-04 |
| Arbitrum One | `ArbitrumOne` | 42161 | `crates/contracts/deployment-provenance.yaml:214` | `services/contracts/generated/contracts-generated/gpv2settlement/src/lib.rs:5103` | `packages/config/src/chains/const/chainIds.ts:23`; `README.md:25` | `0x82aF49447D8a07e3bd95BD0d56f35241523fBab1` (`crates/core/src/config/chains.rs:15`) | 2026-05-04 |
| Avalanche C-Chain | `Avalanche` | 43114 | `crates/contracts/deployment-provenance.yaml:249` | `services/contracts/generated/contracts-generated/gpv2settlement/src/lib.rs:5107` | `packages/config/src/chains/const/chainIds.ts:25`; `README.md:26` | `0xb31f66aa3c1e785363f0875a1b74e27b85fd66c7` (`crates/core/src/config/chains.rs:23`) | 2026-05-04 |
| Ink | `Ink` | 57073 | `crates/contracts/deployment-provenance.yaml:284` | `services/contracts/generated/contracts-generated/gpv2settlement/src/lib.rs:5111` | `packages/config/src/chains/const/chainIds.ts:30`; `README.md:27` | `0x4200000000000000000000000000000000000006` (`crates/core/src/config/chains.rs:17`) | 2026-05-04 |
| Linea | `Linea` | 59144 | `crates/contracts/deployment-provenance.yaml:318` | `services/contracts/generated/contracts-generated/gpv2settlement/src/lib.rs:5115` | `packages/config/src/chains/const/chainIds.ts:29`; `README.md:28` | `0xe5d7c2a44ffddf6b295a15c148167daaaf5cf34f` (`crates/core/src/config/chains.rs:29`) | 2026-05-04 |
| Sepolia (Ethereum testnet) | `Sepolia` | 11155111 | `crates/contracts/deployment-provenance.yaml:352` | `services/contracts/generated/contracts-generated/gpv2settlement/src/lib.rs:5119` | `packages/config/src/chains/const/chainIds.ts:31`; `README.md:29` | `0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14` (`crates/core/src/config/chains.rs:19`) | 2026-05-04 |

### Live Presence

Deployment trust does not rest on committed per-row code hashes. The shipped
evidence is the pinned `source_commit` (the upstream machine-readable manifest the
address was taken from) plus the deterministic CREATE2 address; on top of that a
read-only live probe confirms the claimed deployment actually exists on-chain.

`registry-confirm --mode release` reads every selected row from the manifest,
guards the RPC with `eth_chainId`, and asserts `eth_getCode` returns non-empty
bytecode at the recorded address. It is non-mutating and fails closed on a missing
production-chain RPC or an absent deployment. The last full run confirmed presence
across all 12 deployment chains (the 11 runtime-supported chains plus Lens) with
zero failures.

Per ADR 0032, committed code-hash confirmation is reserved for upgradeable
deployments. The current contract set is non-upgradeable CREATE2 singletons whose
bytecode at a fixed address cannot change, so a live presence probe is the
appropriate check and no per-row code hash is committed.

## Evidence

Primary implementation points:

- `crates/contracts/src/deployments/registry.rs`
- `crates/core/src/config/chains.rs`
- `crates/contracts/registry.toml`
- `crates/contracts/deployment-provenance.yaml`
- `crates/contracts/build.rs`
- `scripts/validation-smoke/src/registry_confirm.rs`

Primary regression coverage:

- `crates/contracts/tests/registry.rs`
- `crates/contracts/tests/registry.rs::registry_address_lookup_matrix_is_exhaustive`
- `crates/contracts/tests/schema_v2_rejection.rs`
- `crates/contracts/tests/deployment_provenance_contract.rs`
- `tests/supported_chains_doc_table.rs::supported_networks_doc_table_matches_enum`
- `scripts/validation-smoke/tests/registry_confirm.rs`

Validation surface:

```text
cargo build -p cow-sdk-contracts
cargo test -p cow-sdk-contracts --test deployment_provenance_contract
cargo test -p cow-rs-workspace-tests --test supported_chains_doc_table
cargo run --manifest-path scripts/validation-smoke/Cargo.toml -- registry-confirm --mode release --chain-ids 1,100,42161,8453,11155111,137,43114,56,9745,59144,57073,232
bash scripts/check-release-docs-agree.sh
```
