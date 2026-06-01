# cow-sdk-pure-helpers

`cow-sdk-pure-helpers` contains runtime-neutral helper functions used by
`cow-sdk-wasm` to expose deterministic CoW Protocol SDK behavior to JavaScript
and TypeScript callers.

The crate owns helper modules for chain support, DTO conversion, app-data
document handling, signing payload construction, EIP-1271 payload construction,
and order UID formatting. It depends only on Rust SDK crates and common
serialization primitives, so it can be reused by host-side Rust tests and future
adapter crates without depending on JavaScript FFI bindings.

Application code does not depend on this crate directly. Reach the same behavior
through `cow-sdk-wasm` for JavaScript and TypeScript callers, or the `cow-sdk`
facade for Rust; this crate is an internal substrate.

FFI-specific code belongs in consumer crates such as `cow-sdk-wasm`. This crate
does not depend on wasm binding, JavaScript system, browser system, or
TypeScript binding crates.
