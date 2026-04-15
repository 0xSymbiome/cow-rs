# Getting Started

`cow-rs` is a trading-first Rust SDK for CoW Protocol. This guide gives one
canonical first-touch path:

1. identify the main crate surface
2. build a ready-state `TradingSdk`
3. produce deterministic signed-order output
4. simulate a signed order post without live services
5. branch into the maintained native and WASM example families

## Install Surface

The stable published install surface is:

```text
cargo add cow-sdk
```

The first crates.io release is not live yet. Until publication, evaluate the
same surface from a local checkout or run the maintained example crate in this
repository.

Repo-local dependency shape:

```toml
[dependencies]
cow-sdk = { path = "/path/to/cow-rs/crates/sdk" }
```

If you want the full deterministic onboarding flow without wiring your own
project first, use the commands in this guide from the repository root.

## Ready-State SDK Setup

`cow-sdk` is the thin facade crate. For quote, post, and off-chain cancellation
flows, a ready-state `TradingSdk` needs:

- a chain id
- an `appCode`
- an owner or signer at call time

Minimal setup:

```rust
use cow_sdk::{Address, SupportedChainId, TradingSdk};

fn build_sdk() -> Result<TradingSdk, Box<dyn std::error::Error>> {
    let owner = Address::new("0x1111111111111111111111111111111111111111")?;
    let sdk = TradingSdk::builder()
        .with_chain_id(SupportedChainId::Sepolia)
        .with_app_code("cow-rs/getting-started")
        .with_owner(owner)
        .build()?;

    Ok(sdk)
}
```

This is the canonical trading-first entrypoint. Read-only analytics stay in
`cow-sdk-subgraph`, and browser-wallet support stays additive behind the
`browser-wallet` feature.

## Step 1: Produce Deterministic Signed-Order Output

Run the maintained signing scenario:

```text
cargo run --manifest-path examples/native/Cargo.toml --example signing_roundtrip
```

This path is deterministic. It does not depend on live orderbook services,
wallet extensions, or deployed WASM pages.

The output confirms the stable signing surface:

- typed-data primary type
- order digest
- order id
- signature bytes and signing scheme
- deterministic cancellation signature output

Use this step when you want to verify the signing contract before you involve
transport or runtime adapters.

## Step 2: Simulate A Signed Limit-Order Post

Run the maintained limit-order scenario:

```text
cargo run --manifest-path examples/native/Cargo.toml --example limit_order_simulation
```

This scenario uses a mock signer plus a mock orderbook client bound to Sepolia.
It proves the order-construction and submission shape without pretending that a
mock transport is the same thing as a live orderbook session.

The output reports:

- the posted order id
- the signature length and signing scheme
- the quote id that fed submission
- the submitted sell and buy amounts
- uploaded app-data activity

Together, `signing_roundtrip` and `limit_order_simulation` give the shortest
path from facade setup to deterministic signed-order output.

## Step 3: Branch Into The Maintained Scenario Families

After the deterministic path is clear, use the existing example families by
goal instead of looking for a second onboarding guide.

Native follow-ons:

- `quote_only_simulation` for quote construction without posting
- `trading_sdk_simulation` for the broader quote, post, allowance, approval,
  and cancellation shape
- `order_lifecycle_simulation` for lookup plus off-chain cancellation
- `orderbook_transport_roundtrip` for typed orderbook transport behavior
- `orderbook_live_probe` and `subgraph_live_query` for opt-in live checks

WASM follow-ons:

- `examples/wasm/sdk-verification-console` for browser-hosted deterministic SDK
  inspection
- `examples/wasm/browser-wallet-console` for explicit injected-wallet flows

Entry pages:

- [Examples](examples.md)
- [Native examples](../examples/native/README.md)
- [WASM examples](../examples/wasm/README.md)

## Deterministic Versus Environment-Sensitive Work

The first-touch path above stays deterministic on purpose.

When you move beyond it, the boundary changes:

- real orderbook usage depends on service availability and your runtime config
- wallet-backed signing depends on the signer or wallet you supply
- browser-wallet flows depend on a supported browser runtime and wallet session
- published install commands depend on the first crates.io release being live

Use [Architecture](architecture.md) for crate boundaries and
[Release Checklist](release-checklist.md) when you need the full validation and
publication posture.

## Next Reads

- [Documentation Index](README.md)
- [Examples](examples.md)
- [Architecture](architecture.md)
- [Verification Guide](verification-guide.md)
