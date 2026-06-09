# Deployment Registry Audit

Status: Current
Last reviewed: 2026-06-09
Re-review by: 2026-08-02
Owning surface: `cow-sdk-contracts` deployment registry
Refresh trigger: Changes to the address constants in `crates/contracts/src/deployments.rs`, the upstream commit pins in `parity/source-lock.yaml`, the `registry-confirm` presence probe, or supported chains
Related docs:
- [ADR 0012](../adr/0012-alloy-sol-bindings-and-registry-authority.md)
- [ADR 0032](../adr/0032-deployment-authority-machine-readable-provenance.md)
- [Deployments](../deployments.md)
- [Architecture](../architecture.md)
- [Parity Matrix](../parity.md)

## Scope

This audit covers:

- the const address table and typed `Registry` lookup surface in `crates/contracts/src/deployments.rs`
- the upstream commit pins in `parity/source-lock.yaml` that anchor each deployed address to a source repository
- live bytecode confirmation through `validation-smoke registry-confirm`

It does not cover binding generation, partner API routing, arbitrary consumer RPC configuration, or future contract upgrades after the recorded confirmation time.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Source authority | Each registered address derives from an upstream source repository whose commit is pinned in `parity/source-lock.yaml` | Conforms |
| Chain provenance | Every `SupportedChainId` variant has a source-cited services, TypeScript SDK, source-lock commit-pin, and wrapped-native-token row | Conforms |
| Runtime lookup matrix | Every supported `(ContractId, SupportedChainId, CowEnv)` tuple is either a typed deployed address or an explicit unsupported lookup without silent fallback | Conforms |
| Live presence | A live `eth_getCode` probe confirms on-chain bytecode presence for every probed row | Conforms |
| Release probe | `registry-confirm --mode release` confirms presence read-only, failing closed on a missing production-chain RPC or an absent deployment | Conforms |

## Current Contract

### Registry And Source Authority

`crates/contracts/src/deployments.rs` is the runtime address source of truth. It resolves the settlement, vault-relayer, and eth-flow CREATE2 singletons from committed address constants: settlement and vault-relayer are chain- and environment-invariant, and eth-flow carries a production and a staging deployment, each identical across the runtime-supported chains. Lens is deployment-only for the composable / COW-Shed contract families and carries none of the GPv2 contracts.

The upstream commit each address derives from is not duplicated on every row. It is pinned once per source repository in `parity/source-lock.yaml` (per [ADR 0012](../adr/0012-alloy-sol-bindings-and-registry-authority.md) and [ADR 0032](../adr/0032-deployment-authority-machine-readable-provenance.md)): the `contracts`, `composable-cow`, and `cow-shed` repository rows pin the commits behind the GPv2, composable-order, and COW Shed contract families respectively. That single per-repository pin, the deterministic CREATE2 address, and the read-only presence probe together establish deployment trust.

The runtime lookup regression enumerates every shipped contract id across each
supported chain and environment. Tuples present in the embedded manifest must
resolve to the same non-zero address as their manifest row; unsupported tuples
must stay typed misses rather than falling back to another chain, environment,
or contract family.

## Per-chain Provenance

The table below is the canonical supported-chain provenance view for the
release-facing registry. It intentionally lives in this deployment-registry
audit so chain support, deployed contract provenance, services-generated
metadata, TypeScript SDK support, and wrapped-native-token evidence have one
reviewed authority. It lists the 11 runtime-supported `SupportedChainId`
variants; Lens (chain 232) is deployment-only and so appears in the registry but
not in this `SupportedChainId` view. Registry rows are authoritatively keyed by
`(contract_id, chain_id, env)` in `crates/contracts/src/deployments.rs`. The
deployment-source column points at the per-repository upstream commit pin in
`parity/source-lock.yaml`; the GPv2 settlement, vault relayer, and eth-flow
addresses behind every chain below derive from the pinned `contracts` repository
row.

