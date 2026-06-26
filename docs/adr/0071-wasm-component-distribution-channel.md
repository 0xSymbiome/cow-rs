# ADR 0071: WebAssembly Component Distribution Channel

- Status: Proposed (deferred)
- Date: 2026-06-21
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: wasm, component-model, public-surface, distribution
- Related: [ADR 0010](0010-runtime-neutral-async-and-transport-posture.md), [ADR 0039](0039-typescript-callable-wasm-sdk-surface.md), [ADR 0044](0044-bundle-size-profile-and-flavor-builds.md)

> **What ships today vs. what is planned.** The `cow-sdk-component` crate is a
> workspace member, and CI builds its three worlds for `wasm32-wasip2` and runs
> its native test. The distribution and cross-runtime parity machinery this ADR
> describes — jco and Wasmtime execution, OCI and GitHub Release publishing, and
> a WIT snapshot gate — is **not yet built**: CI only compiles the component. The
> present-tense claims below describe the planned channel, not shipped pipeline
> steps; the crate and its WIT contract are experimental (`0.x`) until that
> machinery lands.

## Decision

The SDK ships a second WebAssembly distribution channel: a WebAssembly Component,
built from an additive leaf crate (`cow-sdk-component`, `publish = false`) that
compiles the deterministic SDK core to `wasm32-wasip2` with `wit-bindgen` against a
published WIT contract. One audited Rust source is then consumable from many
languages and runtimes — JavaScript and TypeScript through jco (Node and the
browser), native hosts through Wasmtime, and composition through the Component
Model — without reimplementing protocol logic.

It is parallel to the wasm-bindgen channel (ADR 0039), not a replacement:

- the wasm-bindgen leaf compiles a JavaScript-coupled core module shipped to npm,
  for JavaScript applications;
- the component leaf compiles a language-neutral component distributed through OCI
  and GitHub Release, for native hosts, polyglot consumers, and composition.

The crate wraps the SDK crates and never forks protocol logic. HTTP and signing are
host imports, not bundled: the stateful surface runs over the SDK's existing
transport seam, and signing is a host import so the private key never enters the
component. The contract carries a deterministic engine world — order identity, chain and
deployment introspection, app-data, and the gas-free on-chain transaction builders —
with no host imports, and a stateful client world for the order lifecycle; both the
WASI 0.2 and WASI 0.3 lanes are in scope behind one shared implementation.

Outputs distribute only through OCI and GitHub Release. The crate is never published
to crates.io — a component-producing `cdylib` is not a `cargo add`-able library, and
the Rust API already ships as the existing crates — and never through Warg.

## Why

The native crates serve Rust; the wasm-bindgen channel (ADR 0039) serves JavaScript
applications. The Component Model serves a consumer neither reaches: any language or
host that wants the audited core through a typed, language-neutral contract. Shipping
a component reuses the SDK's existing seams — the transport trait is already
wasm-ready and keys already stay out of the wasm — so the channel is a thin adapter
behind those seams, not new logic.

It belongs in the main repository, as a governed leaf, for the same reason the
wasm-bindgen leaf does: it wraps the SDK crates, so it must version-lock and
parity-test with them rather than drift in a separate consumer repository.

## Must Remain True

- The crate wraps the SDK crates and never forks protocol logic; the deterministic
  logic stays plain functions and the Component Model bindings stay a thin,
  target-gated `wit-bindgen` wrapper, so native tests and the component share one
  implementation.
- A native golden test pins the reference values; the planned build step asserts
  the component reproduces them through jco and a Wasmtime host (not yet wired —
  CI builds the component but does not execute it).
- The crate is `publish = false` for crates.io and is never published as a Rust
  library.
- Keys never enter the component: signing is a host import and HTTP is a host
  import; the component build excludes the native HTTP client.
- The WIT contract is versioned, and a planned snapshot gate keeps it from
  drifting silently (the gate is deferred — no `.wit` snapshot test exists yet).
- When publishing is wired, outputs distribute only through OCI and GitHub
  Release, behind the pre-1.0 release trigger; never crates.io, and never Warg.
  (No publish step exists yet — CI only builds the component.)
- The two WebAssembly channels stay distinct and documented: wasm-bindgen to npm for
  JavaScript applications; component to OCI for hosts, polyglot consumers, and
  composition.
- The build target is `wasm32-wasip2`, `wit-bindgen` is pinned, and the component is
  built in a dedicated CI job; runtime parity testing in that job is planned, not
  yet wired.
- Pre-1.0, the component and its contract are experimental (`0.x`).
- Consumer demonstrations live in the examples repository, not in this crate.

## Alternatives Rejected

- Publish the component crate to crates.io: a component-producing `cdylib` is not a
  usable Rust library, so it stays `publish = false` like the wasm-bindgen leaf.
- Put the component source in the examples repository: examples are consumers, while
  the producer must release and parity-test alongside the SDK crates.
- Fold the component into the wasm-bindgen crate or its npm package: conflates two
  ABIs — a wasm-bindgen core module and a Component Model component — with different
  toolchains and dependency sets.
- Build a component runtime or host: out of scope; the SDK ships the guest library
  component, not a runtime.
- Reimplement protocol logic inside the component: would fork the audited core.

## Links

- [Architecture](../architecture.md)
- [ADR 0010](0010-runtime-neutral-async-and-transport-posture.md)
- [ADR 0039](0039-typescript-callable-wasm-sdk-surface.md)
- [ADR 0044](0044-bundle-size-profile-and-flavor-builds.md)
