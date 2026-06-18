# cow-sdk-signing

Deterministic [CoW Protocol](https://cow.fi) order hashing, EIP-712 typed
data payload construction, order UID generation, and EIP-1271 helper
surfaces.

> ⚠️ **Alpha — `0.1.0-alpha`.** Pre-release and not security-audited; the public
> API may change before `0.1.0`. It is published as a pre-release, so Cargo
> selects it only when you opt in (`cow-sdk-signing = "0.1.0-alpha.5"`). Review
> it yourself before relying on it with real funds.

This crate owns the canonical typed-data boundary
(`cow_sdk_core::TypedDataPayload`) and the explicit payload construction
paths used by the trading orchestration surface and by host-supplied
EIP-1193 wallet integrations. Most end-user code reaches these helpers through
[`cow-sdk`](https://crates.io/crates/cow-sdk); depend on this crate
directly when implementing custom signer integrations or offline
signing tooling.

## What it provides

- **Order and cancellation signing** — `sign_order` / `sign_order_with_scheme`
  and `sign_order_cancellation(s)` over a pluggable `TypedDataSigner` /
  `DigestSigner` seam, returning the canonical `r‖s‖v` hex.
- **Offline order identity** — `generate_order_id` computes the 56-byte UID and
  EIP-712 digest with no signing key.
- **Signer-facing typed data** — `order_typed_data_payload` and
  `order_cancellations_typed_data_payload` build the EIP-712 domain, types, and
  message ready to hand to a wallet.
- **Domain separators** — `domain` and `domain_separator(_for)` for any chain or
  settlement-contract override.
- **EIP-1271 helpers** — `eip1271_signature_payload` ABI-encodes the verifier
  payload from an existing ECDSA signature, and `Eip1271Signer` is the
  custom-signer seam for smart accounts. The `SigningScheme` enum and
  `verify_eip1271_signature(_cached)` are re-exported from `cow-sdk-contracts`
  for one-import ergonomics.
- **Optional EIP-1271 verification cache** — `InMemoryEip1271Cache` (positive-only,
  TTL-bounded) behind the `in-memory-cache` feature; the `Eip1271Cache` trait and
  `NoopEip1271Cache` are always available.
- **Typed wallet rejection** — a user-declined signature is surfaced as a typed
  rejection carrying the EIP-1193 provider error code, not a redacted string.

## Install

```toml
[dependencies]
cow-sdk-signing = "0.1.0-alpha.5"
```

## Minimal example

```rust
use cow_sdk_core::{
    Address, Amount, AppDataHash, BuyTokenDestination, OrderData, OrderKind,
    SellTokenSource, SupportedChainId,
};
use cow_sdk_signing::{generate_order_id, order_typed_data_payload};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let owner = Address::new("0x3333333333333333333333333333333333333333")?;
let order = OrderData::new(
    Address::new("0x1111111111111111111111111111111111111111")?,
    Address::new("0x2222222222222222222222222222222222222222")?,
    owner,
    Amount::from_units(1, 18)?,
    Amount::from_units(2000, 6)?,
    1_700_000_000,
    AppDataHash::new(
        "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    )?,
    Amount::ZERO,
    OrderKind::Sell,
    false,
    SellTokenSource::Erc20,
    BuyTokenDestination::Erc20,
);

// Deterministic order identity — no signing key required.
let generated = generate_order_id(SupportedChainId::Mainnet, &order, &owner, None)?;
println!("order uid: {}", generated.order_id.to_hex_string());

// Signer-facing EIP-712 payload, ready to hand to a wallet for signing.
let payload = order_typed_data_payload(SupportedChainId::Mainnet, &order, None)?;
assert_eq!(payload.primary_type, "Order");
# Ok(())
# }
```

## EIP-712 and EIP-191

EIP-191 message hashing routes through
`alloy_primitives::eip191_hash_message`. EIP-712 message digests route
through `alloy_sol_types::SolStruct::eip712_signing_hash` per
[ADR 0052](https://github.com/0xSymbiome/cow-rs/blob/main/docs/adr/0052-alloy-primitives-canonical-primitive-layer.md)
and [ADR 0022](https://github.com/0xSymbiome/cow-rs/blob/main/docs/adr/0022-ecdsa-signature-v-normalization.md).
65-byte recoverable signature byte representation routes through
`cow_sdk_contracts::RecoverableSignature`, which validates the trailing
recovery byte against the canonical accept set `{0, 1, 27, 28}` and
delegates byte assembly to alloy's `Signature::from_bytes_and_parity` /
`Signature::as_bytes`; the compact-2098 form routes through the same
typestate via `RecoverableSignature::parse_erc2098` and
`RecoverableSignature::to_erc2098`.

`cow_sdk_core::traits::typed_data::TypedDataDomain` is a cow-owned
`#[non_exhaustive]` struct with cow-owned `Serialize` / `Deserialize`
impls; the cow `Serialize` emits the canonical EIP-1193
`eth_signTypedData_v4` second-parameter wire shape (numeric `chainId`,
lowercase-hex `verifyingContract`, no `salt`) directly, governed by
[ADR 0040](https://github.com/0xSymbiome/cow-rs/blob/main/docs/adr/0040-wallet-provider-callback-boundary-for-js-consumers.md). The cow-side
`cow_sdk_alloy_signer::conversion` adapter bridges `TypedDataDomain` to
`alloy_sol_types::Eip712Domain` at the alloy-signer seam where the
alloy-primitive form is needed for ECDSA signing.

## Feature flags

| Feature | Default | Enables |
| --- | --- | --- |
| `in-memory-cache` | off | The capacity-bounded, TTL-respecting `InMemoryEip1271Cache` plus the `Clock` / `SystemClock` seam. The `Eip1271Cache` trait and `NoopEip1271Cache` are available without it. |
| `tracing` | off | `tracing` spans on the sign, cancellation, and verify paths; enables `cow-sdk-core` and `cow-sdk-contracts` tracing. |

## Where this fits

This crate orchestrates hashing and payload construction; the ECDSA/key operation
is delegated to the caller's `cow_sdk_core::Signer` — no private keys or keystore
live here. The order-hash and UID math, the `RecoverableSignature` codec, and the
`SigningScheme` enum are owned by
[`cow-sdk-contracts`](https://crates.io/crates/cow-sdk-contracts); this crate
re-uses and re-exports them. It does no HTTP or order submission (that is
[`cow-sdk-orderbook`](https://crates.io/crates/cow-sdk-orderbook)) and builds no
orders or quotes (that is [`cow-sdk-trading`](https://crates.io/crates/cow-sdk-trading)).

## Where to next

- [Getting Started](https://github.com/0xSymbiome/cow-rs/blob/main/docs/getting-started.md)
- [Integrations Guide](https://github.com/0xSymbiome/cow-rs/blob/main/docs/integrations.md)
- [Workspace README](https://github.com/0xSymbiome/cow-rs/blob/main/README.md)

## License

Licensed under GPL-3.0-or-later. See the workspace
[LICENSE](https://github.com/0xSymbiome/cow-rs/blob/main/LICENSE)
file for the full text.
