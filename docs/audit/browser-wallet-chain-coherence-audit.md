# Browser Wallet Chain Coherence Audit

Status: Current  
Last reviewed: 2026-06-03
Owning surface: `cow-sdk-browser-wallet` chain-bound signer and typed chain-management contract  
Refresh trigger: Changes to `BrowserWallet::signer_for_chain`, typed-data chain validation, chain-switch helpers, or shipped browser-wallet proof surfaces  
Related docs:
- [ADR 0004](../adr/0004-feature-gated-browser-wallet-sidecar.md)
- [ADR 0007](../adr/0007-bounded-browser-wallet-support-and-current-browser-runtime-contract.md)
- [Architecture](../architecture.md)
- [Verification Guide](../verification.md)
- [Verification Matrix](../verification.md)
- [Browser-Runtime Proof Posture](../browser-runtime-proof-posture.md)

## Scope

This audit covers:

- chain-bound browser-wallet signers used with quote, signing, gas, and
  transaction flows
- typed browser-wallet chain-management helpers that switch the connected
  wallet session
- the contract between the active wallet session chain and the workflow chain
- the canonical browser-wallet example as a consumer demonstration of that
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
| Example behavior | The canonical browser-wallet example switches to and validates the target chain through `signer_for_chain` before any signed action, backed by the crate-level protections above | Conforms |

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
browser-specific crate or relying on example-only guards. The canonical
browser-wallet example (`examples/wasm/cow-trader-dioxus/`) demonstrates the
contract end to end — it calls `switch_chain` when the wallet is on another
network and then signs only through `signer_for_chain` — while the crate tests,
not the example, remain the proof source for live actions.

## Evidence

Primary implementation points:

- `crates/browser-wallet/src/lib.rs`
- `crates/browser-wallet/src/mock.rs`
- `crates/browser-wallet/src/error.rs`
- `crates/browser-wallet/src/signer.rs`
- `crates/browser-wallet/src/wallet/chain_mgmt.rs`
- `crates/browser-wallet/src/wallet/chain.rs`
- `examples/wasm/cow-trader-dioxus/src/main.rs`

Primary regression coverage:

- `crates/browser-wallet/tests/wallet_contract.rs::signer_for_chain_rejects_wallet_session_mismatches_before_returning_signer`
- `crates/browser-wallet/tests/wallet_contract.rs::chain_bound_signer_rejects_chain_drift_before_address_and_transaction_calls`
- `crates/browser-wallet/tests/wallet_contract.rs::chain_bound_signer_rejects_typed_data_payloads_for_a_different_chain`
- `crates/browser-wallet/tests/wallet_contract.rs::switch_chain_rejects_success_when_the_refreshed_session_stays_on_a_different_chain`
- `crates/browser-wallet/tests/wallet_contract.rs::switch_or_add_chain_rejects_success_when_the_refreshed_session_stays_on_a_different_chain`
- `crates/browser-wallet/tests/wasm_bridge_contract.rs::successful_switch_requests_fail_when_the_refreshed_session_stays_on_a_different_chain`

Validation surface:

```text
cargo fmt --all --check
cargo test -p cow-sdk-browser-wallet
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo build --target wasm32-unknown-unknown -p cow-sdk-browser-wallet
cd crates/browser-wallet && wasm-pack test --headless --firefox
```
