#![no_main]

//! Fuzz target for the retry jitter strategy.
//!
//! **Surface:** `cow_sdk_transport_policy::JitterStrategy::delay_for_attempt`.
//! **Property:** `PROP-TPP-004`.
//! **Seed contract:** corpus inputs cover canonical decorrelated, full, and
//! equal jitter shapes; boundary zero / equal base+max windows; and
//! adversarial extreme attempt counts and noise inputs that perturb the
//! deterministic offset generator.
//!
//! The target maps arbitrary bytes through `Arbitrary` into a typed
//! `JitterInput`, walks every `JitterStrategy` variant against the supplied
//! base/max delays and attempt index, and asserts: every returned delay is
//! bounded by `max_delay`, results are deterministic for identical inputs,
//! and no variant panics on extreme attempt indices.

use std::time::Duration;

use cow_sdk_transport_policy::JitterStrategy;
use libfuzzer_sys::{
    arbitrary::{Arbitrary, Unstructured},
    fuzz_target,
};

const MAX_ATTEMPT_INDEX: usize = 1024;
const MAX_BASE_MS: u64 = 600_000;
const MAX_MAX_MS: u64 = 1_200_000;

#[derive(Debug)]
struct JitterInput {
    seed: u64,
    attempt: usize,
    base_ms: u64,
    max_ms: u64,
    strategy_tag: u8,
}

impl<'a> Arbitrary<'a> for JitterInput {
    fn arbitrary(bytes: &mut Unstructured<'a>) -> libfuzzer_sys::arbitrary::Result<Self> {
        let attempt = usize::from(read_u16(bytes, 1)) % (MAX_ATTEMPT_INDEX + 1);
        let base_ms = read_u64(bytes, 50) % (MAX_BASE_MS + 1);
        let max_ms = base_ms.saturating_add(read_u64(bytes, 0) % (MAX_MAX_MS + 1));
        Ok(Self {
            seed: read_u64(bytes, 0),
            attempt,
            base_ms,
            max_ms,
            strategy_tag: read_u8(bytes, 0),
        })
    }
}

fuzz_target!(|input: JitterInput| {
    let base = Duration::from_millis(input.base_ms);
    let max = Duration::from_millis(input.max_ms);

    let strategy = jitter_for_tag(input.strategy_tag, input.seed);

    let first = strategy.delay_for_attempt(base, max, input.attempt);
    let second = strategy.delay_for_attempt(base, max, input.attempt);
    assert_eq!(
        first, second,
        "JitterStrategy::delay_for_attempt must be deterministic on identical input",
    );

    assert!(
        first <= max,
        "JitterStrategy::delay_for_attempt must respect the configured max_delay",
    );

    // Walk every documented variant to ensure no variant panics for the
    // supplied input shape; assert each variant respects `max_delay`.
    for variant in [
        JitterStrategy::none(),
        JitterStrategy::full_from_seed(input.seed),
        JitterStrategy::equal_from_seed(input.seed),
        JitterStrategy::decorrelated_from_seed(input.seed),
    ] {
        let delay = variant.delay_for_attempt(base, max, input.attempt);
        assert!(
            delay <= max,
            "every documented JitterStrategy variant must respect max_delay",
        );
    }

    // Zero-base inputs must collapse to zero rather than panicking on the
    // documented window-division branch.
    let zero_delay = strategy.delay_for_attempt(Duration::ZERO, max, input.attempt);
    assert!(
        zero_delay <= max,
        "JitterStrategy::delay_for_attempt must respect max_delay even with zero base",
    );
});

fn jitter_for_tag(tag: u8, seed: u64) -> JitterStrategy {
    match tag % 4 {
        0 => JitterStrategy::none(),
        1 => JitterStrategy::full_from_seed(seed),
        2 => JitterStrategy::equal_from_seed(seed),
        _ => JitterStrategy::decorrelated_from_seed(seed),
    }
}

fn read_u8(bytes: &mut Unstructured<'_>, default: u8) -> u8 {
    u8::arbitrary(bytes).unwrap_or(default)
}

fn read_u16(bytes: &mut Unstructured<'_>, default: u16) -> u16 {
    u16::arbitrary(bytes).unwrap_or(default)
}

fn read_u64(bytes: &mut Unstructured<'_>, default: u64) -> u64 {
    u64::arbitrary(bytes).unwrap_or(default)
}
