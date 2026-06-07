# WASM Schema Versioning Policy Audit

Status: Current
Last reviewed: 2026-05-11
Owning surface: `cow-sdk-wasm` JavaScript-visible success and error envelopes, schema-version fields, and unknown-variant escape hatches
Refresh trigger: Changes to `WasmError`, `CowError`, envelope serialization, schema-version constants, unknown variant handling, or TypeScript facade error normalization
Related docs:
- [ADR 0047](../adr/0047-typescript-facade-architecture.md)
- [WASM Public API Stability Audit](wasm-public-api-stability-audit.md)
- [PROPERTIES.md](../../PROPERTIES.md)

## Scope

This audit covers:

- JavaScript-visible success envelopes that carry `schemaVersion`
- error envelopes normalized into `CowError`
- the `__unknown` sentinel used for forward-compatible variant handling
- TypeScript facade handling of known and unknown envelope shapes

It does not cover service API schema versioning or upstream OpenAPI evolution.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Success envelopes | JavaScript-visible helper outputs include a stable `schemaVersion` field beside the payload | Conforms |
| Unknown variants | Unknown enum variants preserve raw payload data behind a scoped `__unknown` sentinel | Conforms |
| Error normalization | Facade errors normalize raw wasm failures into schema-versioned `CowError` values | Conforms |
| Type declarations | Public declarations expose versioned error and output shapes without raw wasm-bindgen internals | Conforms |

## Current Contract

### Envelope Versioning

WASM helper outputs serialize through an envelope that carries `schemaVersion`
and `payload` fields. The version identifies the JavaScript-visible envelope
shape, not the CoW Protocol service schema.

### Unknown Variant Handling

Unknown variants round-trip through a scoped `__unknown` sentinel that keeps
the raw payload available while preventing unrecognized variants from being
misclassified as known SDK states.

### Facade Error Normalization

The TypeScript facade maps raw wasm errors into `CowError` values that preserve
the schema version, known discriminants, redacted fields, and unknown-variant
fallback behavior.

## Evidence

Primary implementation points:

- `crates/wasm/src/exports/envelope.rs`
- `crates/wasm/src/exports/errors.rs`
- `crates/wasm/npm/src/envelope.ts`
- `crates/wasm/npm/src/errors.ts`
- `crates/wasm/snapshots/facade/`

Primary regression coverage:

- `crates/wasm/tests/wasm_envelope_contract.rs::envelope_serializes_schema_version_and_payload`
- `crates/wasm/tests/wasm_envelope_contract.rs::envelope_preserves_unknown_schema_sentinel`
- `crates/wasm/tests/wasm_error_abi_contract.rs::unknown_enum_variant_round_trips`
- `crates/wasm/tests/wasm_error_abi_contract.rs::unknown_sentinel_round_trips_raw_payload`
- `crates/wasm/tests/wasm_snapshot_surface_contract.rs::generated_type_declarations_version_errors_and_outputs`
- `crates/wasm/npm/tests/facade-error-normalization.test.ts`

Validation surface:

```text
cargo test -p cow-sdk-wasm --test wasm_envelope_contract
cargo test -p cow-sdk-wasm --test wasm_error_abi_contract
cargo test -p cow-sdk-wasm --test wasm_snapshot_surface_contract
pnpm --dir crates/wasm/npm test
```
