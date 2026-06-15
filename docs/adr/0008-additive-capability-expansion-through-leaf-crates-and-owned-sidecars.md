# ADR 0008: Additive Capability Expansion Through Leaf Crates And Owned Sidecars

- Status: Superseded by [ADR 0001](0001-multi-crate-sdk-family-with-thin-facade.md)
- Date: 2026-04-13
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: extensibility, packages, sidecars, future-growth

## Superseded

The additive-capability-expansion rule — new capability surfaces land as
additive leaf crates or off-by-default features (subgraph, browser-wallet, the
`cow-shed` contracts feature, the published `cow-sdk-test` doubles), never by
widening the default facade closure, so an optional capability a default consumer
does not use adds nothing to its dependency graph — is now recorded in
[ADR 0001](0001-multi-crate-sdk-family-with-thin-facade.md) as the growth rule of
the multi-crate-family-with-thin-facade decision it was a corollary of.
