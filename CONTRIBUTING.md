# Contributing

Use this file for the public contribution contract. For crate boundaries,
verification scope, and release posture, see the public docs hub in
[docs/README.md](docs/README.md).

## Code Of Conduct

Participation in this repository is governed by the project
[Code of Conduct](docs/code-of-conduct.md). Report unacceptable behavior
through the channels named in that document.

## Before Your First Contribution

Install the pinned Rust toolchain and the clippy and rustfmt components
so local checks match the repository CI lanes:

```text
rustup show
rustup component add clippy rustfmt
```

`rustup show` picks up the pinned toolchain from `rust-toolchain.toml`,
which keeps the local toolchain version aligned with the CI contract.

- The public Rust floor and bump rules are documented in the
  [MSRV policy](docs/msrv-policy.md), including the 30-day notice window and
  the dependency, stable-feature, and security-advisory triggers for a bump.

## Baseline Validation

Run these checks before opening a pull request:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo check --manifest-path examples/native/Cargo.toml --examples
cargo run-deterministic-examples
cargo check-alloy-provider-invariant
cargo check-alloy-signer-invariant
cargo tree --invert alloy-provider -p cow-sdk-core -p cow-sdk-contracts -p cow-sdk-signing -p cow-sdk-orderbook -p cow-sdk-subgraph -p cow-sdk-app-data -p cow-sdk-trading -p cow-sdk-browser-wallet -p cow-sdk-transport-wasm -p cow-sdk-alloy-provider -p cow-sdk-alloy-signer -p cow-sdk-alloy -p cow-sdk
```

The Alloy dependency gates enforce explicit native adapter allow-lists:
`alloy-provider` is allowed only in `cow-sdk-alloy-provider` and
`cow-sdk-alloy`, while `alloy-signer-local` is allowed only in
`cow-sdk-alloy-signer` and `cow-sdk-alloy`. Use the Cargo aliases rather than
reading raw `cargo tree` output directly.

## Test Runner — `cargo nextest`

CI uses the nextest profile in `.github/config/nextest.toml` for the canonical
workspace test runner settings.

Install nextest locally with:

```text
cargo install cargo-nextest --locked
```

Common local commands:

```text
cargo nextest run --workspace
cargo test --workspace --doc
```

## Cargo Aliases

The repository exposes maintainer tooling through Cargo aliases in
`.cargo/config.toml`. Use `cargo --list` to see the available aliases.

Common examples:

```text
cargo parity-validate --source-lock parity/source-lock.yaml
cargo check-property-citations
```

The clippy gate runs under the workspace lint posture declared in the root
`Cargo.toml`, which enables both the `pedantic` and `nursery` groups at warn
level and treats warnings as errors. Contributors should expect the gate to
surface pedantic and nursery findings on new code and either resolve them
inline or, when a fix would require altering the shipped public contract,
attach an `#[allow(clippy::<lint>)]` at module scope with a one-line
justification. Broad file-scope silencing is not accepted in review.

## WASM And Browser Surfaces

Run these checks when a change touches WASM-facing crates or browser-wallet
surfaces:

```text
cargo build --target wasm32-unknown-unknown -p cow-sdk
cargo build --target wasm32-unknown-unknown -p cow-sdk --features browser-wallet
cargo build --target wasm32-unknown-unknown -p cow-sdk-browser-wallet
cargo build --target wasm32-unknown-unknown -p cow-sdk-app-data
cargo build --target wasm32-unknown-unknown -p cow-sdk-transport-wasm
cargo check --target wasm32-unknown-unknown --manifest-path examples/wasm/cow-trader-dioxus/Cargo.toml
```

## Running Fuzz Targets Locally

The `fuzz/` crate ships cargo-fuzz harnesses for the deterministic codec
and validation boundaries across the `cow-sdk-*` crate family — contract
encoders, signing digests, app-data documents, orderbook and transport
error parsing, and the order-bounds validator among them. The
authoritative target list is `cargo fuzz list --fuzz-dir fuzz`, and the
[fuzz coverage audit](docs/audit/fuzz-coverage-audit.md) records the
boundary classes and per-target seed contracts. The fuzz crate is a
standalone package outside the root workspace and requires the Rust
nightly channel.

Install the nightly toolchain and cargo-fuzz once:

```text
rustup toolchain install nightly
cargo install cargo-fuzz --locked
```

List the shipped targets to confirm the toolchain is wired up:

```text
cargo fuzz list --fuzz-dir fuzz
```

Run a single target for one minute locally:

```text
cargo +nightly fuzz run <target> --fuzz-dir fuzz -- -max_total_time=60
```

Reproduce a crash from a saved corpus seed by pointing the target at the
seed file directly:

```text
cargo +nightly fuzz run <target> --fuzz-dir fuzz fuzz/corpus/<target>/<seed>
```

