# Panic-Free Public Surface Audit

Status: Current
Last reviewed: 2026-06-20
Owning surface: every `crates/*/src/**/*.rs` file accessible from the published public API
Refresh trigger: any ADR 0033 panic-policy change, panic-allowlist addition, or new `expect`, `unwrap`, or `panic!` site on a path reachable from the published public API
Related docs:
- [ADR 0033](../adr/0033-minimum-viable-panic-surface.md)
- [Alloy Adapters Audit](alloy-adapters-audit.md)

## Scope

This audit covers:

- every public-API runtime path under `crates/*/src/**/*.rs`
- `expect`, `unwrap`, and `panic!` sites reachable from published crates
- the rationale for every remaining production panic site

It does not cover `#[cfg(test)]` modules, rustdoc examples, integration tests,
benchmarks, or private review tooling. Those surfaces may use `unwrap` or
`expect` for concise test setup without widening the runtime public API.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Public runtime surface | No unchecked panic site remains without an explicit rationale | Conforms |
| Static literals and owned-value serialization | Remaining `expect` sites assert crate-owned constants, validated deployment-registry lookups, or serialization of SDK-owned typed values | Conforms |
| Typestate builders | Builder terminals read each required input from a data-carrying marker and return typed errors; no `expect` site remains on the construction path | Conforms |
| Numeric clamps | Conversion `expect` sites follow saturating or bounded arithmetic immediately before the conversion | Conforms |
| Amount decimal I/O | `Amount::parse_units` and `Amount::from_units` are `Result`-returning constructors carrying no `unwrap`, `expect`, or `panic!` (`parse_units` pre-guards the alloy `parse_units` footguns; `from_units` does checked integer scaling and rejects an over-`uint256` product with `NumericOverflow`); `Amount::format_units` is infallible and clamps `decimals > 77` to `Unit::MAX` rather than panicking | Conforms |
| Panic allowlist | `.github/config/panic-allowlist.yaml` carries 29 reviewed item-path entries, each covering an accepted static-invariant panic-bearing call enumerated below | Conforms |
| Native Alloy adapters | Provider, signer, and umbrella public methods return typed errors for validation, transport, signing, pending transaction, and unsupported capability failures rather than panicking | Conforms |
| Trading wait helper | `WaitOptions` constructors/builders, `submit_and_wait_for_receipt`, `poll_for_receipt`, and `WaitError` formatting/error implementations return typed results and retain only a clamped wasm timer conversion behind the allowlist | Conforms |
| Transport classification growth | `TransportErrorClass::Upgrade` is an additive non-exhaustive enum variant and introduces no new panic path | Conforms |
| Item-level panic artifacts | Each documented allowlist entry requires a rationale, `# Panics` rustdoc on the named item, and a `// SAFETY:` comment in the item body | Conforms |
| WASM exports | Every fallible `#[wasm_bindgen]` export returns `Result<T, JsValue>`; infallible exports (e.g. `supportedChainIds`, `wasmVersion`, `__cow_sdk_wasm_init`) return plain values, and `__cow_sdk_wasm_init` initializes `console_error_panic_hook` exactly once | Conforms |
| Pure WASM helpers | The `cow-sdk-wasm::helpers` module exposes fallible helper APIs through typed errors rather than JavaScript or panic boundaries | Conforms |

Documented public runtime sites:

