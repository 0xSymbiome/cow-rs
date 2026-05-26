# Shared Logic Reviewability Audit

Status: Current  
Last reviewed: 2026-05-26
Owning surface: Orderbook, signing, and trading shared-logic reviewability boundary, plus the canonical primitive-layer invocation paths shared across the cow-rs workspace  
Refresh trigger: Changes to shared orderbook request execution, signing payload construction, thin posting wrappers, boundary-specific order DTO separation, or the canonical primitive-layer invocation paths (keccak256, U256 and quantity parsing, address encoding, hex serde, typed-primitive bridges, and identity-wire-form preservation) that materially affect correctness or reviewability  
Related docs:
- [ADR 0005](../adr/0005-boundary-specific-runtime-contracts-and-strong-domain-types.md)
- [ADR 0052](../adr/0052-alloy-primitives-canonical-primitive-layer.md)
- [Architecture](../architecture.md)
- [Verification Guide](../verification-guide.md)

## Scope

This audit covers:

- orderbook request construction and execution
- retry, status mapping, headers, rate-limit handling, and JSON, text, or
  empty responses
- order signing and cancellation signing payload preparation
- trading posting wrapper paths
- canonical primitive-layer invocation across the cow-rs workspace
  (keccak256, U256 and quantity parsing, address encoding, hex serde,
  typed-primitive bridges, and identity-wire-form preservation)
- generated or schema-derived artifacts as a separate category

It does not cover style-only cleanup notes, generic refactor wishlists, or unrelated
internal refactors that do not affect correctness or reviewability.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Shared HTTP request construction | Use one shared orderbook request path | Conforms |
| Shared retry, status, and rate-limit execution | Use one shared executor for JSON, text, and empty responses | Conforms |
| Shared signing payload preparation | Share payload construction between sync and async signing paths | Conforms |
| Thin trading posting wrappers | Keep ergonomic entry points thin and route workflow logic through the async implementation path | Conforms |
| Boundary-specific order DTO separation | Retain distinct DTOs only where ABI, API, normalized, or user-domain boundaries differ materially | Conforms |
| Canonical primitive-layer invocation | Use one canonical entry point per shared primitive across the workspace, with cow-owned `#[repr(transparent)]` newtypes over `alloy_primitives` per ADR 0052 | Current |

## Current Contract

### Orderbook Request Execution

Orderbook request execution is shared through internal helpers in
`crates/orderbook/src/request.rs`, including `request_with`, `send_request`,
`request_headers`, and `execute_with`.

### Signing Payload Preparation

Signing keeps separate sync and async entry points while sharing payload
construction through:

- `crates/signing/src/order_signing.rs::order_signing_payload`
- `crates/signing/src/cancellation.rs::cancellation_signing_payload`

### Thin Trading Posting Wrappers

Trading keeps ergonomic public entry points while routing workflow logic
through async implementation paths. Shared advanced-parameter extraction lives
in:

- `crates/trading/src/post/generic.rs::swap_additional_params`
- `crates/trading/src/post/generic.rs::limit_additional_params`

### Boundary-Specific Order DTO Separation

Order-like DTO separation is retained only where the boundary is materially
different:

- `cow_sdk_core::UnsignedOrder`
- `cow_sdk_contracts::Order`
- `cow_sdk_contracts::NormalizedOrder`
- `cow_sdk_orderbook::QuoteData`
- `cow_sdk_orderbook::OrderCreation`
- `cow_sdk_orderbook::Order`

Generated or schema-derived artifacts remain internal or test-only and are not
part of the public SDK API.

### Canonical Primitive Layer Invocation

Every shared primitive in production code routes through one canonical
entry point. The contract applies to every cow-rs crate that consumes
the primitive; parallel implementations are a reviewability hazard
because each variant must be re-verified independently and any drift
between variants is invisible to a reviewer who only reads one site.

- **keccak256**: production code across `cow-sdk-contracts`,
  `cow-sdk-signing`, and `cow-sdk-cow-shed` invokes
  `alloy_primitives::keccak256` directly. Hand-rolled
  `sha3::Keccak256` helpers remain only inside `crates/*/tests/` (and
  one inline test-mod helper in
  `crates/contracts/src/deploy.rs::tests`) so the parity assertions
  compare the crate output against an independent keccak
  implementation. Each retained test helper carries a `// SAFETY:`
  comment naming its independent-oracle purpose.