| Chain | `SupportedChainId` variant | Numeric chain id | Deployment source | Services metadata | TypeScript SDK source | Wrapped native token | Last reviewed |
| --- | --- | ---: | --- | --- | --- | --- | --- |
| Ethereum Mainnet | `Mainnet` | 1 | `parity/source-lock.yaml` `contracts` row | `services/contracts/generated/contracts-generated/gpv2settlement/src/lib.rs:5075` | `packages/config/src/chains/const/chainIds.ts:21`; `README.md:19` | `0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2` (`crates/core/src/config/chains.rs:11`) | 2026-05-04 |
| BNB Smart Chain | `Bnb` | 56 | `parity/source-lock.yaml` `contracts` row | `services/contracts/generated/contracts-generated/gpv2settlement/src/lib.rs:5083` | `packages/config/src/chains/const/chainIds.ts:27`; `README.md:20` | `0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c` (`crates/core/src/config/chains.rs:25`) | 2026-05-04 |
| Gnosis Chain | `GnosisChain` | 100 | `parity/source-lock.yaml` `contracts` row | `services/contracts/generated/contracts-generated/gpv2settlement/src/lib.rs:5087` | `packages/config/src/chains/const/chainIds.ts:22`; `README.md:21` | `0xe91D153E0b41518A2Ce8Dd3D7944Fa863463a97d` (`crates/core/src/config/chains.rs:13`) | 2026-05-04 |
| Polygon PoS | `Polygon` | 137 | `parity/source-lock.yaml` `contracts` row | `services/contracts/generated/contracts-generated/gpv2settlement/src/lib.rs:5091` | `packages/config/src/chains/const/chainIds.ts:26`; `README.md:22` | `0x0d500b1d8e8ef31e21c99d1db9a6444d3adf1270` (`crates/core/src/config/chains.rs:21`) | 2026-05-04 |
| Base | `Base` | 8453 | `parity/source-lock.yaml` `contracts` row | `services/contracts/generated/contracts-generated/gpv2settlement/src/lib.rs:5095` | `packages/config/src/chains/const/chainIds.ts:24`; `README.md:23` | `0x4200000000000000000000000000000000000006` (`crates/core/src/config/chains.rs:17`) | 2026-05-04 |
| Plasma | `Plasma` | 9745 | `parity/source-lock.yaml` `contracts` row | `services/contracts/generated/contracts-generated/gpv2settlement/src/lib.rs:5099` | `packages/config/src/chains/const/chainIds.ts:28`; `README.md:24` | `0x6100e367285b01f48d07953803a2d8dca5d19873` (`crates/core/src/config/chains.rs:27`) | 2026-05-04 |
| Arbitrum One | `ArbitrumOne` | 42161 | `parity/source-lock.yaml` `contracts` row | `services/contracts/generated/contracts-generated/gpv2settlement/src/lib.rs:5103` | `packages/config/src/chains/const/chainIds.ts:23`; `README.md:25` | `0x82aF49447D8a07e3bd95BD0d56f35241523fBab1` (`crates/core/src/config/chains.rs:15`) | 2026-05-04 |
| Avalanche C-Chain | `Avalanche` | 43114 | `parity/source-lock.yaml` `contracts` row | `services/contracts/generated/contracts-generated/gpv2settlement/src/lib.rs:5107` | `packages/config/src/chains/const/chainIds.ts:25`; `README.md:26` | `0xb31f66aa3c1e785363f0875a1b74e27b85fd66c7` (`crates/core/src/config/chains.rs:23`) | 2026-05-04 |
| Ink | `Ink` | 57073 | `parity/source-lock.yaml` `contracts` row | `services/contracts/generated/contracts-generated/gpv2settlement/src/lib.rs:5111` | `packages/config/src/chains/const/chainIds.ts:30`; `README.md:27` | `0x4200000000000000000000000000000000000006` (`crates/core/src/config/chains.rs:17`) | 2026-05-04 |
| Linea | `Linea` | 59144 | `parity/source-lock.yaml` `contracts` row | `services/contracts/generated/contracts-generated/gpv2settlement/src/lib.rs:5115` | `packages/config/src/chains/const/chainIds.ts:29`; `README.md:28` | `0xe5d7c2a44ffddf6b295a15c148167daaaf5cf34f` (`crates/core/src/config/chains.rs:29`) | 2026-05-04 |
| Sepolia (Ethereum testnet) | `Sepolia` | 11155111 | `parity/source-lock.yaml` `contracts` row | `services/contracts/generated/contracts-generated/gpv2settlement/src/lib.rs:5119` | `packages/config/src/chains/const/chainIds.ts:31`; `README.md:29` | `0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14` (`crates/core/src/config/chains.rs:19`) | 2026-05-04 |

### Live Presence

Deployment trust does not rest on committed per-row code hashes. The shipped
evidence is the pinned `source_commit` (the upstream machine-readable manifest the
address was taken from) plus the deterministic CREATE2 address; on top of that a
read-only live probe confirms the claimed deployment actually exists on-chain.

`registry-confirm --mode release` resolves every selected row from the const
`Registry`, guards the RPC with `eth_chainId`, and asserts `eth_getCode` returns non-empty
bytecode at the recorded address. It is non-mutating and fails closed on a missing
production-chain RPC or an absent deployment. The last full run confirmed presence
across the 11 runtime-supported chains with zero failures.

Per ADR 0032, committed code-hash confirmation is reserved for upgradeable
deployments. The current contract set is non-upgradeable CREATE2 singletons whose
bytecode at a fixed address cannot change, so a live presence probe is the
appropriate check and no per-row code hash is committed.

## Evidence

Primary implementation points:

- `crates/contracts/src/deployments.rs`
- `crates/core/src/config/chains.rs`
- `parity/source-lock.yaml`
- `scripts/validation-smoke/src/registry_confirm.rs`

Primary regression coverage:

- `crates/contracts/src/deployments.rs::deployment_addresses_resolve_to_canonical_singletons`
- `scripts/validation-smoke/tests/registry_confirm.rs`
- `tests/supported_chains_doc_table.rs::supported_networks_doc_table_matches_enum`
- `scripts/validation-smoke/tests/registry_confirm.rs`

Validation surface:

```text
cargo test -p cow-rs-workspace-tests --test supported_chains_doc_table
cargo run --manifest-path scripts/validation-smoke/Cargo.toml -- registry-confirm --mode release --chain-ids 1,100,42161,8453,11155111,137,43114,56,9745,59144,57073
bash scripts/check-release-docs-agree.sh
```