| Sites | Rationale |
| --- | --- |
| `crates/app-data/src/schema.rs` (`generate_app_data_doc`); `crates/app-data/src/types/partner_fee.rs` (`PartnerFee::to_value`) | Typed app-data and partner-fee structures serialize through owned `serde` implementations; failure would mean the shipped Rust types stopped being JSON-serializable. |
| `crates/signing/src/domain.rs` (`domain`); `crates/trading/src/allowance.rs` (`resolve_vault_relayer`); `crates/trading/src/onchain.rs` (`resolve_settlement_address`, `resolve_eth_flow_address`); `crates/trading/src/order.rs` (`calculate_unique_order_id`) | Canonical deployment lookups resolve through the const address table; the committed address literals are validated at construction and pinned by the `deployment_addresses_resolve_to_canonical_singletons` regression. |
| `crates/contracts/src/deployments.rs` (`DeploymentChainId::from`, `DeploymentEnv::from`); `crates/contracts/src/primitives.rs` (`sell_balance_name`, `buy_balance_name`) | In-crate enum bridges are exhaustive over the currently supported chains, environments, and settlement balance flags; a new upstream variant must land in the same patch as its bridge arm, with bridge parity regressions preventing drift. |
| `crates/core/src/redaction/body.rs` (`strip_credential_tokens`) | Redaction offsets are computed from in-bounds string match indices, so the slicing cannot leave the buffer. |
| `crates/core/src/transport/policy/config.rs` (`TransportPolicy::default_orderbook`, `default_subgraph`, `default_trading`, `default_ipfs`, `TransportPolicyBuilder::new`) | Default user-agent literals are header-safe crate constants used to build shared transport policies. |
| `crates/core/src/transport/policy/jitter.rs` (`bounded_offset`); `crates/core/src/transport/policy/retry.rs` (`RetryPolicy::base_backoff_delay`); `crates/core/src/transport/policy/time.rs` (`sleep`); `crates/trading/src/wait.rs` (`delay_for`); `crates/trading/src/order.rs` (`order_to_sign`); `crates/trading/src/onchain.rs` (`default_gas_limit`) | Values are clamped or statically bounded immediately before conversion, so the fallible conversion documents the invariant rather than accepting caller-controlled overflow. |
| `crates/trading/src/types/params.rs` (`LimitTradeParamsFromQuote::quote_id`); `crates/trading/src/client/swap.rs` (`SwapBuilder::to_trade_parameters`); `crates/trading/src/client/limit.rs` (`LimitBuilder::to_limit_parameters`) | Construction-guarded invariants: the newtype's only public constructor rejects an absent quote id on entry, and the builder terminals are reachable only from the fully-set typestate whose markers prove every required field was assigned. |
| `crates/orderbook/src/request.rs` (`ResponseEnvelope::json`) | The response-envelope test helper serializes an in-memory `serde_json::Value` fixture into bytes; the panic would require JSON value serialization itself to fail. |
| `crates/core/src/types/amount.rs` | No retained panic site. `Amount::parse_units` is the exact decimal-string constructor and returns `Result<Amount, CoreError>`: it rejects empty/whitespace and a leading `+`/`-` before delegating (closing the two alloy `parse_units` fail-open inputs), rejects `decimals > 77` with a typed `DecimalsOutOfRange`, and maps any remaining alloy parse failure to a typed `InvalidNumeric` — there is no `unwrap`, `expect`, or `panic!` on the path. The inverse `Amount::format_units` is infallible: it returns the bare integer for `decimals == 0` and otherwise resolves the unit through `Unit::new(decimals).unwrap_or(Unit::MAX)`, clamping `decimals > 77` to `77` so the alloy `format_units` call cannot reach its error arm. |
| `crates/contracts/src/order.rs` (`extract_order_uid_params`); `crates/contracts/src/signature.rs` (`decode_eip1271_signature_data`) | Length-checked slice-to-array conversions: each `try_into` is preceded by an early-return guard that proves the slice length matches the target array length (`ORDER_UID_LENGTH == 56` in `extract_order_uid_params`; `bytes.len() < 20` short-circuit in `decode_eip1271_signature_data`). The `expect` calls document the unreachability proof inline through `// SAFETY` comments naming the guarantee. |

## Current Contract

### No Undocumented Runtime Panic Sites

No `expect`, `unwrap`, or `panic!` site on a public runtime path ships without a
documented rationale and a refreshed entry in this audit. When a fallible
operation can fail because of caller input, the public contract returns a typed
error instead of panicking. The canonical panic allowlist is
`.github/config/panic-allowlist.yaml`, which carries 29 reviewed item-path
entries; each accepted production panic site is enumerated in the documented
public runtime sites table above and remains tied to a static invariant rather
than to caller-controlled input. `TransportErrorClass::Upgrade` is a reserved
classification slot on an already non-exhaustive enum and adds no panic-bearing
runtime path.

The `cargo check-panic-allowlist` gate enforces ADR 0033 at item level: each
documented entry must name an item whose rustdoc contains a `# Panics` section
and whose body contains a `// SAFETY:` comment explaining the local invariant.
The optional `documented: false` field is reserved for compile-time-only or
test-only items and still requires a rationale.

### Allowed Static-Invariant Sites

The remaining sites are limited to static literals, embedded assets, typestate
marker invariants, already-clamped numeric conversions, and serialization of
owned typed values — invariants the crate owns and tests, not untrusted caller
input.

### Test And Example Matches

Concise `unwrap` and `expect` calls in unit tests and rustdoc examples are not
part of the runtime public API and are excluded from the panic-free surface
claim.

## Evidence

Primary implementation points:

- `.github/config/panic-allowlist.yaml`
- `crates/*/src/**/*.rs`
- `crates/trading/src/wait.rs`
- `Cargo.toml` workspace clippy lint configuration
- `xtask/src/policy/check_panic_allowlist.rs`
- `crates/wasm/src/lib.rs`
- `crates/wasm/src/exports/mod.rs`
- `crates/wasm/src/exports/errors.rs`
- `crates/wasm/src/helpers/`

Primary regression coverage:

- `xtask/tests/check_panic_allowlist.rs::rejects_allowlisted_item_missing_panics_rustdoc`
- `xtask/tests/check_panic_allowlist.rs::rejects_allowlisted_item_missing_safety_comment`
- `xtask/tests/check_panic_allowlist.rs::accepts_item_with_both_artifacts`
- `xtask/tests/check_panic_allowlist.rs::accepts_item_with_documented_false_opt_out`
- `.github/workflows/_quality-gate.yml` clippy job with warnings denied, which
  enforces the workspace `missing_panics_doc` lint so every retained
  static-invariant panic site carries a `# Panics` rustdoc section
- public rustdoc `# Panics` sections on exposed functions that retain a
  static-invariant panic site
- `crates/wasm/tests/wasm_error_abi_contract.rs`
- `crates/wasm/tests/wasm_fail_closed_contract.rs`
- `crates/wasm/tests/host_pure_helpers.rs`

Validation surface:

```text
cargo check-panic-allowlist
cargo test --manifest-path xtask/Cargo.toml --test check_panic_allowlist
cargo clippy --workspace --all-targets --all-features -- -D warnings
```
