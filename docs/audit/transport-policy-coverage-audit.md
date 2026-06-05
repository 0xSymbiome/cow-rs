# Transport Policy Coverage Audit

Status: Current
Last reviewed: 2026-06-05
Owning surface: `cow-sdk-transport-policy` public retry, jitter, rate-limit, classification, and `Retry-After` parser surfaces, the shared `run_with_retry` driver and its `AttemptOutcome`, `RetrySignal`, and `LimiterKey` types, and the target-neutral `system_now` wall clock, including the HTTP-date delegation to `httpdate::parse_http_date` on `retry_after.rs` and the bounded-jitter contract on `jitter.rs`
Refresh trigger: Changes to any public function on `cow-sdk-transport-policy`; changes to `RetryPolicy`, `JitterStrategy`, `RequestRateLimiter`, `RetryAfter`, `NetworkErrorKind`, or `ErrorClassifier`; changes to `run_with_retry`, `AttemptOutcome`, `RetrySignal`, `LimiterKey`, or `system_now`; changes to the `Retry-After` HTTP-date delegation or its expected accept/reject contract; changes to the workspace `Retry-After` cooldown honor rule documented in `http-transport-contract-audit.md`
Related docs:
- [ADR 0041](../adr/0041-transport-policy-l3-layering.md)
- [ADR 0033](../adr/0033-minimum-viable-panic-surface.md)
- [HTTP Transport Contract Audit](http-transport-contract-audit.md)
- [Transport](../transport.md)
- [Fuzz Coverage Audit](fuzz-coverage-audit.md)
- [Bounded Response Reads Audit](bounded-response-reads-audit.md)

## Scope

This audit covers:

- the `parse_retry_after` accept/reject contract on every documented branch
  (delta-seconds, HTTP-date future and past clamp, empty and whitespace and
  garbage rejection, weekday-without-comma and non-GMT timezone rejection,
  trailing-token and truncation rejection, non-numeric components, invalid
  month names, every calendar month, out-of-range time components, leap-year
  rules, day-31 in 30-day months, pre-epoch clamp, and the three RFC 7231
  HTTP-date forms accepted via `httpdate::parse_http_date` — IMF-fixdate,
  legacy RFC 850, and ANSI C `asctime`)
- the `retry_after.rs` HTTP-date delegation to the upstream `httpdate` crate
  and the parity fixture corpus under `parity/fixtures/retry_after/`
  pinning every accept and reject row
- the `JitterStrategy` delay window invariants for every variant (`None`,
  `Full`, `Equal`, `Decorrelated`) including the zero-base-delay short-circuit
- the `RetryPolicy` decision points (`should_retry_status`,
  `should_retry_network`, `base_backoff_delay` clamps, `delay_for_status`
  case-insensitive `Retry-After` dispatch, `max_attempts(0)` clamp to one)
- `RequestRateLimiter` scope semantics (`PerHost` keys by host, `Global` uses
  a constant key), the `unlimited()` short-circuit, `acquire_global` shared
  bucket behaviour, and the pre-cancelled-token fast path
- the `NetworkErrorKind::from_transport_error_class` total mapping across
  every `TransportErrorClass` variant including the wildcard arm
- the optional `reqwest-classifier` feature's dispatch across `Builder`,
  `Request`, `Connect`, `Timeout`, and `HttpStatus` branches
- the shared `run_with_retry` driver: the attempt loop, rate-limit
  acquisition, the success / retryable-status / retryable-transport /
  non-retryable / exhaustion outcome decisions, the `Retry-After`-aware
  backoff selection, and the `attempt_index`-keyed retry and exhaustion
  telemetry, exercised across the documented scenario matrix
- the target-neutral `system_now` wall clock that feeds the `Retry-After`
  HTTP-date evaluation on both native and `wasm32` targets

