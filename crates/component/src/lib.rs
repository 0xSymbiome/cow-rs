//! The `CoW` Protocol SDK as a WebAssembly Component.
//!
//! This crate compiles the SDK to a WebAssembly Component against the
//! [`wit/world.wit`](../wit/world.wit) contract (`cow:protocol`). One audited Rust
//! source is then consumable from many languages and runtimes: JavaScript and
//! TypeScript through `jco` (Node and the browser), native hosts through
//! Wasmtime, and composition through WAC.
//!
//! Each world is one component, selected by exactly one feature:
//!
//! - `world-engine` (default) exports the deterministic interfaces — order
//!   identity, chain and deployment introspection, app-data, the gas-free
//!   transaction builders, the signing payloads, event-log decoding, the
//!   composable (TWAP) conditional-order builders, and the trading-math quote
//!   helpers (amounts-and-costs, slippage suggestion, and the app-data document
//!   builder); it has no host imports.
//! - `world-client-sync` exports the orderbook read/write and trading lifecycle
//!   over the WASI 0.2 HTTP lane.
//! - `world-client-async` exports the same surface over the WASI 0.3 HTTP lane.
//!
//! The deterministic logic wraps `cow-sdk-core`, `cow-sdk-signing`,
//! `cow-sdk-app-data`, `cow-sdk-contracts`, and the pure quote helpers from
//! `cow-sdk-trading`; the stateful lanes run the real
//! `cow-sdk-orderbook` and `cow-sdk-trading` clients over the SDK's
//! `HttpTransport` seam. HTTP and signing are host imports: the host signs the
//! digest through the `signer` import, so the private key never enters the
//! component.

// The deterministic engine lane — order identity, the gas-free on-chain
// transaction builders, the signing payloads, and event-log decoding — all
// pure, with no host imports. Built for the component target and the native
// golden test; absent from a plain native build, where this crate is empty.
#[cfg(any(test, all(target_arch = "wasm32", feature = "world-engine")))]
mod engine;

// The stateful client lanes — the shared lifecycle + orderbook dispatch and the
// two world bindings (synchronous over WASI 0.2, asynchronous over WASI 0.3) —
// run the real orderbook and trading clients over the SDK `HttpTransport` seam.
// HTTP, signing, and contract reads are host imports, so no private key or RPC
// socket enters the component.
#[cfg(all(
    target_arch = "wasm32",
    any(feature = "world-client-sync", feature = "world-client-async")
))]
mod client;

// One cdylib is one component, so exactly one world feature must be selected.
#[cfg(all(
    target_arch = "wasm32",
    not(any(
        all(
            feature = "world-engine",
            not(feature = "world-client-sync"),
            not(feature = "world-client-async")
        ),
        all(
            feature = "world-client-sync",
            not(feature = "world-engine"),
            not(feature = "world-client-async")
        ),
        all(
            feature = "world-client-async",
            not(feature = "world-engine"),
            not(feature = "world-client-sync")
        )
    ))
))]
compile_error!(
    "select exactly one world: build with --no-default-features --features \
     world-engine | world-client-sync | world-client-async"
);
