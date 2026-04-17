# Browser Wallet Chain Coherence Audit

Status: Current  
Last reviewed: 2026-04-15  
Owning surface: `cow-sdk-browser-wallet` chain-bound signer and typed chain-management contract  
Refresh trigger: Changes to `BrowserWallet::signer_for_chain`, typed-data chain validation, chain-switch helpers, or shipped browser-wallet proof surfaces  
Related docs:
- [ADR 0004](../adr/0004-feature-gated-browser-wallet-sidecar.md)
- [ADR 0007](../adr/0007-bounded-browser-wallet-support-and-current-browser-runtime-contract.md)
- [ADR 0009](../adr/0009-wasm-verification-consoles-hybrid-extensibility-and-two-tier-proof.md)
- [Architecture](../architecture.md)
- [Verification Guide](../verification-guide.md)
- [Verification Matrix](../verification-matrix.md)
- [Browser-Runtime Proof Posture](../browser-runtime-proof-posture.md)
- [WASM Example Proof-Posture Audit](wasm-example-proof-posture-audit.md)

## Scope

This audit covers:

- chain-bound browser-wallet signers used with quote, signing, gas, and
  transaction flows
- typed browser-wallet chain-management helpers that switch the connected
  wallet session
- the contract between the active wallet session chain and the workflow chain
- reviewer-facing browser-wallet console behavior as consumer proof of that
  contract

It does not cover injected-wallet discovery, multi-wallet selection, or
environment-sensitive extension prompts beyond the chain-coherence boundary.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Signer construction | `BrowserWallet::signer_for_chain` binds wallet-backed workflows to one reviewed target chain | Conforms |
| Runtime validation | Address, signature, gas, and transaction operations revalidate the active session chain before they proceed | Conforms |
| Chain management | Typed switch helpers treat wallet RPC acknowledgement as provisional until the refreshed session confirms the requested chain | Conforms |
| Typed-data signing | Typed-data payloads fail when the domain chain does not match the expected workflow chain | Conforms |
| Example behavior | Console gating remains a user-facing affordance, not the only protection layer | Conforms |

## Current Contract

### Chain-Bound Signer Construction

`BrowserWallet::signer_for_chain` validates the current wallet session chain
before it returns a signer.

### Runtime Revalidation

The returned `Eip1193Signer` stores that expected chain and revalidates it
before address resolution, message signing, typed-data signing, gas
estimation, and transaction submission. `sign_typed_data_payload` also rejects
payloads whose typed-data domain chain does not match the expected chain.

### Typed Chain Management

`BrowserWallet::switch_chain` and `BrowserWallet::switch_or_add_chain` apply
the same authority rule to typed chain-management. A resolved
`wallet_switchEthereumChain` call is not treated as sufficient on its own.
These helpers return success only after the refreshed wallet session confirms
that the requested chain is now active.

### Example Behavior Boundary

This keeps browser-wallet-backed quote, signing, and submission flows aligned
with one reviewed chain authority without widening `cow-sdk-trading` into a
browser-specific crate or relying on example-only guards.

## Evidence

Primary implementation points:

- `crates/browser-wallet/src/lib.rs`
- `crates/browser-wallet/src/mock.rs`
- `crates/browser-wallet/src/error.rs`
- `crates/browser-wallet/src/signer.rs`
- `crates/browser-wallet/src/wallet.rs`
- `examples/wasm/browser-wallet-console/src/lib.rs`

Primary regression coverage:

- `crates/browser-wallet/tests/wallet_contract.rs::signer_for_chain_rejects_wallet_session_mismatches_before_returning_signer`
- `crates/browser-wallet/tests/wallet_contract.rs::chain_bound_signer_rejects_chain_drift_before_address_and_transaction_calls`
- `crates/browser-wallet/tests/wallet_contract.rs::chain_bound_signer_rejects_typed_data_payloads_for_a_different_chain`
- `crates/browser-wallet/tests/wallet_contract.rs::switch_chain_rejects_success_when_the_refreshed_session_stays_on_a_different_chain`
- `crates/browser-wallet/tests/wallet_contract.rs::switch_or_add_chain_rejects_success_when_the_refreshed_session_stays_on_a_different_chain`
- `crates/browser-wallet/tests/wasm_bridge_contract.rs::successful_switch_requests_fail_when_the_refreshed_session_stays_on_a_different_chain`
- `e2e/browser-wallet/tests/injected-chain-coherence.spec.ts`

Validation surface:

```text
cargo fmt --all --check
cargo test -p cow-sdk-browser-wallet
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo build --target wasm32-unknown-unknown -p cow-sdk-browser-wallet
cd crates/browser-wallet && wasm-pack test --headless --chrome
```
