# ADR 0047: Make The TypeScript Facade The Public WASM Package Surface

- Status: Superseded by [ADR 0039](0039-typescript-callable-wasm-sdk-surface.md)
- Date: 2026-05-11
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: wasm, typescript, facade, npm

## Superseded

The rule that the compiled TypeScript facade is the public npm surface (raw
wasm-bindgen output staying an internal `dist/raw/` artifact) is recorded in
[ADR 0039](0039-typescript-callable-wasm-sdk-surface.md), whose decision and
Must-Remain-True already make the TypeScript facade the canonical published
surface.
