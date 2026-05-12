# WASM Callback Shape Design Audit

Status: Current
Last reviewed: 2026-05-11
Owning surface: `cow-sdk-wasm` typed JavaScript wallet, signer, EIP-1271, cancellation, and HTTP callback boundary
Refresh trigger: Changes to callback type declarations, callback registry ownership, signing callback payloads, cancellation signing, wallet timeout handling, or callback error mapping
Related docs:
- [ADR 0040](../adr/0040-wallet-provider-callback-boundary-for-js-consumers.md)
- [ADR 0043](../adr/0043-callback-registry-internalization.md)
- [ADR 0045](../adr/0045-async-signer-trait-narrowing.md)
- [PROPERTIES.md](../../PROPERTIES.md)

## Scope

This audit covers:

- named TypeScript callback shapes for typed-data, EIP-1193, digest,
  cancellation, custom EIP-1271, and HTTP fetch flows
- internal callback registry ownership by client constructors and facade
  instances
- timeout and abort option propagation into callback-owned work
- typed failure mapping for throws, rejects, malformed outputs, aborts, and
  unsupported capability combinations

It does not cover wallet-vendor UI behavior or live on-chain signature
verification.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Named callbacks | Public declarations expose named callback types rather than raw provider objects | Conforms |
| Registry internalization | Callback registry handles are absent from public facade declarations | Conforms |
| Lifetime retention | Client-owned callbacks survive in-flight requests and are released through owned disposal | Conforms |
| Failure mapping | Callback throws, rejects, malformed outputs, timeout overflow, and aborts map to typed errors | Conforms |
| Capability split | Signing and cancellation functions request only the callback capability they need | Conforms |

## Current Contract

### Callback Shapes

The package exposes typed callbacks for wallet and runtime responsibilities:
`TypedDataSignerCallback`, `Eip1193RequestCallback`,
`DigestSignerCallback`, `CustomEip1271Callback`, and `CowFetchCallback`.
Each callback receives a typed payload or request DTO and may return either a
plain value, a Promise, or a thenable.

### Registry Ownership

Callback registry state is implementation-owned. Public TypeScript declarations
do not expose registry classes, registry ids, or handle constructors. Facade
classes retain callbacks for the lifetime of the owning client and dispose the
underlying resources when the client is disposed.

### Timeout And Abort Semantics

Per-call options carry `signal` and `timeoutMs`. Signing options also carry
`walletConfig.timeoutMs` for wallet-owned operations. HTTP callback requests
receive a live `AbortSignal`; abort and timeout paths clean up listeners and
timer handles.

## Evidence

Primary implementation points:

- `crates/wasm/src/exports/callbacks.rs`
- `crates/wasm/src/exports/registry.rs`
- `crates/wasm/src/exports/transport.rs`
- `crates/wasm/src/exports/signing.rs`
- `crates/wasm/src/exports/cancel.rs`
- `crates/wasm/npm/src/callbacks.ts`
- `crates/wasm/npm/src/internal.ts`
- `crates/wasm/npm/src/options.ts`

Primary regression coverage:

- `crates/wasm/tests/wasm_snapshot_surface_contract.rs::generated_type_declarations_hide_callback_registry`
- `crates/wasm/tests/wasm_snapshot_surface_contract.rs::generated_type_declarations_name_callback_params`
- `crates/wasm/tests/wasm_snapshot_surface_contract.rs::generated_type_declarations_expose_abort_and_wallet_options`
- `crates/wasm/tests/wasm_callback_lifetime_contract.rs::client_owned_callback_survives_until_request_resolves`
- `crates/wasm/tests/wasm_callback_contract.rs::wallet_config_timeout_rejects_pending_signer_callback`
- `crates/wasm/tests/wasm_callback_contract.rs::typed_cancellation_signer_returns_order_uids`
- `crates/wasm/tests/wasm_callback_contract.rs::eip1193_cancellation_callback_shape_is_stable`
- `crates/wasm/tests/wasm_cancellation_contract.rs::abort_bridge_removes_listener_after_success`
- `crates/wasm/tests/wasm_cancellation_contract.rs::abort_bridge_removes_listener_after_callback_throw`
- `crates/wasm/tests/wasm_cancellation_contract.rs::abort_bridge_removes_listener_after_callback_reject`
- `crates/wasm/tests/wasm_cancellation_contract.rs::abort_bridge_removes_listener_after_parse_error`
- `crates/wasm/tests/wasm_cancellation_contract.rs::abort_bridge_removes_listener_after_timeout_overflow`

Validation surface:

```text
cargo test -p cow-sdk-wasm --test wasm_snapshot_surface_contract
wasm-pack test crates/wasm --headless --chrome
pnpm --dir crates/wasm/npm test
```
