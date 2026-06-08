# Deployments And The Registry

This page explains how `cow-rs` resolves deployed contract addresses, how the
embedded deployment manifest is validated, and how deployment coverage evidence
is kept separate from addressable registry rows.

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

The backing store is a deterministic
`BTreeMap<(ContractId, DeploymentChainId, DeploymentEnv), Address>`.
`Registry::len()`, `Registry::is_empty()`, `Registry::entries()`, and
`Registry::entry_details()` expose the complete manifest for audit diffs and
validation sweeps.

Shipped leaf crates resolve through the registry rather than reading
chain-scoped address constants. Environment-agnostic capability contracts can be
looked up with `DeploymentEnv::EnvironmentAgnostic`, and concrete prod or
staging lookups fall back to that shared row only for contracts declared
environment-agnostic.

## Deployment Taxonomy

`ContractId` names each registered contract family:

- GPv2 contracts: `Settlement`, `VaultRelayer`, `EthFlow`
- composable-order contracts: `ComposableCow`, `ExtensibleFallbackHandler`,
  `CurrentBlockTimestampFactory`, `TwapHandler`, `GoodAfterTimeHandler`,
  `StopLossHandler`, `TradeAboveThresholdHandler`,
  `PerpetualStableSwapHandler`
- COW Shed contracts: `CowShedImplementation`, `CowShedFactory`,
  `CowShedForComposableCow`

`DeploymentChainId` is the addressable deployment-chain taxonomy. It currently
contains 12 variants: Mainnet, Bnb, GnosisChain, Polygon, Base, Plasma,
ArbitrumOne, Avalanche, Ink, Linea, Sepolia, and Lens. This taxonomy is wider
than runtime orderbook support, so Lens can be tracked as deployment evidence
without adding `SupportedChainId::Lens`.

`DeploymentEnv` has three variants: `Prod`, `Staging`, and
`EnvironmentAgnostic`. GPv2 rows must use prod or staging. Capability rows must
use the environment-agnostic scope.

`DeploymentVerificationStatus` lives only on registry rows:

- `CodeHashVerified`
- `ExternalVerified`
- `ReadmeTableUnverified`
- `CanonicalUnverified`

`DeploymentCoverageStatus` lives only in the coverage manifest for
non-addressable evidence:

- `NotDeployed`
- `NotSupported`
- `OutOfScope`

Coverage records are not registry rows and never resolve through
`Registry::address`. For example, Optimism is represented as unsupported
coverage because it is outside the Rust target set; Ink entries with empty code
are represented as not-deployed coverage.

## Embedded Manifests

The canonical registry is committed at `crates/contracts/registry.toml` and
embedded at compile time. Schema v2 rows carry a typed key, a validated
20-byte address, and a verification status:

```toml
schema_version = 2

[[entries]]
contract_id = "Settlement"
chain_id = 1
env = "prod"
address = "0x9008D19f58AAbD9eD0D60971565AA8510560ab41"
[entries.verification]
status = "code_hash_verified"
source = "pinned deployment provenance"
```

The registry currently contains 177 addressable rows. The companion
`crates/contracts/deployment-provenance.yaml` file mirrors every registry row
with source provenance, and `crates/contracts/deployment-coverage.yaml` records
not-deployed, not-supported, and out-of-scope evidence that must not become
addressable rows.

The manifests are validated twice:

- At compile time through `build.rs`. Malformed rows, duplicate keys,
  unsupported registry chains, invalid environment scopes, wrong schema
  versions, provenance drift, and COW Shed proxy creation-code hash drift fail
  the build with precise diagnostics.
- At runtime through `Registry::from_toml_str` and
  `DeploymentCoverage::from_yaml_str`, so custom manifests see typed parser
  errors rather than unchecked strings.

## Loading A Custom Manifest

Consumers that want to drive the registry from their own TOML pipe the raw
string into `Registry::from_toml_str`:

```rust
use cow_sdk::contracts::{ContractId, Registry};
use cow_sdk::core::{CowEnv, SupportedChainId};

let raw = r#"
schema_version = 2

[[entries]]
contract_id = "EthFlow"
chain_id = 100
env = "prod"
address = "0x40A50cf069e992AA4536211B23F286eF88752187"
[entries.verification]
status = "canonical_unverified"
source = "local fork fixture"
"#;

let registry = Registry::from_toml_str(raw)
    .expect("custom registry manifest must parse");
let eth_flow = registry
    .address(ContractId::EthFlow, SupportedChainId::GnosisChain, CowEnv::Prod)
    .expect("eth-flow deployment is present");

assert_ne!(
    eth_flow.to_string(),
    "0x0000000000000000000000000000000000000000"
);
```

The parser enforces the same registry validation rules as `build.rs`.

## Layering A Single Override

When a single address differs from the canonical manifest, compose an override
on top of `Registry::default()`:

```rust
use cow_sdk::contracts::{ContractId, Registry};
use cow_sdk::core::{Address, CowEnv, SupportedChainId};

let local = "0x1111111111111111111111111111111111111111"
    .parse::<Address>()
    .expect("fixture address must parse");

let registry = Registry::default().with_override(
    ContractId::Settlement,
    SupportedChainId::Mainnet,
    CowEnv::Prod,
    local,
);

assert_eq!(
    registry.address(ContractId::Settlement, SupportedChainId::Mainnet, CowEnv::Prod),
    Some(local)
);
```

`Registry::with_override` replaces one entry and leaves every other entry at
the canonical value.

## Dependency Posture

The registry is part of `cow-sdk-contracts` and does not pull any chain-RPC
dependency. Resolving an address is a pure in-memory lookup; the SDK never
dispatches a network call on behalf of `Registry::address`. Chain-RPC
resolution, such as querying whether a proxy implementation has been upgraded,
flows through the provider seam in `cow-sdk-core` and is a separate runtime
contract.

Crates.io owner rotation is not deployment provenance. Publication ownership
for the SDK crate family is tracked separately in
[Publication Handoff](publication-handoff.md), while this page remains limited
to on-chain contract address authority.

## Related Docs

- [Architecture](architecture.md)
- [Parity Matrix](parity.md)
- [Verification Guide](verification.md)
- [ADR 0012](adr/0012-alloy-sol-bindings-and-registry-authority.md)
