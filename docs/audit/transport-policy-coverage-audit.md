# Transport Policy Coverage Audit

Status: Current
Last reviewed: 2026-05-14
Owning surface: `cow-sdk-transport-policy` public retry, jitter, rate-limit, classification, and `Retry-After` parser surfaces, including the deterministic civil-day arithmetic on `retry_after.rs` and the bounded-jitter contract on `jitter.rs`
Refresh trigger: Changes to any public function on `cow-sdk-transport-policy`; changes to `RetryPolicy`, `JitterStrategy`, `RequestRateLimiter`, `RetryAfter`, `NetworkErrorKind`, or `ErrorClassifier`; changes to the `Retry-After` IMF-fixdate civil-day arithmetic; changes to the `parse_retry_after` accept/reject contract; changes to the workspace `Retry-After` cooldown honor rule documented in `http-transport-contract-audit.md`
Related docs:
- [ADR 0041](../adr/0041-transport-policy-l3-layering.md)
- [ADR 0033](../adr/0033-minimum-viable-panic-surface.md)
- [HTTP Transport Contract Audit](http-transport-contract-audit.md)
- [Transport](../transport.md)
- [Fuzz Coverage Audit](fuzz-coverage-audit.md)

## Scope

This audit covers:

- the `parse_retry_after` accept/reject contract on every documented branch
  (delta-seconds, IMF-fixdate future and past clamp, empty and whitespace and
  garbage rejection, weekday-without-comma and non-GMT timezone rejection,
  trailing-token and truncation rejection, non-numeric components, invalid
  month names, every calendar month, out-of-range time components, leap-year
  rules, day-31 in 30-day months, pre-epoch clamp)
- the `retry_after.rs` civil-day arithmetic panic-free posture under any
  attacker-controlled year value through `i64` promotion
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

It does not cover the orderbook retry orchestrator (see
`http-transport-contract-audit.md` for the `Retry-After` cooldown honor rule),
the transport adapters (`ReqwestTransport`, `FetchTransport`) which are
covered separately, or the wasm-target build path.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Retry-After parser | `parse_retry_after` accepts delta-seconds and IMF-fixdate values, rejects every documented malformed shape, and the civil-day arithmetic stays panic-free under any attacker-controlled year value through `i64` promotion | Conforms |
| Jitter window | Every `JitterStrategy` variant returns a delay within the documented `[0, max_delay]` window; `None` returns the capped base delay; `Equal` preserves at least half the capped base delay; the zero-window short-circuit returns `Duration::ZERO` across every strategy | Conforms |
| Retry decision points | `should_retry_status` matches the public `RETRYABLE_STATUSES` list; `should_retry_network` retries only `Timeout`, `Connect`, `Request`, and `Other`; backoff clamps at `max_delay` once the exponent saturates; the case-insensitive `Retry-After` helper honours `429` and `503` and ignores other statuses; `max_attempts(0)` clamps to `1` | Conforms |
| Rate-limit scope | `PerHost` scope keys by `Url::host_str`; `Global` scope uses the constant `"global"` key; `unlimited()` never delays or errors; `acquire_global` shares one bucket; pre-cancelled tokens short-circuit before sleeping the limiter interval | Conforms |
| Error classifier | `NetworkErrorKind::from_transport_error_class` is total across every `TransportErrorClass` variant including `Redirect` and `Upgrade` through the wildcard arm; the optional reqwest classifier maps real `reqwest::Error` shapes into the same partition | Conforms |
| Panic-free posture | The `Retry-After` IMF-fixdate civil-day arithmetic promotes every intermediate to `i64` so an attacker-controlled out-of-range year cannot overflow the retry loop; documented panic-allowlist entries on `jitter.rs::bounded_offset` and `transport-policy/src/policy.rs` static-UA constructors stay justified | Conforms |

## Current Contract

### Retry-After Parser

`parse_retry_after(value, now)` is the only public entry point on
`retry_after.rs`. Every helper inside (`parse_http_date`, `parse_http_month`,
`parse_http_time`, `days_from_civil`, `days_in_month`, `is_leap_year`,
`unix_timestamp`) is private and is exercised exclusively through the public
boundary. Delta-seconds inputs are accepted when the trimmed value is composed
solely of ASCII digits; surrounding whitespace is trimmed before dispatch.
IMF-fixdate inputs are validated against the documented format
(`<weekday>, <day> <Mmm> <year> <HH>:<MM>:<SS> GMT`). Past or epoch-equal
dates clamp to `Duration::ZERO`. Civil-day arithmetic promotes every
intermediate to `i64` so out-of-range year values cannot overflow.

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

## Evidence

Primary implementation points:

- `crates/transport-policy/src/retry_after.rs`
- `crates/transport-policy/src/jitter.rs`
- `crates/transport-policy/src/retry.rs`
- `crates/transport-policy/src/rate_limit.rs`
- `crates/transport-policy/src/classify.rs`
- `crates/transport-policy/src/policy.rs`
- `crates/transport-policy/src/status.rs`
- `crates/transport-policy/src/time.rs`

Primary regression coverage:

- `crates/transport-policy/tests/retry_after_contract.rs`
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
- `fuzz/fuzz_targets/fuzz_parse_retry_after.rs`, `fuzz/corpus/fuzz_parse_retry_after/`
- `fuzz/fuzz_targets/fuzz_retry_policy_delay.rs`, `fuzz/corpus/fuzz_retry_policy_delay/`
- `fuzz/fuzz_targets/fuzz_jitter_delay_for_attempt.rs`, `fuzz/corpus/fuzz_jitter_delay_for_attempt/`

Validation surface:

```text
cargo fmt --all --check
cargo clippy -p cow-sdk-transport-policy --all-targets --all-features -- -D warnings
cargo test -p cow-sdk-transport-policy --all-features
cargo llvm-cov -p cow-sdk-transport-policy --all-features --summary-only --fail-under-lines 85
RUSTDOCFLAGS="-D warnings" cargo doc -p cow-sdk-transport-policy --no-deps
```
