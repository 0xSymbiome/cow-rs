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

## Example

Derive the deterministic CREATE2 proxy address for a `(version, factory, user)`
triple. The same inputs always produce the same proxy address:

```rust
use cow_sdk_cow_shed::{CowShedVersion, ProxyAddress, proxy_of};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let factory: ProxyAddress = "0x312f92fe5f1710408B20D52A374fa29e099cFA86".parse()?;
let user: ProxyAddress = "0x76b0340e50BD9883D8B2CA5fd9f52439a9e7Cf58".parse()?;

let proxy = proxy_of(CowShedVersion::V1_0_1, factory, user);
assert_eq!(
    proxy,
    "0x66545B93A314e5BdEC9E5Ff9c4D2C7054e6afb04".parse::<ProxyAddress>()?,
);
# Ok(())
# }
```

Most consumers reach these helpers through the trading facade; depend on this
crate directly only when building hook or proxy orchestration.

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
