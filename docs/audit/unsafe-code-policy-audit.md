# Unsafe-Code Policy Audit

Status: Current
Last reviewed: 2026-06-16
Owning surface: Workspace `unsafe_code = deny` lint declared in `Cargo.toml` workspace lint section
Refresh trigger: any introduction of an `unsafe` block on a public path; any change that weakens or removes the workspace `deny` lint

## Scope

This audit covers:

- the workspace `unsafe_code = "deny"` lint declaration
- every workspace crate that opts into workspace lints through its crate
  manifest
- public Rust source under `crates/*/src/**/*.rs`
- the CI clippy lane that compiles the workspace with warnings denied

It does not cover generated third-party code outside the workspace crate
sources.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Workspace lint | `Cargo.toml` declares `unsafe_code = "deny"` under `[workspace.lints.rust]` | Conforms |
| Crate adoption | Every workspace crate manifest declares `[lints] workspace = true` | Conforms |
| Source posture | Public crate source contains no `unsafe` blocks, `unsafe impl`, or `unsafe trait` definitions | Conforms |
| CI enforcement | The shared clippy job runs with `-D warnings`, so a newly introduced `unsafe` block fails before merge | Conforms |

## Current Contract

### Workspace Lint

The workspace lint declaration in `Cargo.toml` sets
`unsafe_code = "deny"`. Each crate in the workspace inherits the lint set by
declaring `[lints] workspace = true` in its `Cargo.toml`, so no published crate
is exempt from the deny-level unsafe-code posture.

### Public Source

The reviewed source tree contains no public-path `unsafe` block, `unsafe impl`,
or `unsafe trait`. The only source matches for `unsafe` are the workspace lint
declaration and prose comments explaining that certain constructors remain
free of unsafe code.

### CI Enforcement

The shared quality gate runs:

```text
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Because the unsafe-code lint is deny-level at the Rust lint layer, adding an
`unsafe` block to a workspace crate fails this job even before clippy warning
promotion is considered.

## Evidence

Primary implementation points:

- `Cargo.toml`
- `crates/core/Cargo.toml`
- `crates/contracts/Cargo.toml`
- `crates/signing/Cargo.toml`
- `crates/app-data/Cargo.toml`
- `crates/orderbook/Cargo.toml`
- `crates/trading/Cargo.toml`
- `crates/subgraph/Cargo.toml`
- `crates/sdk/Cargo.toml`

Primary regression coverage:

- `.github/workflows/_quality-gate.yml` clippy job

Validation surface:

```text
cargo clippy --workspace --all-targets --all-features -- -D warnings
rg -n "unsafe|unsafe_code|allow\\(unsafe_code\\)|forbid\\(unsafe_code\\)" crates Cargo.toml -g "*.rs" -g "Cargo.toml"
```
