# ADR 0043: Keep WASM Callback Registries Internal To Client Constructors

- Status: Accepted
- Date: 2026-05-11
- Last reviewed: 2026-05-11
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: wasm, callbacks, lifetime, public-surface
- Related: [ADR 0039](0039-typescript-callable-wasm-sdk-surface.md), [ADR 0040](0040-wallet-provider-callback-boundary-for-js-consumers.md), [ADR 0054](0054-typescript-facade-architecture.md)

## Decision

JavaScript callback registration is internal runtime state owned by
`cow-sdk-wasm` clients and the TypeScript facade. Public callers construct
clients with one typed configuration object containing callbacks, timeout
options, abort signals, and transport policy. They do not manually construct
callback handle types, registry ids, or registry disposal objects.

## Why

Callback ids are module-local lifetime machinery, not SDK business concepts.
Exposing them would make consumers reason about wasm module instances,
double-disposal, leaked callbacks, and handle reuse. Constructor-owned
registration gives the SDK one place to bind callbacks, dispose state, and map
failures into typed JavaScript errors.

## Must Remain True

- Public TypeScript declarations do not expose callback registry classes,
  handle constructors, or manual registration helpers.
- Registry state remains scoped to one wasm module instance and reserves zero
  as an invalid handle.
- Client constructors and facade classes own callback retention and disposal.
- Callback throws, rejects, malformed outputs, timeout overflow, and aborts map
  to typed `WasmError` variants.
- Callers still pass their own wallet, signer, and fetch callbacks explicitly.

## Alternatives Rejected

- Expose registry handles for advanced callers: flexible, but it would turn an
  implementation lifetime detail into public API.
- Use one process-wide callback registry: simpler ids, but it would mix module
  instances and make disposal harder to audit.
- Store JavaScript functions inside public Rust traits: compact, but it would
  break the pure `Send + Sync` provider boundary.

## Links

- [WASM Callback Shape Design Audit](../audit/wasm-callback-shape-design-audit.md)
- [WASM Type Generation Audit](../audit/wasm-type-generation-audit.md)
- [WASM Surface Audit](../audit/wasm-surface-audit.md)

**Proven by:**

- [WASM Callback Shape Design Audit](../audit/wasm-callback-shape-design-audit.md)
- [WASM Type Generation Audit](../audit/wasm-type-generation-audit.md)
