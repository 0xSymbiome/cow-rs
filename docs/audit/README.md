# Audits

This directory contains focused public engineering audits for `cow-rs`.

## Audit Contract

Each audit should record:

- the surface being assessed
- the current decision or finding
- the evidence that supports that decision
- any remaining boundary that stays intentionally out of scope

Audits stay narrow. They are not a substitute for guides, ADRs, or release
runbooks.

## Index

| Audit | Scope | Status |
| --- | --- | --- |
| [Duplication Audit](duplication.md) | Mechanical duplication in request execution, signing payload preparation, and trading posting wrappers | Current |
| [CID Dependency Audit](cid-dependency.md) | App-data CID dependency selection, compatibility boundaries, and fail-closed encoding behavior | Current |
| [Trading Orderbook Context Audit](trading-orderbook-context.md) | Canonical chain and environment authority for orderbook-bound trading helpers | Current |
