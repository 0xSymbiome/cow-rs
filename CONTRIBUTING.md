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

## Baseline Validation

Run these checks before opening a pull request:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo check -p cow-sdk --examples
cargo check --manifest-path examples/native/Cargo.toml --examples
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
cd examples/wasm/sdk-verification-console && wasm-pack build --target web
cd examples/wasm/browser-wallet-console && wasm-pack build --target web
```

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
| Browser wallet end-to-end | `.github/workflows/browser-wallet-e2e.yml` | `crates/browser-wallet/**`, `examples/wasm/browser-wallet-console/**`, `e2e/browser-wallet/**`, and any workspace change that pulls the browser-wallet path (`crates/core/**`, `crates/sdk/**`, `Cargo.lock`, `Cargo.toml`, `rust-toolchain.toml`) |
| SDK verification end-to-end | `.github/workflows/sdk-verification-e2e.yml` | `examples/wasm/sdk-verification-console/**`, `e2e/sdk-verification/**`, and every workspace crate that the verification console exercises (`crates/app-data/**`, `crates/contracts/**`, `crates/core/**`, `crates/orderbook/**`, `crates/sdk/**`, `crates/signing/**`, `crates/trading/**`, `Cargo.lock`, `Cargo.toml`, `rust-toolchain.toml`) |

The path filters on each workflow keep the end-to-end lanes off PRs
that cannot plausibly regress the covered surface, so workflows only
run (and only block) when the change touches code that the lane
exercises.

Repository administrators maintain the required-status-check set on
the protected branches so the browser-wallet-e2e and
sdk-verification-e2e lanes block merge whenever a PR matches the
paths listed above. Contributors do not need to configure this; the
path filters keep the lanes deterministic, and the branch-protection
list enforces the merge gate.

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
