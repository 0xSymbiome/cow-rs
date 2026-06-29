# Properties Registry

This registry is the canonical public index of invariants and state contracts
for `cow-rs`.

Use it with:

- [Principles](../principles/index.md) for the engineering posture each invariant
  upholds, and [Architecture](../guides/architecture.md) for the crate that owns it
- [Verification](../guides/verification.md) for how the evidence is interpreted and
  the crate and workflow lanes that exercise each surface

Executable coverage stays with the crate or browser surface that owns the
behavior. This registry records what must remain true, who owns it, and where
the current evidence lives.

`Covered` uses these values:

- `Yes`: dedicated executable coverage exists
- `Partial`: deterministic coverage exists, but not through a dedicated
  property or state-machine suite
- `No`: the property is registered, but no executable coverage is attached yet

`Last reviewed` records the most recent date the row was confirmed against the
shipped code. The registry follows a 90-day re-review rhythm that mirrors the
dependency-exception policy in `.github/config/deny.toml`: every row is
re-confirmed at least once per 90-day window, and the date here is refreshed
in the same change that touches the owning surface.

## Methodology

The `Type` column uses four labels. The two randomized-testing labels share a
single evidence mechanism and differ only in what they assert:

- `Property` rows are backed by a `proptest!` macro harness: randomized input
  generation with shrinking to a minimal counterexample on failure, with a
  committed `tests/proptest-regressions/` seed file once a counterexample has
  been shrunk, so a reproduced failure stays reproducible across contributors.
  These rows assert round-trip and composition laws over the codec crates
  (`cow-sdk-core`, `cow-sdk-contracts`, `cow-sdk-signing`, `cow-sdk-app-data`),
  the EIP-712/EIP-191 signature recovery property in `cow-sdk-alloy-signer`, and
  the order-bounds-validator property in `cow-sdk-trading`.
- `Invariant` rows are backed by the same `proptest!` mechanism, with shrinking
  and a committed regression file, but assert request and response *shape*
  invariants rather than round-trip laws: quote-side coercion,
  `validFor`/`validTo` exclusivity, EIP-1271-gated `verificationGasLimit`,
  app-data document and hash composition, and fee normalization. The
  orchestration crates (`cow-sdk-orderbook`, `cow-sdk-trading`) carry these rows
  through their `tests/invariant_contract.rs` suites. The label marks what is
  asserted, not a different testing method.

The remaining two labels — `Contract` and `Public API` — cover typed API
and parser contracts exercised through targeted regression tests, and the
curated `cow-sdk` facade exported-symbol snapshot respectively.

Release-doc guard package list: `cargo tree --invert alloy-provider -p cow-sdk-core -p cow-sdk-contracts -p cow-sdk-signing -p cow-sdk-orderbook -p cow-sdk-subgraph -p cow-sdk-app-data -p cow-sdk-trading -p cow-sdk-alloy-provider -p cow-sdk-alloy-signer -p cow-sdk-alloy -p cow-sdk -p cow-sdk-js -p cow-sdk-test`.

## Registry

The invariants are grouped into per-domain concept files under [`docs/properties/`](.). Each row's executable evidence is enforced by `cargo xtask docs agree`.

### Crate surfaces

| Concept | Invariants | Covered | Id prefix |
| --- | ---: | --- | --- |
| [App-data invariants](app-data.md) | 8 | 8/8 | `PROP-APP` |
| [Alloy client invariants](alloy.md) | 9 | 9/9 | `PROP-AU` `PROP-AU-CANCEL` |
| [Alloy provider invariants](alloy-provider.md) | 15 | 14/15 | `PROP-AP` `PROP-AP-CANCEL` |
| [Alloy signer invariants](alloy-signer.md) | 9 | 8/9 | `PROP-AS` `PROP-AS-CANCEL` |
| [Contract binding invariants](contracts.md) | 22 | 22/22 | `PROP-CON` `PROP-SHED` |
| [Core codec invariants](core.md) | 23 | 23/23 | `PROP-CORE` `PROP-CORE-RX` |
| [Orderbook client invariants](orderbook.md) | 17 | 17/17 | `PROP-ORD` |
| [Subgraph analytics invariants](subgraph.md) | 5 | 5/5 | `PROP-SBG` |
| [Trading lifecycle invariants](trading.md) | 21 | 21/21 | `PROP-TRD` `PROP-TRD-CANCEL-WAIT` `PROP-TRD-WAIT` |
| [JS/WASM boundary invariants](js.md) | 30 | 30/30 | `PROP-WB` |
| [WASM Component boundary invariants](component.md) | 7 | 3/7 | `PROP-CMP` |
| [SDK facade invariants](sdk.md) | 4 | 4/4 | `PROP-SDK` |
| [Test double invariants](test.md) | 1 | 1/1 | `PROP-TST` |

### Cross-cutting concerns

| Concept | Invariants | Covered | Id prefix |
| --- | ---: | --- | --- |
| [Transport policy invariants](transport-policy.md) | 14 | 14/14 | `PROP-TPP` |
| [Signing consistency invariants](signing.md) | 6 | 6/6 | `PROP-SIG` |
| [Security invariants](security.md) | 3 | 3/3 | `PROP-SEC` |
| [Workspace policy invariants](workspace.md) | 9 | 8/9 | `PROP-WS` `PROP-WS-RX` `PROP-WS-TX` |
| [Documentation governance invariants](docs.md) | 5 | 4/5 | `PROP-AUD` `PROP-DOCS` |
