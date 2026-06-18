# cow-sdk-subgraph

Typed [CoW Protocol](https://cow.fi) subgraph query primitives with
saved query documents, explicit raw-GraphQL request contracts, and a
typed, credential-redacting error boundary.

> ⚠️ **Alpha — `0.1.0-alpha`.** Pre-release and not security-audited; the public
> API may change before `0.1.0`. It is published as a pre-release, so Cargo
> selects it only when you opt in (`cow-sdk-subgraph = "0.1.0-alpha.5"`). Review
> it yourself before relying on it with real funds.

This is a read-only analytics crate. It stays separate from the **default**
[`cow-sdk`](https://crates.io/crates/cow-sdk) facade so trading-first consumers
do not pay a GraphQL transport dependency they do not use. Reach it either by
enabling the `subgraph` feature on `cow-sdk` (`cow-sdk = { features =
["subgraph"] }`, surfaced as `cow_sdk::subgraph`) or by depending on this crate
directly when building analytics, reporting, or dashboards over CoW Protocol
subgraph data.

## Surface

- `totals()` — protocol-wide aggregates (tokens, orders, traders, settlements, volume, fees).
- `last_days_volume(days)` / `last_hours_volume(hours)` — recent volume buckets.
- `query(request)` — the escape hatch: any GraphQL document, decoded into a response type you choose.
- `with_config_override(SubgraphConfigOverride::for_chain(chain))` — query another supported chain from the same client.

The typestate `SubgraphApi::builder()` requires a chain, a The Graph API key,
and (on `wasm32`) an explicit transport before `build()` is reachable. The API
key is redacted in every debug, display, and serialized rendering.

## Install

```toml
[dependencies]
cow-sdk-subgraph = "0.1.0-alpha.5"
```

## Example

```rust,no_run
use cow_sdk_core::SupportedChainId;
use cow_sdk_subgraph::{SubgraphApi, SubgraphConfigOverride, SubgraphQueryRequest, TotalsResponse, TOTALS_QUERY};

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let subgraph = SubgraphApi::builder()
    .chain(SupportedChainId::Mainnet)
    .api_key("your-the-graph-api-key")
    .build()?;

// Typed helper for the canonical totals aggregate.
let totals = subgraph.totals().await?;

// The escape hatch: bring your own document and response type.
let raw: TotalsResponse = subgraph
    .query(SubgraphQueryRequest::new(TOTALS_QUERY).with_operation_name("Totals"))
    .await?;

// Query a different supported chain from the same client.
let gnosis_totals = subgraph
    .with_config_override(SubgraphConfigOverride::for_chain(SupportedChainId::GnosisChain))
    .totals()
    .await?;
# let _ = (totals, gnosis_totals, raw);
# Ok(())
# }
```

## Feature flags

| Feature | Default | Enables |
| --- | --- | --- |
| `tracing` | off | Instruments `totals`, `last_days_volume`, `last_hours_volume`, and `query` with `tracing` spans, and enables `cow-sdk-core`'s tracing. |

## Where this fits

This crate is read-only analytics; it does not place, sign, or mutate orders. It
requires a partner The Graph API key — there is no keyless route — and the key is
redacted in every debug, display, and serialized rendering, including the gateway
URL that embeds it (ADR 0025). Chains without a deployed subgraph (for example
Polygon, Avalanche, BNB, and Linea) return `SubgraphError::UnsupportedNetwork`
rather than failing silently. Order placement, signing, and submission live in
[`cow-sdk-trading`](https://crates.io/crates/cow-sdk-trading),
[`cow-sdk-signing`](https://crates.io/crates/cow-sdk-signing), and
[`cow-sdk-orderbook`](https://crates.io/crates/cow-sdk-orderbook).

## Where to next

- [Architecture](https://github.com/0xSymbiome/cow-rs/blob/main/docs/architecture.md)
- [Workspace README](https://github.com/0xSymbiome/cow-rs/blob/main/README.md)

## License

Licensed under GPL-3.0-or-later. See the workspace
[LICENSE](https://github.com/0xSymbiome/cow-rs/blob/main/LICENSE)
file for the full text.