Fuzz targets are run locally and on demand (see `fuzz/README.md`); the local
commands above are sufficient for day-to-day contributor verification.

## Documentation

Update public docs when a change moves a public crate boundary, support claim,
release contract, or verification story.

Build the workspace documentation locally before opening a pull request that
touches any rustdoc surface:

```text
cargo doc --workspace --no-deps
```

Use `--all-features` if the change touches feature-gated documentation:

```text
cargo doc --workspace --no-deps --all-features
```

## Typed-Primitive Conventions

The cow-rs primitive layer uses cow-owned `#[repr(transparent)]` newtypes
over `alloy_primitives` types for the byte-typed identity family
(`Address`, `Hash32`, `AppDataHash`, `HexData`, `OrderUid`) and the
numeric type (`Amount`) per
[ADR 0052](docs/adr/0052-alloy-primitives-canonical-primitive-layer.md).
The newtypes carry cow-owned `Display`, `Serialize`, and `Deserialize`
impls where the cow wire contract diverges from alloy defaults
(lowercase 0x-prefixed `Address` `Display`; strict-decimal-only
`Amount` `Deserialize` that rejects `0x`, `0o`, and
`0b`-prefixed input).

Contributors extending the public surface should preserve the
`#[repr(transparent)]` layout (keeping the inner field private) so cow
values remain zero-cost convertible to the underlying alloy primitive via
`From::from(value).into()` or the typed `as_*` / `into_*` accessors, and
route accessor methods onto the newtype as inherent methods
rather than adding extension traits. The canonical owned String
accessor on the byte-typed identity newtypes is `to_hex_string()`,
following the Rust stdlib convention that `to_*` returns owned and
`as_*` returns a borrow.

Pre-release semantic versioning means the cow-rs project ships
hard-replacement migrations without `#[deprecated]` annotations during
the 0.x window; the first functional crate-family release at `0.1.0`
is the first tag carrying stability guarantees.

## Branch Naming

Use one focused branch per change set.

Branch names follow `<type>/<short-surface-summary>`.

Use the same change-type vocabulary as the commit convention when it fits:
`feat/`, `fix/`, `docs/`, `refactor/`, `test/`, or `chore/`.

Keep the suffix short, lowercase, and tied to the surface being changed. For
example: `docs/release-checklist` or `fix/orderbook-timeout`.

## Pull Request Process

Open a pull request after the relevant local checks for the touched surface
pass.

Keep each pull request scoped to one crate boundary, documentation surface, or
validation change set.

In the pull request body, summarize the public effect, the validation you ran,
and any follow-up work that remains out of scope.

Request review from repository maintainers once the branch is ready for
review. Update the branch in place if review identifies follow-up fixes.

## Required Status Checks By Path

The repository CI lanes gate merge on the `ci.yml` aggregate status
check plus the path-filtered end-to-end lanes below. Branch-protection
configuration reflects the same list on the public fork target and is
maintained by repository administrators.

| Lane | Workflow | Blocks PRs that touch |
| --- | --- | --- |
| Core CI aggregate | `.github/workflows/ci.yml` (`ci-success`) | every pull request |
| Browser wallet WASM | `.github/workflows/browser-wallet-wasm.yml` | `crates/browser-wallet/**`, `crates/transport-wasm/**`, `examples/wasm/cow-trader-dioxus/**`, and any workspace change that pulls the browser-wallet path (`crates/core/**`, `crates/sdk/**`, `Cargo.lock`, `Cargo.toml`, `rust-toolchain.toml`) |

The path filters on each workflow keep the end-to-end lanes off PRs
that cannot plausibly regress the covered surface, so workflows only
run (and only block) when the change touches code that the lane
exercises.

Repository administrators maintain the required-status-check set on
the protected branches so the browser-wallet-wasm lane blocks merge
whenever a PR matches the paths listed above. Contributors do not need
to configure this; the path filters keep the lane deterministic, and
the branch-protection list enforces the merge gate.

## Merge Policy

Do not merge a pull request until required CI is green and maintainer review
is complete.

Prefer squash merge for single-topic pull requests so the public history stays
concise and easy to audit.

If a pull request intentionally preserves multiple independently meaningful
commits, rebase merge is acceptable. Avoid merge commits unless repository
maintainers explicitly request them.

## Commit Template

The repository includes a commit message template at
`.github/commit-template.md`.

Enable it locally:

```text
git config commit.template .github/commit-template.md
```

Enable the repository-owned local hook:

```text
git config core.hooksPath .githooks
```

Commit subjects use the conventional form
`type(scope): imperative summary`. Body lines stay as flat outcome-focused
bullets beginning with `- `. The local `commit-msg` hook and
`.github/workflows/commit-format.yml` enforce the same rule.
