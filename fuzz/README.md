# cow-sdk-fuzz

`cargo-fuzz` harnesses for the deterministic codec boundaries shipped by
the `cow-sdk-*` crate family. The fuzz crate is a standalone package
excluded from the root workspace because `cargo-fuzz` requires the Rust
nightly channel and unstable `RUSTFLAGS`; keeping the fuzz crate outside
the workspace means the stable toolchain the rest of the repository uses
is never forced onto nightly.

## Supported platforms

`libfuzzer-sys` and `cargo-fuzz` rely on the LLVM sanitizer runtime the
Rust compiler ships on Unix-like x86-64 and AArch64 targets. The
supported platforms for running fuzz targets are Linux and macOS.
Building the fuzz crate under `cargo +nightly fuzz build` works on any
platform the Rust nightly toolchain supports, but running a target
locally on Windows requires the LLVM AddressSanitizer runtime
(`clang_rt.asan_dynamic-x86_64.dll`) on `PATH`, which ships with the
Visual Studio MSVC toolset rather than the Rust toolchain. Linux or
macOS remains the supported local-run environment.

## Toolchain

Every command below assumes the nightly toolchain is installed and
`cargo-fuzz` is available on `$PATH`:

```sh
rustup toolchain install nightly
cargo install cargo-fuzz --locked
```

Then invoke cargo-fuzz with the explicit `+nightly` prefix and
`--fuzz-dir fuzz` so the subcommand reads this crate's manifest rather
than walking up into the workspace root:

```sh
cargo +nightly fuzz list --fuzz-dir fuzz
cargo +nightly fuzz build --fuzz-dir fuzz
cargo +nightly fuzz run <target> --fuzz-dir fuzz -- -max_total_time=60
```

### Working corpus and seeds

`fuzz/corpus/` is gitignored — no corpus or seed file is committed (the
cargo-fuzz convention). libFuzzer's accumulated corpus can grow to
hundreds of MB, so keep it out of the tree: point each run at an
out-of-tree write-back directory, with any local seeds you maintain as a
read-only second directory.

```sh
cargo +nightly fuzz run <target> --fuzz-dir fuzz \
  "$HOME/cow-fuzz-corpus/<target>" "$HOME/cow-fuzz-seeds/<target>" -- -max_total_time=60
```

libFuzzer writes new inputs only to the first directory and reads the
rest read-only, so the accumulated corpus, local seeds, and crash
artifacts all stay outside the repository — the tracked tree never
bloats and a clone or cross-tree copy stays fast.

## Layout and naming

- `fuzz_targets/` — one `.rs` file per target. The file stem matches the
  `[[bin]]` name declared in `Cargo.toml`.
- `corpus/<target>/` — local-only working corpus and seeds for the
  matching target. The whole `fuzz/corpus/` tree is gitignored; nothing
  in it is committed. Baseline `seed-*` inputs and the libFuzzer mutation
  accumulator both stay in maintainer-local working copies and are
  regenerated on demand from the seed classes recorded in the
  [fuzz coverage audit](../../docs/audit/fuzz-coverage-audit.md).
- `artifacts/<target>/` — crash reproducers written by libFuzzer on
  failure. Also not tracked in version control.
- `dictionaries/<target>.dict` — optional libFuzzer token dictionary for
  targets whose input format benefits from a dictionary.

Target names follow the pattern `fuzz_<surface>_<action>`. `<surface>`
is the codec boundary under test (`order_uid`, `typed_data`,
`app_data_cid`, `order_signature`, `subgraph_graphql_error`,
`ethflow_create_order`, `erc20_permit_typed_data`,
`order_bounds_validator`,
`orderbook_rejection`, `app_data_merge`, `transport_error`) and
`<action>` is the specific invariant the target asserts (`pack_unpack`,
`digest`, `roundtrip`, `classify`, `decode`, `encode`, `hash`,
`merge`).

## Seed classes

Seeds are not committed (`fuzz/corpus/` is gitignored). Each target is
seeded locally from three classes, regenerated on demand:

- `canonical` — at least one seed derived from `parity/fixtures/*.json`
  or a pinned upstream test fixture.
