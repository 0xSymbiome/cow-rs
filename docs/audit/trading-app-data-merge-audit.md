# Trading App-Data Merge Audit

Status: Current
Last reviewed: 2026-05-13
Owning surface: `cow-sdk-trading` quote-to-post app-data edit path,
including the public `merge_and_seal_app_data` and
`params_from_doc` helpers, the private typed merge with its
hooks-replacement rule, and the submission-seam
`app_data_signer` derivation consumed by
`OrderBoundsValidator::validate`.
Refresh trigger: Changes to the `merge_and_seal_app_data` or
`params_from_doc` public signatures; changes to the
hooks-replacement rule on `metadata.hooks`; any change that
reintroduces a `serde_json::Value`-taking merge helper on the
quote-to-post path; changes to
`AppDataParams::metadata_wire_value` that move where typed
`signer` or `flashloan` values land on the wire; changes to the
submission-seam derivation of `app_data_signer` that read from
the override object rather than from the typed merged params.
Related docs:
- [ADR 0018](../adr/0018-typed-app-data-merge.md)
- [ADR 0015](../adr/0015-client-side-order-bounds-validator.md)
- [Architecture](../architecture.md)
- [Parity Matrix](../parity.md)

## Scope

This audit covers:

- the public typed merge helper
  `cow_sdk_trading::merge_and_seal_app_data(base_doc,
  override_params) -> Result<(TradingAppDataInfo, AppDataParams),
  TradingError>`
- the public re-parse helper
  `cow_sdk_trading::params_from_doc(base_doc) ->
  Result<AppDataParams, TradingError>`
- the private typed merge
  `cow_sdk_trading::app_data::merge_app_data_params(base, override) ->
  AppDataParams` and its hooks-replacement rule on
  `metadata.hooks`
- the `AppDataParams::metadata_wire_value` lift that puts typed
  `signer` and `flashloan` sub-fields on the wire under the
  reviewed `metadata.signer` and `metadata.flashloan` positions
- partner-fee metadata supplied through quote advanced settings and
  preserved through the quote-to-post merge
- the re-emission pipeline through
  `cow_sdk_app_data::generate_app_data_doc` and
  `cow_sdk_app_data::app_data_info` that produces the final
  `TradingAppDataInfo`
- the submission-seam derivation of `app_data_signer` consumed by
  `OrderBoundsValidator::validate` on the quote-to-post path at
  `crates/trading/src/post/generic.rs`

It does not cover the direct-build app-data path used by
`cow_sdk_trading::build_app_data` (covered by the parity fixture
at `parity/fixtures/trading.json`), the IPFS read and write
preflight (covered by the app-data preflight contract), or the
orderbook submission path (covered by the trading order-bounds
validator audit).

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Merge pipeline | `merge_and_seal_app_data` is the only canonical quote-to-post merge helper and returns `(TradingAppDataInfo, AppDataParams)` so callers read the signer from the typed merged value | Conforms |
| Typed field survival | Override-supplied `signer: Option<Address>` and `flashloan: Option<FlashloanHints>` survive end-to-end into `metadata.signer` and `metadata.flashloan` on the wire | Conforms |
| Hooks replacement | When the override's metadata contains `"hooks"`, the base's `metadata.hooks` is dropped before recursive metadata merge so the override hook set fully replaces the base | Conforms |
| User-consents replacement | Override-supplied `metadata.userConsents` arrays replace the base array through the object-aware fall-through in the typed merge | Conforms |
| Partner-fee metadata | `PartnerFee` metadata supplied through advanced app-data settings survives the merge into the posted app-data document | Conforms |
| Signer derivation | The submission-seam `app_data_signer` reads `merged_params.signer.clone()` from the typed merged value; no override-only read remains on the quote-to-post path | Conforms |
| Round-trip idempotency | `params_from_doc(generate_app_data_doc(p))` equals `p` for every `AppDataParams` value constructed through the public surface | Conforms |
| Runtime-neutral validation | `cow_sdk_app_data::validate_app_data_doc` validates the same `AppDataDoc` wire shape used by browser-facing bindings, so the WASM boundary does not introduce a trading merge rewrite | Conforms |

## Current Contract

### Merge Pipeline

