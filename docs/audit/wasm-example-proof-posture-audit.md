# WASM Example Proof-Posture Audit

Status: Current  
Last reviewed: 2026-04-30
Owning surface: WASM verification consoles and their two-tier proof posture  
Refresh trigger: Any change to the console proof lanes, the mock-versus-injected separation, the staging-versus-proxy posture on static pages, or the shipped deterministic and environment-sensitive evidence set  
Related docs:
- [ADR 0007](../adr/0007-bounded-browser-wallet-support-and-current-browser-runtime-contract.md)
- [ADR 0009](../adr/0009-wasm-verification-consoles-hybrid-extensibility-and-two-tier-proof.md)
- [Browser-Runtime Proof Posture](../browser-runtime-proof-posture.md)
- [Browser Wallet Chain Coherence Audit](browser-wallet-chain-coherence-audit.md)
- [Architecture](../architecture.md)
- [Examples](../examples.md)

## Scope

This audit covers:

- the deterministic mock-wallet proof surface of the browser-wallet console
- the deterministic capability, app-data, CID, order-envelope, EIP-1271,
  approval, trading-default, and manual-network-panel proof surface of the
  sdk-verification console
- the environment-sensitive injected-wallet and static browser-live surfaces
  that run alongside the deterministic lane
- the Playwright fixture lanes that hold the route-mocked CoW orderbook,
  subgraph, and EIP-1193 contracts for both consoles

It does not cover vendor-specific wallet extension behavior, live production
orderbook responses, or bridging or composable-order capabilities that are
deferred to later planning revisions.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Deterministic console proof | Host-side `cargo test`, in-browser `wasm-bindgen-test`, and route-mocked Playwright cover the mock-wallet and capability-verification surfaces on every commit | Conforms |
| WASM compatibility workflow | `.github/workflows/wasm.yml` runs `wasm-pack test --headless --chrome` for `crates/browser-wallet`, `crates/transport-wasm`, `crates/trading`, `crates/signing`, and both WASM consoles | Conforms |
| Mock versus injected separation | The browser-wallet console keeps the mock pane as the deterministic contract and treats the injected pane as environment-sensitive; the UI, README, and test lanes mirror the split | Conforms |
| Staging-by-default posture | Static browser-live orderbook actions default to `staging` and production live actions stay disabled on the shipped static page | Conforms |
| Proof-posture discoverability | A public `docs/browser-runtime-proof-posture.md` describes the two tiers in finished-product language and cross-links to ADR 0009, ADR 0007, and the console READMEs | Conforms |

## Current Contract

### Deterministic Lane

Host-side `cargo test` drives the Rust-native state machines inside both
consoles. The browser-wallet console exercises multi-wallet selection,
confirmation, reconnect, reset, and forget semantics under
`MockEip1193Transport`. The sdk-verification console exercises capability,
app-data, CID, order-envelope, EIP-1271, approval, and trading-default
outputs through property-style wasm-bindgen tests.

In-browser `wasm-bindgen-test` runs both consoles through a real headless
Chrome so the WebAssembly boundary and the `wasm-bindgen` interop idioms
receive continuous proof at every commit.

The WASM compatibility workflow also runs the transport leaf directly through
headless Chrome with its tracing feature enabled. That lane exercises the
fetch-backed adapter's browser dispatch shape, redacted endpoint telemetry,
native-versus-browser error-class parity, and a minimal redaction smoke for
the fetch transport configuration. The browser-wallet console receives its own
headless `wasm-pack test` invocation, including a smoke assertion that the
EIP-1193 trust wrapper rejects anonymous providers and accepts an explicitly
reviewed origin before the console consumes the provider.

Playwright with route-mocked fixtures covers full end-to-end DOM flows. The
`e2e/browser-wallet/fixtures/injected-wallet.ts` fixture supplies EIP-6963
discovery, chain-switch events, and provider rejection shapes. The
`e2e/sdk-verification/fixtures/cow-api.ts` fixture supplies deterministic
payloads for version, quote, solver-competition latest, orders by uid,
trades, app-data, and subgraph queries.

### Environment-Sensitive Lane

Manual QA against real wallet extensions and optional static browser-live
smoke cover the vendor-specific behaviors that cannot be asserted
deterministically. These lanes are explicitly gated in the console UI, the
READMEs, and the proof-posture document so reviewers can distinguish a
deterministic contract failure from an environment-sensitive observation.

### Static Page Posture

Static browser-live orderbook actions default to `staging` on both shipped
consoles. Production browser-live orderbook actions are disabled on the
shipped static page and require a proxy-enabled deployment to surface the
permitted CORS headers.

## Evidence

Primary implementation points:

- `.github/workflows/wasm.yml`
- `crates/transport-wasm/src/fetch.rs`
- `examples/wasm/browser-wallet-console/src/lib.rs`
- `examples/wasm/browser-wallet-console/index.html`
- `examples/wasm/sdk-verification-console/src/lib.rs`
- `examples/wasm/sdk-verification-console/index.html`

Primary regression coverage:

- `crates/transport-wasm/tests/wasm.rs`
- `crates/transport-wasm/tests/wasm/fetch_smoke.rs::fetch_transport_config_debug_redacts_base_url`
- `crates/transport-wasm/tests/fetch_contract.rs::fetch_dispatch_span_matches_transport_shape`
- `crates/transport-wasm/tests/parity_contract.rs`
- `examples/wasm/browser-wallet-console/tests/selection_confirmation_contract.rs`
- `examples/wasm/browser-wallet-console/tests/selection_reconnect_contract.rs`
- `examples/wasm/browser-wallet-console/tests/session_actions_contract.rs`
- `examples/wasm/browser-wallet-console/src/lib.rs::tests::mock_order_lookup_payload_matches_order_wire_shape`
- `examples/wasm/browser-wallet-console/tests/wasm_deterministic.rs::eip1193_trust_wrapper_rejects_anonymous_provider_and_accepts_explicit_origin`
- `examples/wasm/sdk-verification-console/tests/deterministic_exports.rs`
- `e2e/browser-wallet/tests/browser-wallet-console.spec.ts`
- `e2e/browser-wallet/tests/injected-chain-coherence.spec.ts`
- `e2e/sdk-verification/tests/sdk-verification-console.spec.ts`
- `e2e/sdk-verification/tests/live-orderbook-readiness.spec.ts`
- `e2e/sdk-verification/tests/manual-network-panels.spec.ts`

Validation surface:

```text
cargo fmt --all --check
cargo test --manifest-path examples/wasm/browser-wallet-console/Cargo.toml
cargo test --manifest-path examples/wasm/sdk-verification-console/Cargo.toml
cd crates/transport-wasm && wasm-pack test --headless --chrome --features tracing
cd examples/wasm/browser-wallet-console && wasm-pack test --headless --chrome
cd examples/wasm/sdk-verification-console && wasm-pack test --headless --chrome
bun run --cwd e2e/browser-wallet test
bun run --cwd e2e/sdk-verification test
```