It does not cover the per-client attempt closures (request dispatch, success
decoding, and typed-error construction stay owned by `cow-sdk-orderbook`,
`cow-sdk-subgraph`, and `cow-sdk-wasm`; see `http-transport-contract-audit.md`
for the cooldown honor rule and the client-side retry telemetry contract), the
transport adapters (`ReqwestTransport`, `FetchTransport`) which are covered
separately, or the in-browser execution of the wasm regression tests (run by
the wasm workflow).

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Retry-After parser | `parse_retry_after` accepts delta-seconds and every RFC 7231 HTTP-date form (IMF-fixdate, legacy RFC 850, ANSI C `asctime`) via `httpdate::parse_http_date`, rejects every documented malformed shape, and the parity fixture corpus under `parity/fixtures/retry_after/` pins the accept and reject byte contracts | Conforms |
| Jitter window | Every `JitterStrategy` variant returns a delay within the documented `[0, max_delay]` window; `None` returns the capped base delay; `Equal` preserves at least half the capped base delay; the zero-window short-circuit returns `Duration::ZERO` across every strategy | Conforms |
| Retry decision points | `should_retry_status` matches the public `RETRYABLE_STATUSES` list; `should_retry_network` retries only `Timeout`, `Connect`, `Request`, and `Other`; backoff clamps at `max_delay` once the exponent saturates; the case-insensitive `Retry-After` helper honours `429` and `503` and ignores other statuses; `max_attempts(0)` clamps to `1` | Conforms |
| Rate-limit scope | `PerHost` scope keys by `Url::host_str`; `Global` scope uses the constant `"global"` key; `unlimited()` never delays or errors; `acquire_global` shares one bucket; pre-cancelled tokens short-circuit before sleeping the limiter interval | Conforms |
| Error classifier | `NetworkErrorKind::from_transport_error_class` is total across every `TransportErrorClass` variant including `Redirect` and `Upgrade` through the wildcard arm; the optional reqwest classifier maps real `reqwest::Error` shapes into the same partition | Conforms |
| Retry driver | `run_with_retry` returns on the first `AttemptOutcome::Success`, backs off and retries a retryable status or transport signal until `max_attempts`, returns the closure's terminal error on a non-retryable signal without re-dispatching, and surfaces the last terminal error on exhaustion; the recorded backoff sequence matches the policy schedule | Conforms |
| Wall clock | `system_now` returns a real wall-clock `SystemTime` on native and `wasm32` targets without reading the standard `SystemTime::now`, so an HTTP-date `Retry-After` evaluates against the current time on both targets and the retry path never aborts a browser runtime | Conforms |
| Panic-free posture | The `Retry-After` HTTP-date path delegates to `httpdate::parse_http_date`, an upstream maintained crate that surfaces malformed input as a typed `Err` rather than a panic, so an attacker-controlled `Retry-After` header value cannot panic the retry loop; documented panic-allowlist entries on `jitter.rs::bounded_offset` and `transport-policy/src/policy.rs` static-UA constructors stay justified | Conforms |

## Current Contract

### Retry-After Parser

`parse_retry_after(value, now)` is the only public entry point on
`retry_after.rs`. Delta-seconds inputs are accepted when the trimmed value is
composed solely of ASCII digits; surrounding whitespace is trimmed before
dispatch. HTTP-date inputs delegate to `httpdate::parse_http_date`, which
accepts the three forms enumerated in RFC 7231 section 7.1.1.1:

- IMF-fixdate, e.g. `Sun, 06 Nov 1994 08:49:37 GMT`;
- legacy RFC 850, e.g. `Sunday, 06-Nov-94 08:49:37 GMT`;
- ANSI C `asctime`, e.g. `Sun Nov  6 08:49:37 1994`.

In-range past dates and epoch-equal dates clamp to `Duration::ZERO`.
Pre-epoch HTTP-date values surface as `None` because the upstream
`httpdate` parser rejects every date strictly before 1970-01-01 outright,
which collapses cleanly into the "ignore the header" path the
retry decision points already honour. Any other malformed input that the
upstream parser rejects also surfaces as `None`.
The parity fixture corpus at `parity/fixtures/retry_after/` pins three
files: `imf_fixdate_accept.json` (every accept row carries the canonical
Unix timestamp the HTTP-date resolves to), `imf_fixdate_reject.json`
(every reject row must surface as `None`), and `legacy_rfc850.json` (the
RFC 850 capability gain documented in the `## [Unreleased]` block of
`CHANGELOG.md`).

### Jitter Window

`JitterStrategy::delay_for_attempt(base, max_delay, attempt)` returns a
`Duration` in `[0, max_delay]` for every variant. `None` returns the capped
base delay unchanged. `Full(seed)` picks a uniform offset in
`[0, capped_base]`. `Equal(seed)` preserves the lower half of the capped base
delay and jitters the upper half. `Decorrelated(seed)` adds a bounded offset
to the capped base delay. The zero-base-delay short-circuit returns
`Duration::ZERO` across every strategy.

### Retry Decision Points

`RetryPolicy` exposes `should_retry_status`, `should_retry_network`,
`base_backoff_delay`, `delay_for_attempt`, and `delay_for_status` as the
documented decision points. `should_retry_status` forwards to
`is_retryable_status` over the workspace-documented `RETRYABLE_STATUSES`
(`408`, `425`, `429`, `500`, `502`, `503`, `504`).
`should_retry_network` retries only `Timeout`, `Connect`, `Request`, and the
forward-compatible `Other` variant. `base_backoff_delay` clamps the exponent
to at most six before saturating-multiplying the base delay, and the result
is bounded by `max_delay`. `delay_for_status` reads `Retry-After` from the
provided headers case-insensitively for `429` and `503` responses only;
other statuses ignore the header.