- `boundary` — at least one input-domain edge: an empty payload,
  all-zero or all-`0xff` bytes, a single-element list, a capped
  maximum-length list, or a numeric extreme.
- `adversarial` — at least one seed from a documented edge case,
  upstream regression, named audit risk, or known historical bug.

The recommended local-disk count is at least five files per target. The
per-target seed-class coverage and its parity-fixture provenance are
recorded in the [fuzz coverage audit](../../docs/audit/fuzz-coverage-audit.md)
and the `PROPERTIES.md` rows each target strengthens; the binary seeds
stay in maintainer-local working copies.

## Encoder Fuzz Targets

One target exercises the `alloy::sol!`-generated encoder surface for
shipped contract binding families in `cow-sdk-contracts`:

- `fuzz_ethflow_create_order_encode` — round-trips
  `CoWSwapEthFlow.createOrder(EthFlowOrderData)` through the matching
  decoder and asserts every struct field survives the encode/decode
  cycle.

## Validator Fuzz Targets

- `fuzz_order_bounds_validator` — maps arbitrary bytes into an
  `OrderCreation`, signing scheme, optional app-data signer, timestamp,
  and EthFlow flag, then asserts `OrderBoundsValidator::validate`
  always returns a typed result without panicking. Its corpus seeds the
  happy path, each validator rejection class, timestamp extremes, and
  the WETH/native sentinel pair.

## Parser, Merge, And Transport Fuzz Targets

- `fuzz_orderbook_rejection_decode` — feeds arbitrary response bodies to
  the typed orderbook rejection parser under `400` and `500` statuses,
  asserting no panic and deterministic `Display` rendering for any typed
  rejection.
- `fuzz_app_data_merge` — maps arbitrary bytes into a bounded
  `(serde_json::Value, AppDataParams)` pair, runs the typed
  quote-to-post app-data merge, and asserts canonical JSON idempotency
  for successful merges.
- `fuzz_transport_error_classify` — maps arbitrary status, body, and
  header bytes into the typed transport-error partition and asserts that
  public diagnostics do not leak credential-bearing URL snippets.

## Input-size convention

Targets that accept raw `&[u8]` carry a documented minimum-length gate
at the top of the file (for example `const MIN_INPUT_LEN: usize = 56;`
for `fuzz_order_uid_pack_unpack`). Inputs shorter than the gate return
early without panicking so the fuzzer itself stays alive and the minimal
well-formed input is easy to reproduce from the corpus.

Targets that factor through `arbitrary::Arbitrary` cap the structured
input at a documented constant such as `const MAX_FUZZ_INPUT: usize =
4096;` so individual runs stay bounded even when libFuzzer explores
deeply nested shapes.

## Reproducing a crash

When a local run surfaces a crash, libFuzzer writes the reproducer
under `fuzz/artifacts/<target>/` and the offending corpus entry under
`fuzz/corpus/<target>/`. Reproduce locally by pointing the target at
the saved input directly:

```sh
cargo +nightly fuzz run <target> --fuzz-dir fuzz fuzz/corpus/<target>/<seed>
```

## Adding a new target

1. Add `[[bin]]` to `Cargo.toml` with `name`, `path =
   "fuzz_targets/<target>.rs"`, and `test = false`, `doc = false`,
   `bench = false`.
2. Create `fuzz_targets/<target>.rs`. Start with `#![no_main]`, import
   `libfuzzer_sys::fuzz_target`, document the minimum-input-size gate
   or the `Arbitrary`-derived struct, parse defensively (no `unwrap` on
   arbitrary input), call the helper under test, and assert the
   documented invariant. Keep the assertion messages specific so a
   crash in CI names the diverging field.
3. Create at least 5 deterministic local seed files (under
   `fuzz/corpus/<target>/`, gitignored) covering the canonical,
   boundary, and adversarial classes, and record the new target's
   seed-class coverage in the fuzz coverage audit.
4. Smoke-run locally: `cargo +nightly fuzz run <target> --fuzz-dir fuzz
   -- -runs=1000`.

A target is complete when it builds under
`cargo +nightly fuzz build --fuzz-dir fuzz`, runs panic-free on a
1000-iteration smoke, carries an assertion on the invariant its boundary
guarantees, and ships a documented seed corpus.
