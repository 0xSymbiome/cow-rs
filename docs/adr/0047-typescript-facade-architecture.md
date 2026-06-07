# ADR 0047: Make The TypeScript Facade The Public WASM Package Surface

- Status: Accepted
- Date: 2026-05-11
- Last reviewed: 2026-05-11
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: wasm, typescript, facade, npm
- Related: [ADR 0039](0039-typescript-callable-wasm-sdk-surface.md), [ADR 0042](0042-pure-helpers-extraction.md), [ADR 0043](0043-callback-registry-internalization.md), [ADR 0044](0044-bundle-size-profile-and-flavor-builds.md)

## Decision

The npm package for `cow-sdk-wasm` exposes a compiled TypeScript facade as the
public SDK surface. Raw wasm-bindgen output remains an internal package
artifact under `dist/raw/`, while public exports point to compiled facade
modules selected by flavor and runtime target.

## Why

Raw wasm-bindgen declarations mirror Rust export mechanics rather than a stable
TypeScript SDK. The facade gives consumers stable camelCase methods, named
callback types, single-object constructors, explicit disposal, normalized error
envelopes, and package subpaths that can evolve independently from raw binding
details.

## Must Remain True

- Public imports use package exports and facade modules, never deep raw
  wasm-bindgen paths.
- Facade declarations expose named callbacks, typed config objects,
  `TransportPolicyConfig`, `CowError`, and explicit `dispose` behavior.
- Raw callback registry handles and generated wasm-bindgen classes remain
  hidden from public declarations.
- Error envelopes carry `schemaVersion` and preserve an unknown-variant escape
  hatch for forward compatibility.
- The facade may own JavaScript adapter state, but deterministic protocol logic
  stays in Rust crates.

## Alternatives Rejected

- Publish raw wasm-bindgen output as the SDK: quickest to ship, but it exposes
  generated names and lifetime details as public contract.
- Split the facade into a separate npm package immediately: cleaner package
  layering, but it adds a second package name and version before publication.
- Rely only on handwritten TypeScript docs: readable, but without compiled
  declarations and tests it would not prove the consumer contract.

## Links

- [WASM npm README](../../crates/wasm/npm/README.md)
- [WASM Facade Architecture Audit](../audit/wasm-facade-architecture-audit.md)
- [WASM Public API Stability Audit](../audit/wasm-public-api-stability-audit.md)
- [WASM Schema Versioning Policy Audit](../audit/wasm-schema-versioning-policy-audit.md)

**Proven by:**

- [WASM Facade Architecture Audit](../audit/wasm-facade-architecture-audit.md)
- [WASM Public API Stability Audit](../audit/wasm-public-api-stability-audit.md)
- [WASM Schema Versioning Policy Audit](../audit/wasm-schema-versioning-policy-audit.md)