- **SigningScheme**: the cow-protocol-side
  `cow_sdk_contracts::SigningScheme` (repr-u8) and the wire-side
  `cow_sdk_orderbook::SigningScheme` (`serde(rename_all =
  "lowercase")`) carry distinct wire formats and so remain separate
  types. A typed `From` / `TryFrom` bridge in
  `crates/orderbook/src/types/enums.rs` is the canonical conversion
  surface, and a per-variant parity test prevents drift if any
  upstream variant is added or renamed.
- **Identity wire-form preservation**: the cow-named identity types
  (`Address`, `Hash32`, `AppDataHash`, `HexData`, `OrderUid`) and the
  cow-named numeric types (`Amount`, `SignedAmount`) resolve to
  cow-owned `#[repr(transparent)]` newtypes over the corresponding
  `alloy_primitives` type per
  [ADR 0052](../adr/0052-alloy-primitives-canonical-primitive-layer.md).
  Each newtype's `Display` and serde impls emit the canonical wire
  form (lowercase 0x-prefixed hex for byte-typed identities;
  strict-decimal for `Amount` and `SignedAmount`); the cow inherent
  `to_hex_string()` returns the same form. The wire-format preservation
  invariant is pinned by
  `crates/core/tests/wire_format_preservation_contract.rs` and the
  parity fixtures under `parity/fixtures/`, so changes to the
  underlying primitive backing cannot silently drift the wire
  format. `Address` carries a cow-owned `Display` impl because
  `alloy_primitives::Address::Display` defaults to EIP-55 checksum
  casing and the cow wire form is lowercase; `Amount` and
  `SignedAmount` carry cow-owned `Serialize`/`Deserialize` impls
  because alloy's `U256::Serialize` emits hex and alloy's
  `ruint::Uint::FromStr` accepts four radices, both of which would
  silently relax the cow strict-decimal-only fail-closed contract.

`TypedDataDomain` remains a cow-owned struct (preserved as-is from
the current working tree); the cow struct owns its
`Serialize`/`Deserialize` impls and emits the canonical EIP-1193
`eth_signTypedData_v4` wire shape directly (numeric `chainId`,
required `verifyingContract`, no `salt`), so no bridge-side JSON
coercion is required. The
`crates/alloy-signer/src/conversion.rs` cow-to-alloy
`TypedDataDomain` adapter remains in place as a thin
`into_alloy_domain()` accessor (≈30 LoC); it simplifies but does not
retire, because the cow struct deliberately owns the wire shape and
is intentionally independent of the alloy `Eip712Domain` field
layout (which is `Option<_>` for every field and uses `U256` for
`chainId`).

## Evidence

Primary regression coverage:

- `crates/orderbook/tests/request_contract.rs::request_json_retries_429_and_preserves_headers_on_each_attempt`
- `crates/orderbook/tests/request_contract.rs::request_text_and_empty_share_the_request_builder_and_success_path`
- `crates/orderbook/tests/request_contract.rs::rate_limiter_spaces_requests_after_token_budget_is_consumed`
- `crates/signing/tests/order_signing_contract.rs::async_sign_order_paths_match_sync_signing_behavior`
- `crates/signing/tests/cancellation_contract.rs::async_cancellation_signing_paths_match_sync_variants`
- `crates/contracts/tests/order_contract.rs::unsigned_order_conversion_makes_user_domain_and_contract_boundaries_explicit`
- `crates/orderbook/tests/types_contract.rs::order_creation_from_quote_keeps_quote_shape_and_quote_id`
- `crates/orderbook/tests/signing_scheme_bridge_contract.rs`
- `crates/core/tests/wire_format_preservation_contract.rs`

Validation surface:

```text
cargo fmt --all --check
cargo test -p cow-sdk-orderbook
cargo test -p cow-sdk-signing
cargo test -p cow-sdk-trading
cargo test --workspace
cargo clippy -p cow-sdk-orderbook --all-targets --all-features -- -D warnings
cargo clippy -p cow-sdk-signing --all-targets --all-features -- -D warnings
cargo clippy -p cow-sdk-trading --all-targets --all-features -- -D warnings
```
