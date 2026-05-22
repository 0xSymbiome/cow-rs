# cow-sdk-transport-policy

Shared retry, rate-limit, `Retry-After`, jitter, and transport classification
policy for CoW Protocol SDK HTTP clients.

The crate is target-neutral. Native builds use `futures-timer` for retry sleeps;
`wasm32` builds use `gloo-timers`. Orderbook and subgraph clients consume the
same `TransportPolicy` so callers can configure HTTP timeout, user-agent,
retry attempts, backoff jitter, and limiter scope consistently.

```rust
use std::time::Duration;

use cow_sdk_transport_policy::{JitterStrategy, RetryPolicy, TransportPolicy};

let retry = RetryPolicy::builder()
    .max_attempts(4)
    .base_delay(Duration::from_millis(100))
    .jitter(JitterStrategy::decorrelated_from_seed(7))
    .build();

let policy = TransportPolicy::default_orderbook().with_retry(retry);

assert_eq!(policy.retry().max_attempts(), 4);
```

`Retry-After` HTTP-date header parsing routes through
`httpdate::parse_http_date` per RFC 7231, accepting the IMF-fixdate,
legacy RFC 850, and ANSI C `asctime` date forms.
