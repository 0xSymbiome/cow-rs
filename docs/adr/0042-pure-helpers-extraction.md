# ADR 0042: Extract Pure WASM Helpers Into `cow-sdk-pure-helpers`

- Status: Accepted
- Date: 2026-05-11
- Last reviewed: 2026-05-11
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: wasm, crate-boundary, pure-helpers, component-model
- Related: [ADR 0008](0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md), [ADR 0039](0039-typescript-callable-wasm-sdk-surface.md), [ADR 0047](0047-typescript-facade-architecture.md)

## Decision

Deterministic helper logic used by `cow-sdk-wasm` lives in
`cow-sdk-pure-helpers`. The wasm crate owns JavaScript ABI exports and facade
adaptation; the pure-helper crate owns host-safe protocol composition for
chain parsing, app-data helpers, typed-data payloads, EIP-1271 payloads,
order UIDs, digest strings, DTO envelopes, and stable helper errors.

## Why

The TypeScript-callable package needs JavaScript interop, but its deterministic
protocol helpers should be reusable from native tests and any future adapter
that does not use wasm-bindgen. A pure leaf crate keeps those helpers reviewable
without `JsValue`, callback registry, or package-generation concerns.

## Must Remain True

- Public package entry points continue to flow through the TypeScript facade and
  wasm export crate.
- `cow-sdk-pure-helpers` stays free of wasm-bindgen, `js-sys`, and `web-sys`.
- Host tests prove helper output parity with the wasm-facing signing and
  app-data modules.
- Adding a helper that needs JavaScript state keeps that state in
  `cow-sdk-wasm`, not in the pure-helper crate.
- The extra crate boundary is accepted so host-safe helper logic remains
  testable without a wasm runtime.

## Alternatives Rejected

- Keep all helpers inside `cow-sdk-wasm`: fewer crates, but deterministic logic
  would remain tied to wasm-bindgen and JavaScript ABI dependencies.
- Move the helpers into `cow-sdk-core`: simpler reuse, but it would pull
  app-data, signing, and TypeScript-facing helper cadence into the core
  transport crate.

## Links

- [cow-sdk-pure-helpers README](../../crates/pure-helpers/README.md)
- [WASM Component Model Future Prep Audit](../audit/wasm-component-model-future-prep-audit.md)
- [WASM Surface Audit](../audit/wasm-surface-audit.md)

**Proven by:**

- [WASM Component Model Future Prep Audit](../audit/wasm-component-model-future-prep-audit.md)
- [WASM Surface Audit](../audit/wasm-surface-audit.md)
