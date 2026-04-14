# Browser Wallet Chain Coherence Audit

Status: Current  
Last reviewed: 2026-04-14

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

## Decision Summary

| Area | Decision |
| --- | --- |
| Signer construction | Expose `BrowserWallet::signer_for_chain` for workflows that already know the target chain |
| Runtime validation | Revalidate the wallet session chain before address, signature, gas, and transaction operations |
| Chain management | Treat switch RPC acknowledgement as provisional and return success only after the refreshed session reports the requested chain |
| Typed-data signing | Reject payloads whose domain chain does not match the expected workflow chain |
| Example behavior | Keep UI gating as a user-facing affordance, not as the only protection layer |

## Current Contract

`BrowserWallet::signer_for_chain` validates the current wallet session chain
before it returns a signer.

The returned `Eip1193Signer` stores that expected chain and revalidates it
before address resolution, message or typed-data signing, gas estimation, and
transaction submission. `sign_typed_data_payload` also rejects payloads whose
typed-data domain chain does not match the expected chain.

`BrowserWallet::switch_chain` and `BrowserWallet::switch_or_add_chain` apply
the same authority rule to typed chain-management. A resolved
`wallet_switchEthereumChain` call is not treated as sufficient on its own.
These helpers return success only after the refreshed wallet session confirms
that the requested chain is now active.

This keeps browser-wallet-backed quote, signing, and submission flows aligned
with one reviewed chain authority without widening `cow-sdk-trading` into a
browser-specific crate or relying on example-only guards.

## Evidence

Relevant source files:

- `crates/browser-wallet/src/lib.rs`
- `crates/browser-wallet/src/mock.rs`
- `crates/browser-wallet/src/error.rs`
- `crates/browser-wallet/src/signer.rs`
- `crates/browser-wallet/src/wallet.rs`
- `examples/wasm/browser-wallet-console/src/lib.rs`

Relevant contract coverage:

- `crates/browser-wallet/tests/wallet_contract.rs::signer_for_chain_rejects_wallet_session_mismatches_before_returning_signer`
- `crates/browser-wallet/tests/wallet_contract.rs::chain_bound_signer_rejects_chain_drift_before_address_and_transaction_calls`
- `crates/browser-wallet/tests/wallet_contract.rs::chain_bound_signer_rejects_typed_data_payloads_for_a_different_chain`
- `crates/browser-wallet/tests/wallet_contract.rs::switch_chain_rejects_success_when_the_refreshed_session_stays_on_a_different_chain`
- `crates/browser-wallet/tests/wallet_contract.rs::switch_or_add_chain_rejects_success_when_the_refreshed_session_stays_on_a_different_chain`
- `crates/browser-wallet/tests/wasm_bridge_contract.rs::successful_switch_requests_fail_when_the_refreshed_session_stays_on_a_different_chain`
- `e2e/browser-wallet/tests/injected-chain-coherence.spec.ts`

Validation commands:

```text
cargo fmt --all --check
cargo test -p cow-sdk-browser-wallet
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo build --target wasm32-unknown-unknown -p cow-sdk-browser-wallet
cd crates/browser-wallet && wasm-pack test --headless --chrome
```
