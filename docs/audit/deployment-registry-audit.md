---
type: Audit
id: deployment-registry
title: "Deployment Registry Audit"
description: "Every deployed address resolves through the typed Registry from a commit-pinned upstream source, confirmed live by eth_getCode, with no silent fallback on an unsupported chain."
status: Current
owning_surface: "cow-sdk-contracts Registry deployment authority"
related: [ADR-0012, ADR-0032]
timestamp: 2026-06-20
---

# Deployment Registry Audit

## Scope

Reviews the typed `Registry` deployment authority: the const address table, the
per-source commit pins that anchor each address, the live `eth_getCode`
confirmation, and the Lens chain-taxonomy evidence. It does not cover the
contract bindings themselves (the Contract Bindings Parity Audit).

## Findings

- Each registered address derives from an upstream source repository pinned by
  commit in `parity/source-lock.yaml` — one trust anchor per repository.
- Every `SupportedChainId` variant resolves through the typed
  `(ContractId, chain, env)` lookup to a deployed address or an explicit miss,
  with no silent fallback.
- The live `registry-confirm` probe reads `eth_getCode` for every
  production/staging row across the runtime-supported chains; it is read-only and
  fails closed on a missing RPC or an absent deployment.
- The Lens chain exists in the deployment taxonomy for composable and COW Shed
  rows but is absent from the runtime `SupportedChainId` enum, so the registry
  returns `None` and orderbook clients cannot select it.
- Trust rests on the pinned source commit plus the deterministic CREATE2 address;
  the current set is non-upgradeable singletons whose bytecode at a fixed address
  cannot change.

### Per-chain provenance

Each chain's deployment, services-metadata, and TypeScript-SDK provenance is the
correspondingly named row in `parity/source-lock.yaml`; the wrapped-native token
address is pinned in `crates/core/src/config/chains.rs`.

| Chain | `SupportedChainId` | Chain id | Wrapped native token |
| --- | --- | ---: | --- |
| Ethereum Mainnet | `Mainnet` | 1 | `0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2` |
| BNB Smart Chain | `Bnb` | 56 | `0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c` |
| Gnosis Chain | `GnosisChain` | 100 | `0xe91D153E0b41518A2Ce8Dd3D7944Fa863463a97d` |
| Polygon PoS | `Polygon` | 137 | `0x0d500b1d8e8ef31e21c99d1db9a6444d3adf1270` |
| Base | `Base` | 8453 | `0x4200000000000000000000000000000000000006` |
| Plasma | `Plasma` | 9745 | `0x6100e367285b01f48d07953803a2d8dca5d19873` |
| Arbitrum One | `ArbitrumOne` | 42161 | `0x82aF49447D8a07e3bd95BD0d56f35241523fBab1` |
| Avalanche C-Chain | `Avalanche` | 43114 | `0xb31f66aa3c1e785363f0875a1b74e27b85fd66c7` |
| Ink | `Ink` | 57073 | `0x4200000000000000000000000000000000000006` |
| Linea | `Linea` | 59144 | `0xe5d7c2a44ffddf6b295a15c148167daaaf5cf34f` |
| Sepolia | `Sepolia` | 11155111 | `0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14` |

## Evidence

- Decision: [ADR 0012](../adr/0012-alloy-sol-bindings-and-registry-authority.md), [ADR 0032](../adr/0032-deployment-authority-machine-readable-provenance.md).
- Rule: [Evidence-Backed Public Claims](../principles/evidence-backed-public-claims.md).
- Invariants: the `PROP-CON` family ([contracts](../properties/contracts.md)).
- Governing gate: `deployment_addresses_resolve_to_canonical_singletons` + `xtask/src/parity/registry_confirm.rs`.
- Code: `crates/contracts/src/deployments.rs`, `crates/core/src/config/chains.rs`, `parity/source-lock.yaml`.
