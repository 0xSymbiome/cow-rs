# ADR 0064: App-Data Validation Is Typed By Construction, Not JSON-Schema

- Status: Accepted
- Date: 2026-06-03
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: app-data, validation, typing
- Related: [ADR-0018](0018-typed-app-data-merge.md), [ADR-0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md), [ADR-0052](0052-alloy-primitives-canonical-primitive-layer.md)

## Decision

`cow-sdk-app-data` validates app-data documents through typed Rust construction
plus a small set of structural checks, not through a runtime JSON-Schema
validator.

- A document is valid when it carries a `<major>.<minor>.<patch>` `version`
  string and every metadata family the SDK models validates.
- Families the SDK models and the reviewed services parser also models —
  `flashloan` and `partnerFee` — are validated strictly: a present-but-malformed
  value is rejected with a typed, field-named error that never echoes the
  caller-supplied bytes.
- `quote` is bound-checked opportunistically: the slippage bound is enforced when
  the value is in the current typed shape, while earlier wire shapes carried by
  historical documents pass through so they continue to hash.
- Unmodeled metadata families pass through unchanged, so the SDK is no stricter
  than the orderbook's own acceptance contract.
- The schema bundle is reduced to one self-contained drift fixture per modeled
  metadata family (`flashloan`, `partnerFee`, `quote`, and the `hook` shape)
  retained under `parity/fixtures/app_data/schemas/`; a drift test asserts the typed
  structs still cover the upstream field names, so an upstream rename or addition
  fails review rather than diverging silently. The root-envelope schema, the
  unmodeled-family sub-schemas, the shared `definitions.json`, and the
  byte-for-byte schema-vendoring tooling (the `vendor-app-data-schemas` command)
  are removed: nothing resolves the schema graph at runtime, so a flat fixture
  per modeled family is the whole drift surface.

## Why

The protocol's own Rust backend models app-data with typed serde structs and
hashes the original bytes; it carries no JSON-Schema validator. Mirroring that
posture removes a runtime dependency surface (`jsonschema`, `include_dir`, and an
embedded schema bundle) and unifies validation with the typed merge pipeline
already required by [ADR 0018](0018-typed-app-data-merge.md). Strong typed domain
values are the default public contract under
[ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md)
and [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md); the type
system, not a dynamic schema, is the validation authority.

## Must Remain True

- Public surface: `validate_app_data_doc(&AppDataDoc) -> Result<(), AppDataError>`
  (a valid document is `Ok(())`, a failure the typed field-named `AppDataError`)
  and `AppDataParams::into_validated` are the validation entries; the
  `ValidationResult { success, errors }` struct is removed from `cow-sdk-app-data`
  and survives only in the `cow-sdk-wasm` DTO layer. `get_app_data_schema` and the
  per-family `LATEST_*_METADATA_VERSION` constants are removed; `SchemaVersion`
  remains the typed semver version.
- Runtime and support: the crate carries no JSON-Schema validator or embedded
  schema bundle at runtime; validation is typed plus structural and the
  keccak/CID hashing path is unchanged, so previously valid documents keep their
  digests.
- Validation and review: typed-construction bounds (addresses, amounts, basis
  points), the document-size ceiling, and the schema-drift fixtures stay covered
  by the crate's contract tests; the drift test must fail when an upstream field
  name the typed structs depend on changes.
- Cost: validation no longer rejects every malformed shape of an unmodeled or
  earlier-versioned metadata family — the SDK is intentionally no stricter than
  the orderbook for metadata it does not model.

## Alternatives Rejected

- Keep the embedded JSON-Schema validator: retains a runtime dependency surface
  and a dynamic-validation mechanism inconsistent with the typed merge pipeline,
  and validates families for capabilities the SDK does not implement.
- Pure typed validation with no retained schemas: loses the forward-compatibility
  signal when an upstream field is renamed or added; the drift fixtures plus the
  drift test restore that signal at test-only cost.

## Links

- [Typed App-Data Merge](0018-typed-app-data-merge.md)
- [Strong Domain Types](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md)
- [App-Data Crate README](../../crates/app-data/README.md)
