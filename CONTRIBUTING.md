# Contributing

Use this file for the public contribution contract. For crate boundaries,
verification scope, and release posture, see the public docs hub in
[docs/README.md](docs/README.md).

## Baseline Validation

Run these checks before opening a pull request:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo check -p cow-sdk --examples
cargo check --manifest-path examples/native/Cargo.toml --examples
```

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

## Commit Template

The repository includes a commit message template at
`.github/commit-template.md`.

Enable it locally:

```text
git config commit.template .github/commit-template.md
```

Commit subjects use the conventional form
`type(scope): imperative summary`. Body lines stay as flat outcome-focused
bullets beginning with `- `. Pull requests are checked by
`.github/workflows/commit-format.yml`.
