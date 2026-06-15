# ADR 0018: Typed App-Data Merge As The Single Canonical Quote-To-Post Edit Path

- Status: Accepted
- Date: 2026-04-22
- Last reviewed: 2026-05-22
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: trading, app-data, metadata, validation, typed-boundaries
- Related: [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md), [ADR 0011](0011-typed-amount-boundary-and-typestate-ready-state-construction.md), [ADR 0015](0015-client-side-order-bounds-validator.md), [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md)

## Decision

Quote-to-post app-data edits in `cow-sdk-trading` run through a
single typed merge pipeline: the base wire document is parsed back
into `cow_sdk_app_data::AppDataParams` through the crate's
`Deserialize` impl, the override merges through
`merge_app_data_params(base, override) -> AppDataParams`, and the
merged typed value is re-emitted as the wire document through the
existing `cow_sdk_app_data::generate_app_data_doc` plus
`cow_sdk_app_data::app_data_info` pipeline. The public helper
`cow_sdk_trading::merge_and_seal_app_data(base_doc,
override_params)` returns both the `TradingAppDataInfo` and the
typed merged `AppDataParams` so downstream validators read the
typed `signer` field from the same merged value that produced the
uploaded document. The `serde_json::Value`-taking merge helper
previously exported from the trading crate is retired; no public
quote-to-post edit path touches the wire shape directly.

## Why

Mixing an opaque `serde_json::Value` base document with a typed
override in one merge helper produced three separate correctness
defects at once. The typed `signer` and `flashloan` fields on the
override are only lifted onto the wire through
`AppDataParams::metadata_wire_value`, so a merge helper that reads
the override's open-ended `metadata` map without performing that
lift silently drops both fields before the hash is computed. The
submission-seam validator reads the declared signer from the same
override object, so a base document that already carries
`metadata.signer` escapes the `AppdataFromMismatch` check when the
caller does not re-supply the override. The reviewed services
authority clears `metadata.hooks` before override merge; a
recursive deep-merge on the `metadata` map instead preserves stale
`pre` or `post` hook siblings across edits. Collapsing the
pipeline onto typed merge forces the typed-to-wire lift through a
single point — `generate_app_data_doc` — and makes every one of
the three defects unreachable without reintroducing an opaque
`Value`-merge helper.

## Must Remain True

- Public surface: `cow_sdk_trading::merge_and_seal_app_data` is the
  only canonical quote-to-post merge helper, and its public
  signature is `fn merge_and_seal_app_data(base_doc: &Value,
  override_params: &AppDataParams) -> Result<(TradingAppDataInfo,
  AppDataParams), TradingError>`. The companion
  `cow_sdk_trading::params_from_doc(base_doc: &Value) ->
  Result<AppDataParams, TradingError>` exposes the typed
  re-parse step as a separate reviewable unit for downstream
  composition. Both helpers re-export through the `cow-sdk`
  facade's `trading` module. No `serde_json::Value`-taking
  merge helper is exposed from the trading crate.
- Runtime and support: `merge_app_data_params` preserves the full
  typed `AppDataParams` contract (`app_code`, `environment`,
  `signer`, `flashloan`, and the open-ended `metadata` slot) with
  override precedence, and applies the hooks-replacement rule —
  when the override's metadata carries a `"hooks"` entry, the
  base's `metadata.hooks` is dropped before the recursive metadata
  merge. Arrays on sibling metadata keys (including
  `metadata.userConsents`) continue to replace rather than
  concatenate under the merge helper's object-aware fall-through.
  The submission-seam `app_data_signer` derivation reads
  `merged_params.signer.clone()` from the typed merged params
  produced by `merge_and_seal_app_data`; the override-only read on
  `advanced_settings.app_data.signer` is gone from the
  quote-to-post path.
- Validation and review: the typed merge pipeline is locked down
  by the regression module at
  `crates/trading/tests/app_data_merge_contract.rs`. The module
  exercises override-only-signer survival, override-only-flashloan
  survival, simultaneous survival, the hooks replacement rule, the
  hooks preservation boundary without a hooks override, the
  `AppdataFromMismatch` detection when the base document carries
  signer metadata, the override-signer precedence, the
  extra-top-level-key preservation, the round-trip idempotency of
  `params_from_doc(generate_app_data_doc(p)) == p`, and the
  `metadata.userConsents` array replacement. The typed merge pipeline
  is exercised end-to-end by
  `crates/trading/tests/app_data_merge_contract.rs`.
- Cost: one public helper pair on `cow-sdk-trading`
  (`merge_and_seal_app_data` and `params_from_doc`), one
  extension to the private `merge_app_data_params` helper for the
  hooks replacement rule, and the retirement of the opaque-merge
  helper. The submission seam gains one typed destructure
  (`let (info, merged_params) = ...`) and reads the signer from
  the typed field rather than the override object.

## Alternatives Rejected

- Patch the opaque-merge helper narrowly to lift `signer` and
  `flashloan` from the override before the `metadata` merge: fixes
  the signer and flashloan drop but leaves the submission-seam
  validator reading from the override object and leaves the
  `metadata.hooks` drift in place. The typed pipeline closes all
  three at once.
- Keep both the typed and the opaque helpers and let callers
  choose: permanently splits the merge idiom across two code
  paths and re-opens drift surface on every future typed-field
  addition. The opaque path would accumulate the same class of
  defect again.
- Import the upstream TypeScript `deepmerge`-based helper verbatim
  into Rust: brings a TypeScript-only library semantic for array
  concatenation into the Rust surface and forces the pipeline to
  carry an explicit `metadata.userConsents = []` pre-clear that
  Rust's object-aware fall-through already handles correctly.
- Hide the typed pipeline behind a private free function only:
  workable today but closes the reviewable extension point
  consumers already rely on for composition through
  `cow_sdk::trading`.

## Links

- [Architecture](../architecture.md)
- [Parity Matrix](../parity.md)
- [Verification Guide](../verification.md)
- [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md)
- [ADR 0011](0011-typed-amount-boundary-and-typestate-ready-state-construction.md)
- [ADR 0015](0015-client-side-order-bounds-validator.md)

**Proven by:**

- [Trading App-Data Merge Audit](../audit/trading-app-data-merge-audit.md)
