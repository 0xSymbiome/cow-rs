# WASM EIP-1271 Parity Audit

Status: Current
Last reviewed: 2026-05-09
Owning surface: `cow-sdk-wasm` EIP-1271 payload helpers and smart-account signing callbacks
Refresh trigger: Changes to EIP-1271 payload construction, smart-account callback shapes, UID/digest string handling, signature normalization, or upstream parity fixtures
Related docs:
- [ADR 0039](../adr/0039-typescript-callable-wasm-sdk-surface.md)
- [ADR 0040](../adr/0040-wallet-provider-callback-boundary-for-js-consumers.md)
- [EIP-1271 Verification Cache Audit](eip1271-verification-cache-audit.md)
- [PROPERTIES.md](../../PROPERTIES.md)

## Scope

This audit covers:

- `cow-sdk-wasm` EIP-1271 signature payload construction
- `signOrderWithEip1271` and `signOrderWithCustomEip1271`
- UID and digest string propagation into cross-ABI DTOs
- parity against native Rust signing helpers and the upstream TypeScript SDK
  vector

It does not cover on-chain `isValidSignature` execution, cache behavior, or
wallet-vendor UI flows.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Native Rust parity | The wasm EIP-1271 payload equals `cow_sdk_signing::eip1271_signature_payload` for the fixed vector | Conforms |
| TypeScript SDK parity | The fixed vector matches upstream `OrderSigningUtils.getEip1271Signature` output | Conforms |
| Facade-resolves-callback | JavaScript supplies the final signature, while Rust stores only a pure resolved provider | Conforms |
| UID and digest strings | Cross-ABI DTOs reuse canonical `as_str()` output instead of re-encoding bytes | Conforms |
| Signature validation | Malformed ECDSA signatures fail before being surfaced as signed orders | Conforms |

## Current Contract

### Payload Parity

`eip1271SignaturePayload` and the EIP-1271 order-signing functions wrap the
same Rust helper used by native signing. The wasm tests compare the output
against native Rust and against a recorded upstream TypeScript SDK vector for
the same order, owner, verifier, and signature bytes.

### Smart-Account Callback Boundary

`signOrderWithCustomEip1271` invokes a JavaScript callback at the facade
boundary. The callback returns the final ABI-encoded signature, and Rust wraps
that resolved string in a `Send + Sync` provider implementation. No JavaScript
function handle or `JsValue` is stored in the provider trait object.

### String Ownership

Order IDs and digests crossing to TypeScript use the canonical string stored by
the Rust type. The wasm boundary never reconstructs those fields from raw byte
arrays.

## Evidence

Primary implementation points:

- `crates/wasm/src/pure/eip1271.rs`
- `crates/wasm/src/exports/eip1271.rs`
- `crates/wasm/src/exports/signing.rs`
- `crates/wasm/tests/fixtures/eip1271_upstream_vector.json`
- `parity/source-lock.yaml`

Primary regression coverage:

- `crates/wasm/tests/host_pure_helpers.rs::eip1271_payload_matches_signing_module_output_and_vector`
- `crates/wasm/tests/host_pure_helpers.rs::generated_order_uid_uses_canonical_strings`
- `crates/wasm/tests/wasm_eip1271_contract.rs::eip1271_payload_matches_native_rust`
- `crates/wasm/tests/wasm_eip1271_contract.rs::eip1271_payload_matches_recorded_typescript_sdk_vector`
- `crates/wasm/tests/wasm_eip1271_contract.rs::sign_order_with_eip1271_uid_equals_generated_order_id_as_str`
- `crates/wasm/tests/wasm_eip1271_contract.rs::custom_eip1271_callback_signature_is_used_verbatim`
- `crates/wasm/tests/wasm_eip1271_contract.rs::resolved_eip1271_provider_is_send_sync_without_jsvalue`
- `e2e/wasm-typescript/tests/eip1271.spec.ts`

Validation surface:

```text
cargo test -p cow-sdk-wasm --test host_pure_helpers
wasm-pack test crates/wasm --headless --chrome
pnpm --dir e2e/wasm-typescript test
```