### Rate Limiter

`RequestRateLimiter::key_for_url(url)` returns
`url.host_str().unwrap_or("").to_ascii_lowercase()` for `LimiterScope::PerHost`
and the constant `"global"` for `LimiterScope::Global`. The `unlimited()`
constructor produces a limiter with `tokens_per_interval == 0`; the
`acquire_key` short-circuit returns `Ok(())` immediately for that case.
`acquire_global` dispatches to `acquire_key("global", ...)` so every call
shares the same bucket regardless of scope. Cancelled tokens are detected
at the top of `acquire_key` and `sleep_or_cancel` so a pre-cancelled call
never sleeps the limiter interval.

### Error Classification

`NetworkErrorKind::from_transport_error_class` is a total `const fn` over the
`TransportErrorClass` enum from `cow-sdk-core`. Each documented variant maps
to its named `NetworkErrorKind` (`Timeout` -> `Timeout`, `Connect` -> `Connect`,
`Decode` and `Body` -> `Decode`, `Status` -> `HttpStatus(0)`, `Request` ->
`Request`, `Builder` -> `Builder`) and the wildcard `_` arm maps `Redirect`,
`Upgrade`, and any future-added variant to `NetworkErrorKind::Other`. The
optional `reqwest-classifier` feature exposes `ReqwestErrorClassifier` which
maps real `reqwest::Error` shapes into the same partition through the
documented `is_timeout`/`is_connect`/`is_decode`/`is_body`/`status`/
`is_request`/`is_builder` dispatch ladder.

### Retry Driver

`run_with_retry(policy, rate_limiter, limiter_key, attempt)` is the shared
retry loop consumed by the orderbook, subgraph, and IPFS clients. It is
generic over the success payload `T` and the caller's error type `E`; the
`attempt` closure runs once per 1-based attempt and returns an
`AttemptOutcome<T, E>` that is either `Success(value)` or a
`Failure { error, signal }`. For each attempt the driver acquires a
rate-limit token from the bucket named by `LimiterKey` (`Global` or
`PerUrl`), runs the closure, and then:

- returns `Ok(value)` on `Success`;
- on a `RetrySignal::HttpStatus { status, headers }` that
  `should_retry_status` accepts and while `attempt_index < max_attempts`,
  sleeps for `delay_for_status` and retries;
- on a `RetrySignal::Transport { class }` that `should_retry_network`
  accepts and while `attempt_index < max_attempts`, sleeps for
  `delay_for_attempt` and retries;
- otherwise returns the closure's terminal `error` — a non-retryable signal
  returns immediately without re-dispatching, and a retryable signal returns
  the last terminal error once `max_attempts` is reached.

`run_with_retry` imposes no `Send` bound on the attempt future, so the same
driver serves native (`Send`) and browser (`?Send`) transports. Cancellation
is external: the driver holds no caller token and relies on the call-site
`Cancellable::cancel_with` wrapper to drop the future — and any in-flight
acquire or backoff sleep — when a token fires.

### Wall Clock

`system_now()` returns the current wall-clock `std::time::SystemTime`. On
native targets it delegates to `std::time::SystemTime::now()`. On `wasm32`
targets, where the standard clock is unavailable, it reads the browser wall
clock through `web_time::SystemTime` and re-anchors the elapsed duration onto
a `std::time::SystemTime` starting at `UNIX_EPOCH`, saturating at the epoch
for a clock reported earlier. The driver passes `system_now()` to
`delay_for_status`, so an HTTP-date `Retry-After` evaluates against the
current time on both targets without the standard clock's wasm abort.

## Evidence

Primary implementation points:

- `crates/transport-policy/src/retry_after.rs`
- `crates/transport-policy/src/jitter.rs`
- `crates/transport-policy/src/retry.rs`
- `crates/transport-policy/src/rate_limit.rs`
- `crates/transport-policy/src/classify.rs`
- `crates/transport-policy/src/policy.rs`
- `crates/transport-policy/src/status.rs`
- `crates/transport-policy/src/runner.rs`
- `crates/transport-policy/src/time.rs`

Primary regression coverage:

