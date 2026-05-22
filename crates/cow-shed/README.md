# cow-sdk-cow-shed

`cow-sdk-cow-shed` contains the pure COW Shed building blocks used by higher
SDK layers: version selection, generated ABI bindings, deterministic proxy
address derivation, EIP-712 domain and message hashing, and calldata encoders
for hook execution.

The proxy address type is the cow `Address` newtype around
`alloy_primitives::Address` per
[ADR 0052](https://github.com/cowdao-grants/cow-rs/blob/main/docs/adr/0052-alloy-primitives-canonical-primitive-layer.md);
CREATE2 init-code hashing routes through `alloy_primitives::keccak256`,
and EIP-712 typed-data structs (`Call`, `ExecuteHooks`) are macro-emitted
by `alloy_sol_types::sol!`.

The crate keeps four authorities separated:

- the deployed factory ABI defines factory calldata, including
  `initializeProxy(address,bool)`
- the Solidity implementation defines type strings, struct layout, and hash
  algorithms
- deployment rows define chain and address availability
- version-keyed constants pin factory, implementation, and proxy creation code

Feature flags are opt-in. `cow-shed-ens` exposes ENS-oriented factory getters,
`with_ens` is a compatibility alias for that flag, and `cow-shed-gnosis`
exposes the Gnosis-only composable forwarder binding. Default builds avoid
provider and signer adapters.

The MSRV is Rust 1.94.0. This crate does not run service loops, persistence
adapters, polling cadences, or notification workflows; consumers build those
orchestration layers above the pure helpers.
