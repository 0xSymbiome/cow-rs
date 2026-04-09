# Strategy

## Goal

Ship a Rust SDK for CoW Protocol that is useful to Rust-native builders on day one, aligned with the established TypeScript surfaces, and maintainable after the first milestone.

## Approach

- Keep the root `cow-sdk` crate narrow and let leaf crates own behavior.
- Map Rust crates to the existing CoW Protocol package surfaces instead of inventing a new abstraction tree.
- Keep deterministic protocol transforms separate from HTTP, GraphQL, pinning, and wallet I/O.
- Make `cow-sdk-trading` the main workflow layer instead of burying orchestration inside transport code.
- Keep browser support additive through `cow-sdk-browser-wallet` and the `browser-wallet` feature on `cow-sdk`.
- Back public claims with repository-visible tests, examples, and CI checks.

## Parity Method

- Pin upstream producer commits in `parity/source-lock.yaml`.
- Capture each surface as a committed fixture contract in `parity/fixtures/*.json`.
- Keep normal builds and tests independent from upstream checkouts.
- Use `scripts/parity-maintainer` only for deliberate refresh and validation against the pinned sources.

## What This Optimizes For

- Reviewable boundaries between hashing, signing, transport, and orchestration
- Reusable leaf crates for bots, services, and browser applications
- A trading-first public entrypoint without hiding lower-level building blocks
- A publishable crate family rather than a single oversized package
