# ADR 0028: Integrate Account Abstraction Through Provider Capabilities And EIP-1271 Signing

- Status: Accepted
- Date: 2026-04-27
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: account-abstraction, provider, signing, eip1271, eip4337, eip7702, eip7212
- Related: [ADR 0014](0014-eip1271-verification-cache.md), [ADR 0024](0024-asyncprovider-asyncsigningprovider-capability-split.md)

## Decision

Account-abstraction support enters through the existing provider and signing
capability split. EIP-7702 set-code EOAs, EIP-4337 user-operation or paymaster
flows, and EIP-7212 secp256r1 verification are integrated by explicit provider,
signer, and EIP-1271 signature-provider adapters. Core SDK order construction
and read-only chain access continue to depend on `AsyncProvider`; signer
creation continues to require `AsyncSigningProvider`; signature production
continues to flow through `AsyncSigner` or explicit EIP-1271 provider surfaces.

The root facade does not grow a monolithic account-abstraction client. Bundler,
paymaster, wallet, and passkey-specific behavior belongs in leaf adapters until
the protocol wire contract requires a stable SDK-owned type.

## Why

Account abstraction changes who signs, who submits, and where verification
happens. It does not remove the need for explicit read-only provider access,
signing-capable provider access, or contract-mediated signature verification.
Keeping those capabilities separate avoids hidden wallet or bundler
dependencies in read-only flows and keeps order ownership reviewable.

## Must Remain True

- Public surface: read-only operations are bounded by `AsyncProvider`; signer
  creation is bounded by `AsyncSigningProvider`; account-abstraction-specific
  bundler or paymaster types stay out of the core facade until stabilized.
- Runtime and support: EIP-7702, EIP-4337, and EIP-7212 integrations are
  explicit adapters; the SDK does not silently choose a bundler, paymaster, or
  browser-wallet runtime.
- Validation and review: EIP-1271 cache behavior, browser-wallet trust
  posture, typestate construction, and parity-scope rows for signing, trading,
  contracts, and orderbook remain the review anchors for new flows.
- Cost: account-abstraction ergonomics may require adapter crates, but that
  cost preserves dependency and trust-boundary clarity for the stable facade.

## Alternatives Rejected

- Add one root account-abstraction client: this would mix read-only access,
  signer creation, bundler transport, paymaster policy, and wallet trust into a
  single broad dependency surface.
- Treat every smart-account path as browser-wallet signing: native and contract
  account flows need the same trait contract without assuming an EIP-1193
  runtime.
- Bypass EIP-1271 for contract-mediated signatures: that would duplicate the
  reviewed verification path and weaken cache behavior.

## Links

- [Providers](../providers/README.md)
- [Adapting alloy providers](../providers/adapting-alloy.md)
- [Parity scope surface boundaries](../parity-scope.md#surface-boundaries)
- [Verification matrix crate contracts](../verification-matrix.md#crate-contracts)
- [Core provider traits](../../crates/core/src/traits.rs)
- [Trading EIP-1271 signature provider](../../crates/trading/src/types.rs)

**Proven by:**

- [EIP-1271 Verification Cache Audit](../audit/eip1271-verification-cache-audit.md)
- [Browser Wallet Trust Posture Audit](../audit/browser-wallet-trust-posture-audit.md)
- [Typestate Builder Contract Audit](../audit/typestate-builder-contract-audit.md)
