# cow-sdk-test

In-memory test doubles for the `cow-rs` SDK public trait seams, so a downstream
application can test its integration without a live orderbook, RPC endpoint, or
wallet — the `tokio-test` / `tower-test` pattern, built only on the public API.

> ⚠️ **Alpha — `0.1.0-alpha`.** Pre-release and not security-audited; the public
> API may change before `0.1.0`. It is published as a pre-release, so Cargo
> selects it only when you opt in (`cow-sdk-test = "0.1.0-alpha.9"`).

## Install

```toml
[dev-dependencies]
cow-sdk-test = "0.1.0-alpha.9"
```

## What it provides

- **`MockOrderbook`** — an `OrderbookClient` double: a canned quote and order-uid,
  orders registered for `order`, recorded requests, and injectable failures
  (`OrderbookFailure::{NotFound, RateLimited, Rejected}`).
- **`MockSigner`** — a `Signer` double that really signs: by default it signs
  EIP-712 typed data and EIP-191 messages with a public development key, so a
  signed order recovers to the address it reports and clears the SDK's
  owner-recovery gate. Reporting a different address models a mismatched signer;
  fixed-signature overrides remain for error-path and wire-shape tests. Records
  sent transactions and signed messages, with failure injection.
- **`MockProvider`** — a `Provider` + `SigningProvider` double: canned chain id,
  allowance, code, and receipt — plus a scriptable receipt sequence for driving a
  receipt-polling wait (a receipt that arrives after a number of polls, a revert,
  or a timeout) — with recorded contract reads and calls.
- **`trading(chain, app_code)`** — one call returning a real `Trading` client
  wired to the doubles, plus the handles (`orderbook`, `signer`, `provider`) to
  assert against.
- **`defaults`** — the same panic-free canned values the doubles return, so a
  hand-built quote and the mock quote never drift.
- Ready-made orderbook errors: `order_not_found()`, `rate_limited()`,
  `rejected(message)`.

Only orderbook errors are canned. `OrderbookError` is not `Clone`, so the
orderbook double stores a small cloneable description and rebuilds the error on
each call — the canned constructors give that something to build. The other error
types need none: `TradingError`, `SubgraphError`, `SigningError`, and
`TransportError` variants are publicly constructible directly, and the signer and
provider doubles surface injected failures as `MockError`. Simulate HTTP-layer
failures by returning the status code you want from a `wiremock` server.

## When to use it

You want to test your CoW integration — that your code posts exactly one order,
handles a rejection, or reads an allowance — deterministically, with no network
or wallet.

Add it as a **dev-dependency**. Reach it through the facade with the opt-in
feature (`cow-sdk = { features = ["testing"] }` → `cow_sdk::testing`), or depend
on `cow-sdk-test` directly.

```rust
use cow_sdk_core::SupportedChainId;
use cow_sdk_test::trading;

let testing = trading(SupportedChainId::Sepolia, "my-app").unwrap();
// Drive `testing.trading` with `testing.signer` in your async test, e.g.
//   testing.trading.post_swap_order(params, &testing.signer, None).await.unwrap();
// then assert on what your code sent. Un-driven, the log is empty:
assert_eq!(testing.orderbook.recorded().sent_orders.len(), 0);
```

Through the facade the same entry point is `cow_sdk::testing::trading`.

## Design

- **Panic-free** (ADR 0033): every canned value is built through infallible
  constructors — no `unwrap`/`expect`/`panic`.
- **Built only on the public trait surface**, so it continuously proves those
  seams are implementable from outside the workspace.
- **Native `Send` doubles** (`Arc<Mutex<_>>`), so they drive a multi-threaded
  `tokio::test`; a `wasm32` variant is a possible follow-on.
- Distinct from the workspace-internal, unpublished `cow-sdk-test-utils`: this is
  the **published, consumer-facing** test surface (ADR 0062 and ADR 0063).

The MSRV is Rust 1.94.0.
