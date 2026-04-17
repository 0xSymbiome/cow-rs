# Performance Posture

This document maps the performance-sensitive surfaces of the `cow-rs` SDK
family and records the benchmark coverage that protects them against
regressions. Absolute microbenchmark numbers are hardware-sensitive; the
reported ranges below are intended to track order-of-magnitude shifts rather
than pin exact timings.

## Hot Paths

The benchmarked hot paths align with the `0x`-bounded workflows that appear in
every trading and settlement pipeline:

- Order hashing and UID packing for every signed order and batch cancellation.
- Typed-data payload construction for every signing round-trip.
- Deterministic app-data stringification for every persisted order metadata
  payload.
- Orderbook quote fee aggregation for every public quote surface.
- Limit-order construction for every quote-to-post flow.

## Benchmark Coverage

Each benchmark target uses the `criterion` harness and lives under the owning
crate's `benches/` directory. The workflow at `.github/workflows/benchmarks.yml`
compiles and runs the full suite on a scheduled cadence and publishes the HTML
and JSON reports as non-blocking build artifacts.

| Surface | Benchmark | Owning crate |
| --- | --- | --- |
| Order EIP-712 digest | `order_hashing::hash_order` | `cow-sdk-contracts` |
| Order UID pack and extract | `uid_packing::pack_order_uid_params`, `uid_packing::extract_order_uid_params` | `cow-sdk-contracts` |
| Signing typed-data envelope | `typed_data::order_typed_data_payload` | `cow-sdk-signing` |
| App-data deterministic stringify | `stringify::stringify_deterministic` | `cow-sdk-app-data` |
| Orderbook quote fee aggregation | `quote_cost::calculate_total_fee` | `cow-sdk-orderbook` |
| Trading limit-order construction | `order_build::get_order_to_sign` | `cow-sdk-trading` |

## Reported Ranges

The benchmarks are reported as coarse ranges because microbenchmark absolutes
are hardware-sensitive and day-to-day variance is expected. The ranges below
capture the latest scheduled-run measurements on a GitHub-hosted `ubuntu-latest`
runner; each bound is the min-max observed across a representative sampling
window, not a single absolute number.

| Benchmark | Reported range |
| --- | --- |
| `hash_order` | single-digit microseconds |
| `pack_order_uid_params` | sub-microsecond |
| `extract_order_uid_params` | sub-microsecond |
| `order_typed_data_payload` | single-digit microseconds |
| `stringify_deterministic` | single-digit microseconds |
| `calculate_total_fee` | sub-microsecond |
| `get_order_to_sign` | low-single-digit microseconds |

Refresh the table when the next scheduled run reports a shift that crosses one
of these order-of-magnitude boundaries.

## Running Locally

Compile the benchmarks without running them:

```text
cargo bench --workspace --no-run
```

Run a specific surface and print a textual summary:

```text
cargo bench -p cow-sdk-contracts -- --output-format bencher
```

Interactive HTML reports are written under `target/criterion/` after a full
run. The scheduled workflow uploads those reports as build artifacts for each
crate in the matrix.

## Zero-Copy Call Data

Settlement, interaction, and swap encoder outputs hold their call-data payload
as `bytes::Bytes`. Reference-counted cloning means fanning the same encoded
payload across multiple settlement candidates no longer reallocates, which
matters most inside tight solver-evaluation loops. Public JSON wire
serialisation remains a `0x`-prefixed hexadecimal string, so the storage change
is invisible to downstream consumers.

## Address Equality

`cow_sdk_core::Address` compares and hashes case-insensitively through the
lowercase normalised key while its `as_str` accessor preserves the input
casing. Equality on the public address boundary is therefore `O(n)` byte
comparisons without any intermediate allocation, which keeps token-registry
lookups and order-owner checks out of the allocator on every signed-order
path.
