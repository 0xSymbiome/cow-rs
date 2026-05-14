#![no_main]

//! Fuzz target for the orderbook total-fee transform.
//!
//! **Surface:** `cow_sdk_orderbook::calculate_total_fee`.
//! **Property:** `PROP-ORD-005`.
//! **Seed contract:** corpus inputs cover the canonical executed-fee
//! decimal-string shape pinned by
//! `parity/fixtures/orderbook.json::orderbook-total-fee-transform`,
//! empty-input and leading-zero boundaries, and adversarial non-digit
//! and uint256-overflow inputs.
//! **Corpus README:** `../corpus/fuzz_calculate_total_fee/README.md`.
//!
//! The target feeds arbitrary bytes through `std::str::from_utf8` into
//! `Option<&str>` and asserts the function's documented invariants:
//!
//! * Never panics for any input.
//! * `Err(...)` is returned iff the input string is empty or carries any
//!   non-ASCII-digit byte (the `validate_decimal` precondition).
//! * For every `Ok(amount)` the rendered `amount.to_string()` equals the
//!   input with leading zeros stripped (a value of `"0"` is kept as a
//!   single zero).
//! * Determinism: identical input always produces identical output.

use cow_sdk_orderbook::{OrderbookError, calculate_total_fee};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let candidate: Option<&str> = std::str::from_utf8(data).ok();
    let first = calculate_total_fee(candidate);

    // Determinism: identical input must produce identical output.
    let second = calculate_total_fee(candidate);
    match (&first, &second) {
        (Ok(a), Ok(b)) => assert_eq!(
            a.to_string(),
            b.to_string(),
            "calculate_total_fee must be deterministic on identical input",
        ),
        (Err(_), Err(_)) => {}
        _ => panic!("calculate_total_fee must be deterministic on identical input"),
    }

    let Some(input) = candidate else {
        // Non-UTF-8 input cannot reach `calculate_total_fee`, but we still
        // exercised the helper with `None`. The `None` case treats the
        // missing executed fee as `"0"` and must succeed.
        let zero =
            calculate_total_fee(None).expect("calculate_total_fee(None) must succeed as zero");
        assert_eq!(zero.to_string(), "0", "missing executed fee must render as zero");
        return;
    };

    let only_digits = !input.is_empty() && input.bytes().all(|b| b.is_ascii_digit());

    match first {
        Ok(amount) => {
            // Precondition was satisfied: input is non-empty, all ASCII digits.
            assert!(
                only_digits,
                "calculate_total_fee returned Ok for non-digit input: {input:?}",
            );

            let rendered = amount.to_string();
            let expected = trim_leading_zeroes_reference(input);
            assert_eq!(
                rendered, expected,
                "Ok(amount) must render with leading zeros stripped",
            );
        }
        Err(error) => {
            // The documented failure mode is `InvalidTransform` — both the
            // `validate_decimal` rejection path and the `Amount::new` overflow
            // fallback collapse to the same typed variant.
            assert!(
                matches!(error, OrderbookError::InvalidTransform { .. }),
                "calculate_total_fee must surface InvalidTransform on rejection: got {error:?}",
            );
        }
    }
});

/// Reference implementation of `trim_leading_zeroes` used in the
/// orderbook transform: strips ASCII `'0'` runs from the front but
/// preserves a single zero when every input byte was zero.
fn trim_leading_zeroes_reference(value: &str) -> String {
    let trimmed = value.trim_start_matches('0');
    if trimmed.is_empty() {
        "0".to_owned()
    } else {
        trimmed.to_owned()
    }
}