- `crates/transport-policy/src/runner.rs` (`tests::immediate_success_does_not_sleep`, `tests::retryable_status_then_success_backs_off_once`, `tests::delta_retry_after_overrides_backoff_floor`, `tests::http_date_retry_after_uses_the_injected_clock`, `tests::persistent_retryable_status_exhausts_attempts`, `tests::persistent_transport_error_exhausts_attempts`, `tests::non_retryable_status_returns_immediately`, `tests::non_retryable_transport_returns_without_redispatch`, `tests::mixed_transport_then_status_then_success`, `tests::no_retry_policy_makes_one_attempt`)
- `crates/wasm/tests/wasm_retry_runner_contract.rs::system_now_returns_a_wall_clock_value_without_panicking`
- `crates/wasm/tests/wasm_retry_runner_contract.rs::retryable_status_drives_backoff_without_panicking`
- `crates/transport-policy/tests/retry_after_contract.rs`
- `crates/transport-policy/tests/retry_after_fixture_contract.rs`
- `parity/fixtures/retry_after/imf_fixdate_accept.json`
- `parity/fixtures/retry_after/imf_fixdate_reject.json`
- `parity/fixtures/retry_after/legacy_rfc850.json`
- `crates/transport-policy/tests/classify_contract.rs::network_error_kind_mapping_round_trip_is_total`
- `crates/transport-policy/tests/classify_contract.rs::reqwest_classifier::reqwest_classifier_maps_invalid_url_to_builder_or_request`
- `crates/transport-policy/tests/classify_contract.rs::reqwest_classifier::reqwest_classifier_maps_unreachable_host_to_connect_or_timeout`
- `crates/transport-policy/tests/classify_contract.rs::reqwest_classifier::reqwest_classifier_maps_status_500_to_http_status`
- `crates/transport-policy/tests/policy_contract.rs::prop_tpp_001_default_orderbook_transport_policy_is_stable`
- `crates/transport-policy/tests/policy_contract.rs::prop_tpp_009_explicit_constructor_disables_tracing_and_preserves_parts`
- `crates/transport-policy/tests/policy_contract.rs::prop_tpp_010_default_trading_uses_trading_user_agent_and_orderbook_limiter`
- `crates/transport-policy/tests/policy_contract.rs::prop_tpp_011_default_ipfs_disables_retry_and_timeout_and_uses_unlimited_limiter`
- `crates/transport-policy/tests/policy_contract.rs::prop_tpp_012_with_setters_replace_only_their_targeted_field`
- `crates/transport-policy/tests/policy_contract.rs::prop_tpp_014_builder_round_trip_preserves_every_setter`
- `crates/transport-policy/tests/policy_contract.rs::prop_tpp_016_none_jitter_returns_capped_base_delay_unchanged`
- `crates/transport-policy/tests/policy_contract.rs::prop_tpp_018_equal_jitter_returns_at_least_half_capped_base_delay`
- `crates/transport-policy/tests/policy_contract.rs::prop_tpp_020_zero_base_delay_returns_zero_across_every_strategy`
- `crates/transport-policy/tests/policy_contract.rs::prop_tpp_021_unlimited_rate_limiter_never_delays_or_errors`
- `crates/transport-policy/tests/policy_contract.rs::prop_tpp_022_global_scope_uses_constant_key_regardless_of_host`
- `crates/transport-policy/tests/policy_contract.rs::prop_tpp_024_pre_cancelled_token_returns_cancelled_immediately`
- `crates/transport-policy/tests/policy_contract.rs::prop_tpp_028_should_retry_status_matches_the_public_retryable_list`
- `crates/transport-policy/tests/policy_contract.rs::prop_tpp_029_should_retry_network_only_retries_documented_kinds`
- `crates/transport-policy/tests/policy_contract.rs::prop_tpp_030_base_backoff_clamps_to_max_delay_across_attempt_range`
- `crates/transport-policy/tests/policy_contract.rs::prop_tpp_031_retry_after_helper_is_case_insensitive`
- `crates/transport-policy/tests/policy_contract.rs::prop_tpp_032_retry_builder_round_trip_and_zero_attempts_clamps_to_one`
- `fuzz/fuzz_targets/fuzz_parse_retry_after.rs`
- `fuzz/fuzz_targets/fuzz_retry_policy_delay.rs`
- `fuzz/fuzz_targets/fuzz_jitter_delay_for_attempt.rs`

Validation surface:

```text
cargo fmt --all --check
cargo clippy -p cow-sdk-transport-policy --all-targets --all-features -- -D warnings
cargo test -p cow-sdk-transport-policy --all-features
cargo llvm-cov -p cow-sdk-transport-policy --all-features --summary-only --fail-under-lines 85
RUSTDOCFLAGS="-D warnings" cargo doc -p cow-sdk-transport-policy --no-deps
```
