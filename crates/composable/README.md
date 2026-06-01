# cow-sdk-composable

Reserved crate for CoW Protocol composable (programmatic, conditional) order
helpers.

## Status

This crate is a placeholder. It exposes no public API yet and is not part of the
published `cow-sdk` surface, so depending on it has no effect today.

Composable orders let a single signed primitive authorize many orders over time —
for example time-weighted schedules, stop-loss orders, or threshold-triggered
trades. Helpers for building and verifying these order types are planned. Until
they land, use the standard trading flow exposed by the
[`cow-sdk`](https://docs.rs/cow-sdk) facade.

## When it ships

Once this crate gains a public API, this README and the crate rustdoc will
document its purpose, a minimal example, and its feature and runtime boundaries,
following the same crate-documentation contract as the other `cow-sdk` crates.