`merge_and_seal_app_data` parses the base wire document into
`AppDataParams` through the `Deserialize` impl that lifts
`metadata.signer` and `metadata.flashloan` out of the wire shape
and into the typed sub-fields. The helper then calls
`merge_app_data_params(&base_params, override_params)` to produce
the merged typed value, re-emits the wire document through
`generate_app_data_doc(merged.clone())`, and derives the canonical
digest via `app_data_info(doc.clone())`. The return tuple
carries both the `TradingAppDataInfo` and the merged
`AppDataParams` so the submission seam reads the signer from the
same typed value that produced the uploaded document. The opaque
`serde_json::Value`-taking merge helper previously exported from
the crate is deleted; no public merge helper reaches into the
wire shape directly.

### Typed Field Survival

`AppDataParams` carries typed `signer: Option<Address>` and
`flashloan: Option<FlashloanHints>` sub-fields alongside the
open-ended `metadata` map. `metadata_wire_value` is the only
translation point that lifts those typed values onto the wire
under `metadata.signer` and `metadata.flashloan`. Because the
merge pipeline runs entirely over typed `AppDataParams`, the lift
happens once inside `generate_app_data_doc` and both fields
survive every override merge that carries them.

### Hooks Replacement

`merge_app_data_params` inspects the override's open-ended
`metadata` map for a `"hooks"` key. When present, the base's
`metadata.hooks` entry is removed before the recursive merge
recurses into `metadata`, so the override hook set fully replaces
the base hook set rather than layering over it. The rule is
narrow and applies only to `metadata.hooks`; every other sibling
metadata key follows the default recursive-merge plus
override-wins behavior.

### User-Consents Replacement

`metadata.userConsents` on the wire is a JSON array. The typed
merge's `deep_merge_values` helper falls through to the override
value for any pair of values whose runtime shapes are not both
JSON objects, so an override-supplied array replaces the base
array rather than concatenating with it. The same rule applies
to every other array-valued metadata sibling.

### Partner-Fee Metadata

Partner-fee policy metadata supplied through
`advanced_settings.app_data` remains part of the override
`AppDataParams` and is emitted through the same typed merge path as
other metadata siblings. The quote-to-post regression pins the
resulting posted app-data document so later merge changes cannot
drop the partner-fee policy while preserving the surrounding order.

### Signer Derivation

The quote-to-post submission seam reads
`app_data_signer = merged_params.signer.clone()` from the typed
merged value returned by `merge_and_seal_app_data`. No read on
`advanced_settings.app_data.signer` remains on the path. When
the override is absent, the submission seam re-parses the base
document through `params_from_doc` so the signer derivation still
reflects the actually-uploaded document.

### Round-Trip Idempotency

`params_from_doc(generate_app_data_doc(p))` equals `p` for every
`AppDataParams` value constructed through the public surface. The
regression suite pins this invariant explicitly so a later change
to the wire serializer or the `Deserialize` lift cannot silently
drift the merge pipeline.

### Runtime-Neutral Validation

`cow_sdk_app_data::validate_app_data_doc` validates the same `AppDataDoc` wire
document shape that trading emits. Browser-facing bindings can pass that shape
through unchanged, keeping validation ownership in `cow-sdk-app-data` without
introducing another merge or rewrite step on the quote-to-post path.

## Evidence

Primary implementation points:

- `crates/trading/src/app_data.rs`
- `crates/trading/src/post/generic.rs`
- `crates/trading/src/lib.rs`
- `crates/app-data/src/types/partner_fee.rs`
- `crates/app-data/src/schema.rs`
- `crates/sdk/src/prelude.rs`

Primary regression coverage:

- `crates/trading/tests/app_data_merge_contract.rs`
- `crates/trading/tests/app_data_merge_contract.rs::merge_preserves_override_signer_byte_identical`
- `crates/trading/tests/app_data_merge_contract.rs::merge_replaces_hooks_per_adr_0018`
- `crates/trading/tests/app_data_merge_contract.rs::merge_lifts_flashloan_metadata_through_quote_to_post`
- `crates/trading/tests/app_data_merge_contract.rs::partner_fee_in_advanced_settings_appdata_merges_through_to_post`
- `crates/trading/tests/post_contract.rs`
- `crates/trading/tests/parity_contract.rs`
- `parity/fixtures/trading.json`

Validation surface:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p cow-sdk-trading --test app_data_merge_contract
cargo test -p cow-sdk-trading --test post_contract
cargo test -p cow-sdk-trading --test parity_contract
cargo check --workspace --all-features --target wasm32-unknown-unknown
```
