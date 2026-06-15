# ADR 0042: Extract Pure WASM Helpers Into `cow-sdk-pure-helpers`

- Status: Superseded (2026-06-09)
- Date: 2026-05-11
- Last reviewed: 2026-06-09
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: wasm, crate-boundary, pure-helpers, component-model
- Related: [ADR 0008](0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md), [ADR 0039](0039-typescript-callable-wasm-sdk-surface.md), [ADR 0047](0047-typescript-facade-architecture.md)

## Superseded 2026-06-09: helpers folded into `cow-sdk-wasm::helpers`

The deterministic helper logic now lives in the non-FFI `helpers` module of
`cow-sdk-wasm` (`crates/wasm/src/helpers/`) rather than in a standalone
`cow-sdk-pure-helpers` crate, which has been removed. The two boundary
properties that justified the original split are **retained at the module
level**, not the crate level:

- The `helpers` module carries no JavaScript FFI bindings and compiles for both
  native and `wasm32-unknown-unknown` targets. This is enforced by
  `crates/wasm/tests/no_ffi_helpers.rs`, which scans `src/helpers` for FFI
  tokens, and re-strengthened by the host build gate `cargo check -p
  cow-sdk-wasm --no-default-features`, which now compiles the helper logic on
  the native target.
- Helper output parity with the wasm-facing signing and app-data modules is
  proven by `crates/wasm/tests/host_pure_helpers.rs`.

The crate-level rationale did not hold under review: `cow-sdk-wasm` is the only
consumer, the crate was not part of the published release family, and the
"future non-wasm-bindgen adapter" reuse path was a documented posture rather
than a present consumer. A single non-FFI module inside the wasm crate keeps the
host-safe boundary while removing a workspace crate whose name did not name a
domain. The JavaScript ABI surface (`exports`) stays `wasm32`-gated exactly as
before.

The historical decision text below is retained as design history.

---

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
- The host-safe helper modules stay free of wasm-bindgen, `js-sys`, and
  `web-sys` (now enforced at the `cow-sdk-wasm::helpers` module boundary).
- Host tests prove helper output parity with the wasm-facing signing and
  app-data modules.
- Adding a helper that needs JavaScript state keeps that state in the
  `cow-sdk-wasm` `exports` surface, not in the host-safe helper modules.

## Alternatives Rejected

- Move the helpers into `cow-sdk-core`: simpler reuse, but it would pull
  app-data, signing, and TypeScript-facing helper cadence into the core
  transport crate.

## Links

- [WASM Component Model Future Prep Audit](../audit/wasm-component-model-future-prep-audit.md)
- [WASM Surface Audit](../audit/wasm-surface-audit.md)

**Proven by:**

- [WASM Component Model Future Prep Audit](../audit/wasm-component-model-future-prep-audit.md)
- [WASM Surface Audit](../audit/wasm-surface-audit.md)
