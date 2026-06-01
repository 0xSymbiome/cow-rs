# cow-sdk-signing

Deterministic [CoW Protocol](https://cow.fi) order hashing, EIP-712 typed
data payload construction, order UID generation, and EIP-1271 helper
surfaces.

This crate owns the canonical typed-data boundary
(`cow_sdk_core::TypedDataPayload`) and the explicit payload construction
paths used by the trading orchestration surface and by browser-wallet
runtime adapters. Most end-user code reaches these helpers through
[`cow-sdk`](https://crates.io/crates/cow-sdk); depend on this crate
directly when implementing custom signer integrations or offline
signing tooling.

## Install

```toml
[dependencies]
cow-sdk-signing = "0.1"
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
    Amount::from(1_000_000_000_000_000_000u128),
    Amount::from(2_000_000_000u128),
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
[ADR 0052](https://github.com/cowdao-grants/cow-rs/blob/main/docs/adr/0052-alloy-primitives-canonical-primitive-layer.md)
and [ADR 0022](https://github.com/cowdao-grants/cow-rs/blob/main/docs/adr/0022-ecdsa-signature-v-normalization.md).
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
lowercase-hex `verifyingContract`, no `salt`) directly, pinned by
`PROP-BWL-007` against
`parity/fixtures/signing/eth_sign_typed_data_request.json`. The cow-side
`cow_sdk_alloy_signer::conversion` adapter bridges `TypedDataDomain` to
`alloy_sol_types::Eip712Domain` at the alloy-signer seam where the
alloy-primitive form is needed for ECDSA signing.

## Where to next

- [Getting Started](https://github.com/cowdao-grants/cow-rs/blob/main/docs/getting-started.md)
- [Integrations Guide](https://github.com/cowdao-grants/cow-rs/blob/main/docs/integrations.md)
- [Workspace README](https://github.com/cowdao-grants/cow-rs/blob/main/README.md)

## License

Licensed under GPL-3.0-only. See the workspace
[LICENSE](https://github.com/cowdao-grants/cow-rs/blob/main/LICENSE)
file for the full text.
