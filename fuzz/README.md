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
supported platforms for running fuzz targets are Linux and macOS; the
scheduled workflow under `.github/workflows/` targets `ubuntu-latest`.
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

## Layout and naming

- `fuzz_targets/` — one `.rs` file per target. The file stem matches the
  `[[bin]]` name declared in `Cargo.toml`.
- `corpus/<target>/` — seed inputs for the matching target. Filenames
  describe the boundary case they exercise. Not tracked in version
  control so corpus entries can accumulate out-of-band.
- `artifacts/<target>/` — crash reproducers written by libFuzzer on
  failure. Also not tracked in version control.
- `dictionaries/<target>.dict` — optional libFuzzer token dictionary for
  targets whose input format benefits from a dictionary.

Target names follow the pattern `fuzz_<surface>_<action>`. `<surface>`
is the codec boundary under test (`order_uid`, `typed_data`,
`app_data_cid`, `order_signature`, `subgraph_graphql_error`) and
`<action>` is the specific invariant the target asserts
(`pack_unpack`, `digest`, `roundtrip`, `classify`, `decode`).

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

When a scheduled run surfaces a crash, libFuzzer writes the reproducer
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
3. Smoke-run locally: `cargo +nightly fuzz run <target> --fuzz-dir fuzz
   -- -runs=1000`.

A target is complete when it builds under
`cargo +nightly fuzz build --fuzz-dir fuzz`, runs panic-free on a
1000-iteration smoke, and carries an assertion on the invariant its
boundary guarantees.
