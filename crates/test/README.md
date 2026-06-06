# cow-sdk-test

In-memory test doubles for the `cow-rs` SDK public trait seams, so a downstream
application can test its integration without a live orderbook, RPC endpoint, or
wallet — the `tokio-test` / `tower-test` pattern, built only on the public API.

## What it provides

- **`MockOrderbook`** — an `OrderbookClient` double: a canned quote and order-uid,
  orders registered for `order`, recorded requests, and injectable failures
  (`OrderbookFailure::{NotFound, RateLimited, Rejected}`).
- **`MockSigner`** — a `Signer` double: canned signatures, transaction hash, and
  gas, with recorded sent transactions and signed messages, plus failure
  injection.
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

## When to use it

You want to test your CoW integration — that your code posts exactly one order,
handles a rejection, or reads an allowance — deterministically, with no network
or signing.

Add it as a **dev-dependency**. Reach it through the facade with the opt-in
feature (`cow-sdk = { features = ["testing"] }` → `cow_sdk::testing`), or depend
on `cow-sdk-test` directly.

```rust
use cow_sdk::testing::trading;
use cow_sdk::SupportedChainId;

let testing = trading(SupportedChainId::Sepolia, "my-app")?;
// Drive `testing.sdk` with `testing.signer` in your async test:
//   testing.sdk.post_swap_order(params, &testing.signer, None).await?;
// then assert on what your code sent:
assert_eq!(testing.orderbook.recorded().sent_orders.len(), 1);
```

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
