# cow-sdk-component

The CoW Protocol Rust SDK compiled to a **WebAssembly Component** against a typed
WIT contract, so one audited Rust source is consumable from many languages and
runtimes: JavaScript/TypeScript through [`jco`](https://bytecodealliance.github.io/jco/)
(Node and the browser), native hosts through Wasmtime, and composition through
[WAC](https://github.com/bytecodealliance/wac).

It is the second WASM distribution lane of the SDK, alongside the wasm-bindgen
crate `cow-sdk-js` (which targets npm for JavaScript apps). This crate targets
`wasm32-wasip2` and distributes as a component through OCI and GitHub Release,
never crates.io.

The crate wraps `cow-sdk-core`, `cow-sdk-signing`, `cow-sdk-orderbook`, and
`cow-sdk-trading`; it never forks protocol logic. HTTP and signing are host
imports, not bundled: the stateful lanes run over the SDK's `HttpTransport` seam,
and signing is a host import, so the private key stays out of the component.

## Worlds

One world is one component; build with exactly one world feature.

| World | Feature | Exports | Imports |
| --- | --- | --- | --- |
| `order-engine` | `world-engine` (default) | `order`, `chains`, `app-data`, `tx`, `composable`, `trading-math`, `order-signing`, `events` | none |
| `client-sync` | `world-client-sync` | `orderbook-read`, `orderbook-write`, `trading` (sync) | `signer`, `contract-read`, `wasi:http@0.2` |
| `client-async` | `world-client-async` | `orderbook-read-async`, `orderbook-write-async`, `trading-async` | `signer`, `contract-read`, `wasi:http@0.3` |

## Pull

Each world ships as an OCI package under `ghcr.io/0xsymbiome`:

| World | Package |
| --- | --- |
| `order-engine` | `cow-sdk-component-engine` |
| `client-sync` | `cow-sdk-component-client-sync` |
| `client-async` | `cow-sdk-component-client-async` |

```text
wkg oci pull ghcr.io/0xsymbiome/cow-sdk-component-engine:0.1.0-alpha.9
```

## Build

```text
rustup target add wasm32-wasip2
cargo build -p cow-sdk-component --target wasm32-wasip2 --release
cargo build -p cow-sdk-component --target wasm32-wasip2 --release \
  --no-default-features --features world-client-sync
cargo build -p cow-sdk-component --target wasm32-wasip2 --release \
  --no-default-features --features world-client-async
```

The WIT contract is [`wit/world.wit`](wit/world.wit). Consumer demonstrations
live in the [`0xSymbiome/cow-sdk-examples`](https://github.com/0xSymbiome/cow-sdk-examples)
repository.
