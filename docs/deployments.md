# Deployments And The Registry

This page explains how `cow-rs` resolves deployed contract addresses.

## Single Authority

Every deployed-address lookup in the workspace routes through one typed
registry:

```rust
use cow_sdk::contracts::{ContractId, Registry};
use cow_sdk::core::{CowEnv, SupportedChainId};

let registry = Registry::default();
let settlement = registry
    .address(ContractId::Settlement, SupportedChainId::Mainnet, CowEnv::Prod)
    .expect("settlement is deployed on mainnet");

assert_ne!(
    settlement.to_string(),
    "0x0000000000000000000000000000000000000000"
);
```

`Registry::address` returns the deployed address for a
`(ContractId, chain, env)` triple, or `None` when the contract is not
deployed on that chain. Resolving an address is a pure in-memory lookup; the
SDK never dispatches a network call on behalf of `Registry::address`. Shipped
leaf crates resolve through the registry rather than reading chain-scoped
address constants directly.

## Deployment Taxonomy

`ContractId` names each registered contract:

- `Settlement` — `GPv2Settlement`, the settlement entry point.
- `VaultRelayer` — `GPv2VaultRelayer`, the allowance spender ERC-20 approvals
  should target.
- `EthFlow` — `CoWSwapEthFlow`, the native-asset order wrapper.

`DeploymentChainId` is the deployment-chain taxonomy: the eleven
runtime-supported chains plus Lens, which is deployment-only for the
composable / COW-Shed contract families. `DeploymentEnv` is `Prod` or
`Staging`.

## Addresses Are CREATE2 Singletons

The settlement, vault-relayer, and eth-flow contracts are CREATE2 singletons:
`GPv2Settlement` and `GPv2VaultRelayer` deploy to the same address on every
supported chain, and `CoWSwapEthFlow` carries one production and one staging
deployment, each identical across chains. The registry is therefore a small
committed const table rather than a per-chain manifest:

```rust
use cow_sdk::contracts::{ContractId, Registry};
use cow_sdk::core::{CowEnv, SupportedChainId};

let registry = Registry::default();

// Settlement and vault-relayer are chain- and environment-invariant.
let mainnet =
    registry.address(ContractId::Settlement, SupportedChainId::Mainnet, CowEnv::Prod);
let base = registry.address(ContractId::Settlement, SupportedChainId::Base, CowEnv::Staging);
assert_eq!(mainnet, base);

// Eth-flow resolves a distinct production and staging deployment.
let prod = registry.address(ContractId::EthFlow, SupportedChainId::GnosisChain, CowEnv::Prod);
let staging =
    registry.address(ContractId::EthFlow, SupportedChainId::GnosisChain, CowEnv::Staging);
assert_ne!(prod, staging);
```

The Lens chain carries none of the GPv2 contracts, so it resolves to `None`.

## Provenance And Confirmation

The upstream commit each address derives from is pinned once per source
repository in `parity/source-lock.yaml`; the addresses are deterministic
CREATE2 singletons; and the read-only `registry-confirm` presence probe
(`scripts/validation-smoke`) confirms `eth_getCode` returns non-empty bytecode
at each resolved address on-chain. The probe never mutates a file, and its
release mode fails closed on a missing production-chain RPC. See
[ADR 0032](adr/0032-deployment-authority-machine-readable-provenance.md).

## Dependency Posture

The registry is part of `cow-sdk-contracts` and pulls no chain-RPC dependency.
Chain-RPC resolution — such as querying whether a proxy implementation has been
upgraded — flows through the provider seam in `cow-sdk-core` and is a separate
runtime contract.

## Related Docs

- [Architecture](architecture.md)
- [Parity Matrix](parity.md)
- [Verification Guide](verification.md)
- [Deployment Registry Audit](audit/deployment-registry-audit.md)
- [ADR 0012](adr/0012-alloy-sol-bindings-and-registry-authority.md)
- [ADR 0032](adr/0032-deployment-authority-machine-readable-provenance.md)
