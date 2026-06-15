# CoW Browser-Wallet Trade — Minimal Dioxus Example

A minimal, end-to-end example of the **`cow-sdk-browser-wallet`** crate, written
entirely in Rust with **Dioxus** (web/wasm). On one screen it discovers an
injected wallet (EIP-6963), connects, signs, and trades a CoW Protocol order —
**WETH ↔ COW on Sepolia** — using only the SDK's public types. No JavaScript and
no raw RPC.

Everything is in a single, well-commented `src/main.rs`: a Dioxus component for
the UI, and a handful of small `async` functions for the SDK calls.

## Run

```bash
rustup target add wasm32-unknown-unknown
cargo install dioxus-cli --locked   # provides `dx` (v0.7.x)
dx serve --platform web             # then open the printed URL
```

You need a browser wallet (MetaMask, Rabby, …) on **Sepolia** with a little test
ETH or COW.

## What it demonstrates

| Step | SDK call | EIP-1193 / HTTP |
| --- | --- | --- |
| Connect | `BrowserWallet::discover()` → `wallet_at(i)` → `connect()` | `eth_requestAccounts`, `eth_chainId` |
| Sign | `Eip1193Signer::sign_message` | `personal_sign` |
| Wrap | `wrap_interaction` + `send_transaction` | `eth_sendTransaction` |
| Approve | `approval_transaction` + `send_transaction` | `eth_sendTransaction` |
| Quote | `Trading::quote_results` | orderbook `GET` |
| Swap | `Trading::post_swap_order` | `eth_signTypedData_v4` + orderbook `POST` |

## Using it

1. **Connect** — *Discover wallets* lists every injected provider via
   `discovery.wallets()`; click the one you want (it does `wallet_at(index)` →
   `connect()`). With several extensions installed, no wallet is auto-selected.
2. **Sign** — a `personal_sign` round-trip that proves signing works.
3. **Trade** — pick a direction, enter an amount, then *Approve* → *Get quote* →
   *Sign & submit swap*. If you only hold ETH, *Wrap* it to WETH first.

### Sepolia specifics

- **Network / liquidity.** The example uses the **production** API
  (`CowEnv::Prod`, `api.cow.fi/sepolia`); barn/staging returns
  `404 … no liquidity` for these pairs.
- **Fees.** The fee is a fixed gas cost charged in the *sell* token (~0.04 COW
  vs ~0.0008 WETH), so a sell smaller than the fee returns `404 … no route
  found`. Selling COW needs a larger amount (≥ ~0.1 COW); 0.01 WETH is fine — the
  direction toggle sets a sensible default for each side.
- **Balance.** The orderbook checks your on-chain balance
  (`400 … insufficient balance`), so sell the token you actually hold.
- **Chain.** *Sign & submit* calls `wallet.switch_chain(Sepolia)`
  (`wallet_switchEthereumChain`) if you're on another network, then validates
  with `signer_for_chain` — which is fail-closed and never signs on the wrong
  chain. Quotes are read-only. Some wallets (e.g. **Phantom**) require **Testnet
  Mode** to be enabled before they'll use Sepolia.

> Tip: a read-only `priceQuality: "fast"` quote needs no balance, so you can
> probe any pair/size before committing funds.

## How it maps to the SDK

- `cow-sdk` (feature `browser-wallet`) provides `BrowserWallet`, `Eip1193Signer`,
  and the `Trading` client. On `wasm32` the orderbook builder defaults its HTTP
  transport to the browser `fetch`, so the example needs no transport crate and
  no transport wiring.

The trading client is built exactly as a real app would — the same code on
native and in the browser, because the orderbook builder supplies the default
transport for each target:

```rust
let orderbook = OrderbookApi::builder_from_context(ApiContext::new(CHAIN, ENV))
    .build()?; // wasm32 → browser fetch · native → reqwest
let trading = Trading::builder()
    .chain_id(CHAIN)
    .app_code(APP_CODE)
    .orderbook(orderbook)
    .build()?;
```

Read-only actions use the chainless `wallet.signer()`; signing actions use
`wallet.signer_for_chain(…)`, which validates the network (and the example
switches it) first. SDK errors propagate with `anyhow` and are shown verbatim.

The example also models the consumer-side hygiene the SDK leaves to the
application: it refuses a non-positive or sub-wei (truncated-to-zero) amount
before constructing any wrap, approve, or quote; it leaves slippage unset so the
SDK applies the quote's AUTO (suggested) tolerance, and surfaces a short warning
when that suggestion signals a fee-dominated trade; and it notes a chain mismatch
at connect time. Input and economic policy belong to the consumer — the SDK stays
a faithful primitive (an ERC-20 `approve(0)`, for instance, is the canonical
allowance reset and is never blocked at the SDK level).

## Quality

The example is held to the same bar as the crates:

```bash
cargo check  --target wasm32-unknown-unknown                  # clean, 0 warnings
cargo clippy --target wasm32-unknown-unknown -- -D warnings   # 0 lints
cargo fmt --check                                             # rustfmt-clean
```

## Production note

`dx serve` runs without a Content-Security-Policy. A production build should set
`connect-src` to the host it talks to:
`connect-src 'self' https://api.cow.fi;`.
