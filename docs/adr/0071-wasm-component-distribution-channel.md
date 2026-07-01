---
type: Decision Record
id: ADR-0071
title: "ADR 0071: WebAssembly Component Distribution Channel"
description: "The SDK ships a second WebAssembly distribution channel: a WebAssembly Component built from an additive leaf crate (cow-sdk-component, publish = false) that compiles the deterministic SDK core to wasm32-wasip2 with wit-bindgen against a published WIT contract, parallel to the wasm-bindgen channel (ADR 0039)."
status: Accepted
date: 2026-06-21
authors: ["0xSymbiotic"]
tags: [wasm, component-model, public-surface, distribution]
related: [ADR-0010, ADR-0039, ADR-0044]
timestamp: 2026-06-21T00:00:00Z
---

# ADR 0071: WebAssembly Component Distribution Channel

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
deployment introspection, app-data, the gas-free on-chain transaction builders, the
signing payloads, event-log decoding, the composable (TWAP) conditional-order
builders, and the pure quote-pipeline helpers (amounts-and-costs, slippage suggestion,
and the app-data document builder) — with no host imports, and a stateful client world
for the order lifecycle;
both the WASI 0.2 and WASI 0.3 lanes are in scope behind one shared implementation.

Outputs distribute only through OCI and GitHub Release. The crate is never published
to crates.io — a component-producing `cdylib` is not a `cargo add`-able library, and
the Rust API already ships as the existing crates — and never through Warg. The engine,
sync-client, and async-client worlds publish as `cow-sdk-component-engine`,
`cow-sdk-component-client-sync`, and `cow-sdk-component-client-async` under
`ghcr.io/0xsymbiome`, versioned `0.1.0-alpha.x`.

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
- A native golden test pins the reference values and runs in CI; runtime
  reproduction through jco and a Wasmtime host is not yet exercised in CI, so the
  golden remains the native-only reference.
- The crate is `publish = false` for crates.io and is never published as a Rust
  library.
- Keys never enter the component: signing is a host import and HTTP is a host
  import; the component build excludes the native HTTP client.
- The WIT contract is versioned, and a planned snapshot gate keeps it from
  drifting silently (the gate is deferred — no `.wit` snapshot test exists yet).
- Outputs distribute only through OCI and GitHub Release; never crates.io, and
  never Warg. The three worlds publish as `cow-sdk-component-engine`,
  `cow-sdk-component-client-sync`, and `cow-sdk-component-client-async` under
  `ghcr.io/0xsymbiome`.
- The two WebAssembly channels stay distinct and documented: wasm-bindgen to npm for
  JavaScript applications; component to OCI for hosts, polyglot consumers, and
  composition.
- The build target is `wasm32-wasip2`, `wit-bindgen` is pinned, and CI builds all
  three worlds and runs the engine golden in a dedicated job; cross-runtime parity
  testing in that job is not yet wired.
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

- [Architecture](../guides/architecture.md)
- [ADR 0010](0010-runtime-neutral-async-and-transport-posture.md)
- [ADR 0039](0039-typescript-callable-wasm-sdk-surface.md)
- [ADR 0044](0044-bundle-size-profile-and-flavor-builds.md)
