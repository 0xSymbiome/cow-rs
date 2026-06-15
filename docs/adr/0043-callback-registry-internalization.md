# ADR 0043: Keep WASM Callback Registries Internal To Client Constructors

- Status: Superseded by [ADR 0039](0039-typescript-callable-wasm-sdk-surface.md)
- Date: 2026-05-11
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: wasm, callbacks, lifetime, public-surface

## Superseded

The rule that JavaScript callback registries stay internal to client
constructors — never exposed as public handle, registry-id, or disposal types —
is recorded in [ADR 0039](0039-typescript-callable-wasm-sdk-surface.md), whose
Must-Remain-True keeps callback registries module-scoped, off the public
TypeScript surface, and reached only through one typed config object.
