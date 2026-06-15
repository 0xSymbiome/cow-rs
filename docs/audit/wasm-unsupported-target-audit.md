# WASM Unsupported Target Audit

Status: Current
Last reviewed: 2026-05-06
Owning surface: native-only Alloy adapter crates and facade Alloy features on `wasm32`
Refresh trigger: Changes to `cow-sdk` Alloy feature gating, native Alloy adapter target guards, wasm workflows, or browser-wallet guidance
Related docs:
- [ADR 0035](../adr/0035-alloy-provider-adapter.md)
- [ADR 0036](../adr/0036-alloy-signer-adapter.md)
- [ADR 0037](../adr/0037-alloy-umbrella-adapter.md)
- [Browser Wallet Trust Posture Audit](browser-wallet-trust-posture-audit.md)

## Scope

This audit covers the explicit unsupported-target contract for:

- `cow-sdk-alloy-provider`
- `cow-sdk-alloy-signer`
- `cow-sdk-alloy`
- `cow-sdk` facade features `alloy-provider`, `alloy-signer`, and `alloy`

It does not cover browser-wallet runtime behavior beyond the documented
recommendation that wasm consumers use browser-wallet signing and
consumer-supplied EIP-1193 provider reads.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Native adapter crates | Each native Alloy adapter fails closed on `wasm32` targets with a compile-time diagnostic | Conforms |
| Facade features | Enabling any Alloy facade feature on `wasm32-unknown-unknown` fails with the documented native-only message | Conforms |
| Supported wasm path | Browser-wallet signing and consumer-supplied provider reads remain the documented browser runtime path | Conforms |
| CI coverage | CI asserts the three facade Alloy features fail on wasm and treats a successful wasm build as a failure | Conforms |

## Evidence

- `crates/alloy-provider/src/lib.rs`
- `crates/alloy-signer/src/lib.rs`
- `crates/alloy/src/lib.rs`
- `crates/sdk/src/lib.rs`
- `.github/workflows/ci.yml`
- `docs/providers/adapting-alloy.md`
- `docs/transport.md`

## Residual Risk

Future upstream Alloy releases may add browser-compatible provider components.
The current SDK contract still keeps these native adapter crates unsupported on
wasm until a separate browser-provider design is accepted and tested.

## Validation

```text
cargo check -p cow-sdk --target wasm32-unknown-unknown --features alloy
cargo check -p cow-sdk --target wasm32-unknown-unknown --features alloy-provider
cargo check -p cow-sdk --target wasm32-unknown-unknown --features alloy-signer
```
