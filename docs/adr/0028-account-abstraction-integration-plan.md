# ADR 0028: Integrate Account Abstraction Through Provider Capabilities And EIP-1271 Signing

- Status: Accepted (amended)
- Date: 2026-04-27
- Last reviewed: 2026-05-22
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: account-abstraction, provider, signing, eip1271, eip4337, eip7702, eip7212
- Related: [ADR 0014](0014-eip1271-verification-cache.md), [ADR 0024](0024-asyncprovider-asyncsigningprovider-capability-split.md), [ADR 0039](0039-typescript-callable-wasm-sdk-surface.md), [ADR 0040](0040-wallet-provider-callback-boundary-for-js-consumers.md), [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md)

## Decision

Account-abstraction support enters through the existing provider and signing
capability split. EIP-7702 set-code EOAs, EIP-4337 user-operation or paymaster
flows, and EIP-7212 secp256r1 verification integrate by explicit provider,
signer, and EIP-1271 signature-provider adapters. Core order construction and
read-only chain access depend on `Provider`; signer creation requires
`SigningProvider`; signature production flows through `Signer` or
explicit EIP-1271 provider surfaces.

EIP-1271 callbacks for wasm consumers follow the facade-resolves-callback
pattern. `signOrderWithCustomEip1271` is the JavaScript smart-account entry
point: the callback returns the final ABI-encoded signature (verifier plus
signature blob), and the Rust facade wraps that resolved hex string in a
`cow_sdk_signing::Eip1271Signer` implementation. No
`js_sys::Function` or `JsValue` is stored in the trait object; the trait
remains trivially `Send + Sync` and composes with native consumers.

Contributor rule for cross-ABI DTOs that include an `OrderUid` or
`OrderDigest`: source the field from `as_str()` (the canonical hex string),
never from `as_bytes()`. The wasm crate's PROP-WB-004 covers this invariant;
CI and contract tests enforce it.

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

- Public surface: read-only operations are bounded by `Provider`; signer
  creation is bounded by `SigningProvider`; bundler and paymaster types
  stay out of the core facade until stabilized.
- Runtime and support: account-abstraction integrations are explicit adapters;
  the SDK does not silently choose a bundler, paymaster, or wallet runtime.
- Validation and review: EIP-1271 cache behavior, browser-wallet trust
  posture, typestate construction, and parity rows for signing, trading,
  contracts, and orderbook remain review anchors.
- Cost: account-abstraction ergonomics may require adapter crates, preserving
  dependency and trust-boundary clarity for the stable facade.

## Alternatives Rejected

- Add one root account-abstraction client: this would mix read-only access,
  signer creation, bundler transport, paymaster policy, and wallet trust.
- Treat every smart-account path as browser-wallet signing: native and contract
  account flows need the same trait contract without assuming EIP-1193.
- Bypass EIP-1271 for contract-mediated signatures: that would duplicate the
  reviewed verification path and weaken cache behavior.

## Links

- [Providers](../providers/README.md)
- [Adapting alloy providers](../providers/adapting-alloy.md)
- [Parity scope surface boundaries](../parity.md#surface-matrix)
- [Verification matrix crate contracts](../verification.md#crate-evidence-matrix)
- [Core provider traits](../../crates/core/src/traits/provider.rs)
- [Trading EIP-1271 signature provider](../../crates/trading/src/types/seams.rs)
- See also: ADR 0024, ADR 0031, ADR 0039, and ADR 0040.

**Proven by:**

- [EIP-1271 Verification Cache Audit](../audit/eip1271-verification-cache-audit.md)
- [Browser Wallet Trust Posture Audit](../audit/browser-wallet-trust-posture-audit.md)
- [Typestate Builder Contract Audit](../audit/typestate-builder-contract-audit.md)

## Amendment 2026-05-22: canonical primitive layer (per ADR 0052)

The contributor rule above on cross-ABI DTOs that include an `OrderUid`
or `OrderDigest` is preserved in substance: the canonical hex string is
sourced from the cow newtype's `to_hex_string()` accessor (owned hex
form, following the Rust stdlib convention that `to_*` returns owned)
or through the `Display` impl, never from `as_bytes()`. The prior
`as_str()` accessor name retires per
[ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md); the
cow-owned newtype shape, the canonical lowercase hex wire form, and the
PROP-WB-004 + contract-test enforcement are preserved unchanged.
