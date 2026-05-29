# WASM Public API Stability Audit

Status: Current
Last reviewed: 2026-05-29
Owning surface: `cow-sdk-wasm` TypeScript facade declarations, package exports, runtime config shapes, error envelopes, and transport policy configuration
Refresh trigger: Changes to facade declarations, package export maps, raw wasm-bindgen exports, TypeScript config objects, transport policy fields, or JS-visible error envelope schema
Related docs:
- [ADR 0039](../adr/0039-typescript-callable-wasm-sdk-surface.md)
- [ADR 0041](../adr/0041-transport-policy-l3-layering.md)
- [ADR 0044](../adr/0044-bundle-size-profile-and-flavor-builds.md)
- [ADR 0046](../adr/0046-transport-policy-js-exposure.md)
- [ADR 0047](../adr/0047-typescript-facade-architecture.md)
- [PROPERTIES.md](../../PROPERTIES.md)

## Scope

This audit covers:

- compiled TypeScript facade declarations for each package flavor
- package exports and raw-export denylist behavior
- single-object client constructor config shapes
- `TransportPolicyConfig` translation for HTTP-capable clients
- `SdkError` and wasm envelope schema-version compatibility

It does not cover final npm package naming, npm publication, or application
code outside the repository fixtures.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Facade declarations | Public declarations are flavor-scoped and hide raw wasm-bindgen internals | Conforms |
| Export map | Public package imports resolve through declared facade subpaths | Conforms |
| Constructor shape | HTTP-capable clients accept a single typed config object, including transport and policy fields | Conforms |
| Transport policy | JavaScript `TransportPolicyConfig` maps into the shared Rust policy and rejects invalid values | Conforms |
| Error compatibility | Envelopes carry `schemaVersion` and preserve an unknown-variant sentinel for forward compatibility | Conforms |

## Current Contract

### Public Facade Surface

The facade snapshots under `crates/wasm/snapshots/facade/` are the reviewed
TypeScript contract for each package flavor. They expose camelCase methods,
named callback types, typed config objects, `dispose`, `SdkError`, and
runtime-specific initialization helpers. Every flavor that bundles the signing
capability also exposes the deterministic `decodeSettlementLog` and
`decodeEthFlowLog` helpers and their `EventLogInput` / `SettlementEventDto` /
`EthFlowEventDto` declarations.

### Package Export Stability

Package exports are rendered from the package template and verified after
build. Public entries resolve to facade modules; raw wasm-bindgen outputs are
generated package artifacts and are not public import targets.

### Transport Policy And Error Envelope

HTTP-capable constructors accept `TransportPolicyConfig` and translate it into
the Rust policy. Error envelopes include `schemaVersion`, low-cardinality
error fields, and a scoped `__unknown` sentinel so newer variants can be
handled without losing the raw payload.

## Evidence

Primary implementation points:

- `crates/wasm/npm/src/index.ts`
- `crates/wasm/npm/src/default.ts`
- `crates/wasm/npm/src/orderbook.ts`
- `crates/wasm/npm/src/signing.ts`
- `crates/wasm/npm/src/cloudflare.ts`
- `crates/wasm/npm/src/errors.ts`
- `crates/wasm/npm/src/envelope.ts`
- `crates/wasm/npm/src/options.ts`
- `crates/wasm/npm/scripts/verify-exports.mjs`
- `crates/wasm/npm/scripts/verify-no-raw-exports.mjs`
- `crates/wasm/npm/scripts/verify-facade-denylist.mjs`
- `crates/wasm/snapshots/facade/`

Primary regression coverage:

- `crates/wasm/tests/wasm_facade_snapshot_contract.rs::facade_declarations_match_flavour_matrix`
- `crates/wasm/tests/wasm_facade_snapshot_contract.rs::facade_declarations_hide_raw_wasm_bindgen_surface`
- `crates/wasm/tests/wasm_facade_snapshot_contract.rs::facade_declarations_expose_dispose_and_named_callback_types`
- `crates/wasm/tests/wasm_snapshot_surface_contract.rs::generated_type_declarations_expose_transport_policy_config_for_http_flavours`
- `crates/wasm/tests/wasm_transport_policy_contract.rs::all_client_constructors_accept_transport_policy`
- `crates/wasm/tests/wasm_transport_policy_contract.rs::invalid_transport_policy_user_agent_is_rejected`
- `crates/wasm/tests/wasm_envelope_contract.rs::envelope_serializes_schema_version_and_payload`
- `crates/wasm/tests/wasm_envelope_contract.rs::envelope_preserves_unknown_schema_sentinel`

Validation surface:

```text
cargo test -p cow-sdk-wasm --test wasm_facade_snapshot_contract
cargo test -p cow-sdk-wasm --test wasm_snapshot_surface_contract
wasm-pack test crates/wasm --headless --chrome
bash crates/wasm/npm/scripts/build.sh
node crates/wasm/npm/scripts/verify-exports.mjs
node crates/wasm/npm/scripts/verify-no-raw-exports.mjs
node crates/wasm/npm/scripts/verify-facade-denylist.mjs
pnpm --dir crates/wasm/npm test
```
