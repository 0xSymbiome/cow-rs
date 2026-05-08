# Deployments And The Registry

This page explains how `cow-rs` resolves deployed contract addresses,
how the embedded deployment manifest is validated, and how to layer a
local-dev or fork-specific deployment on top of the default map.

## Single Authority

Every deployed-address lookup in the workspace routes through one
typed registry:

```rust
use cow_sdk::contracts::{ContractId, Registry};
use cow_sdk::prelude::{CowEnv, SupportedChainId};

let registry = Registry::default();
let settlement = registry
    .address(
        ContractId::Settlement,
        SupportedChainId::Mainnet,
        CowEnv::Prod,
    )
    .expect("settlement is deployed on mainnet");

assert_ne!(
    settlement.to_string(),
    "0x0000000000000000000000000000000000000000"
);
```

The backing store is a `BTreeMap<(ContractId, SupportedChainId, CowEnv),
Address>`, so every entry is keyed on a typed triple and iteration order
is deterministic. `Registry::len()`, `Registry::is_empty()`, and
`Registry::entries()` expose the full manifest for audit diffs and
validation sweeps.

Shipped leaf crates resolve through the registry rather than reading
free-function constants. Consumers that need a deployed address reach
for `Registry::address(...)` at the call site instead of importing a
per-chain constant.

## The `ContractId` Enum

`ContractId` names each registered contract family. The shipped set
covers the surfaces the SDK emits call-data against:

- `ContractId::Settlement` — `GPv2Settlement`
- `ContractId::VaultRelayer` — `GPv2VaultRelayer`
- `ContractId::EthFlow` — `CoWSwapEthFlow`

New families are added as they become part of the shipped contract
surface. Third-party protocol deployments (Aave, bridging adapters,
composable schedulers) live in their capability crates rather than
expanding this enum.

## The Embedded Manifest

The canonical manifest is committed at
`crates/contracts/registry.toml` and embedded at compile time. Each
entry pairs a typed key with a validated 20-byte address:

```toml
schema_version = 1

[[entries]]
contract_id = "settlement"
chain_id = 1
env = "prod"
address = "0x9008d19f58aabd9ed0d60971565aa8510560ab41"

[[entries]]
contract_id = "vault_relayer"
chain_id = 1
env = "prod"
address = "0xc92e8bdf79f0507f65a392b0ab4667716bfe0110"
```

The manifest is validated twice:

- **At compile time** through `build.rs`. Malformed rows (bad hex,
  duplicate key, unsupported chain, wrong schema version) fail the
  build with a precise diagnostic pointing at the offending row, so
  drift between the TOML manifest and the typed domain types is caught
  before the crate binary boots.
- **At runtime** through `Registry::from_toml_str` when a consumer
  loads their own manifest (for a fork or integration test). The
  runtime parser surfaces each failure as a typed `RegistryError`
  variant: `UnsupportedSchemaVersion`, `UnsupportedChainId`,
  `InvalidAddress`, `DuplicateEntry`, or `Parse { source: toml::de::Error }`.

## Loading A Custom Manifest

Consumers that want to drive the registry from their own TOML pipe the
raw string into `Registry::from_toml_str`:

```rust
use cow_sdk::contracts::{ContractId, Registry};
use cow_sdk::prelude::{CowEnv, SupportedChainId};

let raw = r#"
schema_version = 1

[[entries]]
contract_id = "EthFlow"
chain_id = 100
env = "prod"
address = "0x40a50cf069e992aa4536211b23f286ef88752187"
"#;

let registry = Registry::from_toml_str(raw)
    .expect("custom registry manifest must parse");
let eth_flow = registry
    .address(
        ContractId::EthFlow,
        SupportedChainId::GnosisChain,
        CowEnv::Prod,
    )
    .expect("eth-flow deployment is present");

assert_ne!(
    eth_flow.to_string(),
    "0x0000000000000000000000000000000000000000"
);
```

The parser enforces the same validation rules as `build.rs`, so any
manifest accepted at compile time is also accepted at runtime and any
manifest rejected by the runtime parser would also fail the
compile-time gate.

## Layering A Single Override

When a single address differs from the canonical manifest (a
local-dev settlement contract, a fork-specific deployment, an
integration-test fixture), compose an override on top of
`Registry::default()`:

```rust
use cow_sdk::contracts::{ContractId, Registry};
use cow_sdk::prelude::{Address, CowEnv, SupportedChainId};

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
    registry.address(
        ContractId::Settlement,
        SupportedChainId::Mainnet,
        CowEnv::Prod,
    ),
    Some(local)
);
```

`Registry::with_override` returns a registry with a single entry
replaced; every other entry stays at the canonical value. This
preserves the compile-time completeness of the default manifest while
giving the caller a focused injection point.

## Dependency Posture

The registry is part of `cow-sdk-contracts` and does not pull any
chain-RPC dependency. Resolving an address is a pure in-memory lookup;
the SDK never dispatches a network call on behalf of a `Registry::address`
call. Chain-RPC resolution — for example querying whether a proxy
implementation has been upgraded — flows through the `AsyncProvider`
seam in `cow-sdk-core` and is a separate runtime contract. Native
applications that already use Alloy can satisfy that runtime contract with
`cow-sdk-alloy-provider` for read-only checks or `cow-sdk-alloy` when the same
client also needs signing and transaction submission.

Crates.io owner rotation is not deployment provenance. Publication ownership
for the SDK crate family is tracked separately in
[Publication Handoff](publication-handoff.md), while this page remains limited
to on-chain contract address authority.

## Related Docs

- [Architecture](architecture.md) — how the registry sits inside the
  contracts crate
- [Parity Matrix](parity-matrix.md) — the parity contract covering
  both the `alloy::sol!` bindings and the registry authority
- [ADR 0012](adr/0012-alloy-sol-bindings-and-registry-authority.md) —
  the architectural rule behind the single-authority posture
