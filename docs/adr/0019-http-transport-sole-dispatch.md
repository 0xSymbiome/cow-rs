# ADR 0019: HTTP Transport Is The Sole Live-Dispatch Surface On The Orderbook And Subgraph Clients

- Status: Superseded by [ADR 0013](0013-http-transport-injection-and-typestate-builders.md)
- Date: 2026-04-22
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: transport, orderbook, subgraph, wasm, async, error-typing

## Superseded

The sole-live-dispatch invariant — `OrderbookApi` and `SubgraphApi` each hold
exactly one `Arc<dyn HttpTransport + Send + Sync>` with no parallel
`reqwest::Client`, the success channel returns a `TransportResponse` (2xx status,
headers, body), and non-2xx responses stay on the typed
`TransportError::HttpStatus { status, headers, body }` channel so no layer
fabricates response metadata on the success path — is now recorded in
[ADR 0013](0013-http-transport-injection-and-typestate-builders.md) as the
enforcement corollary of the transport-injection decision.
