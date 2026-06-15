#![no_main]

//! Fuzz target for the app-data size-limit warning and rejection thresholds.
//!
//! **Surface:** `cow_sdk_app_data::app_data_info` with public constants
//! `cow_sdk_app_data::{APP_DATA_APPROACHING_LIMIT_RATIO, APP_DATA_MAX_BYTES}`.
//! The private `approaching_size_limit(bytes_used, max_bytes)` helper is
//! exercised through this public wrapper, which is the documented surface
//! for the `AppDataValidation` warning channel and the
//! `AppDataError::TooLarge` rejection path.
//! **Property:** `PROP-APP-004`.
//! **Seed contract:** corpus inputs cover canonical sub-limit documents,
//! boundary documents at exactly the 75%-of-max warning threshold and
//! exactly the hard ceiling, and adversarial documents that overshoot the
//! ceiling (must reject with `TooLarge`) and one that is well below the
//! threshold (must surface zero warnings).
//!
//! The target invariants are:
//!
//! * `app_data_info` never panics for any padding length, including
//!   padding that pushes the rendered document past the configured ceiling
//!   (the path must surface a typed `TooLarge` error).
//! * The `ApproachingSizeLimit` warning fires iff the rendered byte size
//!   reaches or exceeds the documented `0.75 * APP_DATA_MAX_BYTES` floor.
//! * The hard `TooLarge` rejection fires iff the rendered byte size strictly
//!   exceeds `APP_DATA_MAX_BYTES`.
//! * The renderer-and-validator pipeline is deterministic on identical input.

use cow_sdk_app_data::{
    APP_DATA_APPROACHING_LIMIT_RATIO, APP_DATA_MAX_BYTES, AppDataError, AppDataWarning,
    app_data_info,
};
use libfuzzer_sys::fuzz_target;
use serde_json::{Value, json};

/// Maximum input width accepted by the target. The two input bytes drive a
/// padding-length selector; anything beyond is ignored so the fuzzer stays
/// bounded and the iteration cost stays low.
const MAX_FUZZ_INPUT: usize = 2;

/// Padding-length cap that lets the fuzzer explore both the warning and the
/// hard-ceiling boundaries. The cap is just above `APP_DATA_MAX_BYTES` so
/// at-limit and over-limit shapes both appear.
const MAX_PAD_LEN: usize = APP_DATA_MAX_BYTES + 64;

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len().min(MAX_FUZZ_INPUT)];

    // Two bytes encode a 0..=MAX_PAD_LEN padding-string length. The padding
    // is appended to a documented minimal app-data document so the rendered
    // canonical JSON byte size scales linearly with `pad_len`.
    let pad_len = if data.len() >= 2 {
        ((usize::from(data[0]) << 8) | usize::from(data[1])) % (MAX_PAD_LEN + 1)
    } else if data.len() == 1 {
        usize::from(data[0]) * 32
    } else {
        0
    };

    let padding = "a".repeat(pad_len);
    // `appCode` is an unconstrained-length string in the v1.14.0 schema so
    // padding here scales the rendered canonical JSON byte size linearly
    // while every other field is left at the minimal valid shape required
    // by the embedded schema validator.
    let document: Value = json!({
        "appCode": padding,
        "version": "1.14.0",
        "metadata": {},
    });

    let first = app_data_info(&document);
    let second = app_data_info(&document);

    // Determinism: same input must classify the same way.
    assert_eq!(
        first.is_ok(),
        second.is_ok(),
        "app_data_info must be deterministic on identical input",
    );

    let expected_threshold = expected_threshold(APP_DATA_MAX_BYTES);

    match (first, second) {
        (Ok(left), Ok(right)) => {
            assert_eq!(
                left.validation.bytes_used, right.validation.bytes_used,
                "bytes_used must be deterministic on identical input",
            );
            assert_eq!(
                left.validation.warnings, right.validation.warnings,
                "warnings list must be deterministic on identical input",
            );

            let bytes_used = left.validation.bytes_used;
            // Documented ceiling: Ok path implies bytes_used <= APP_DATA_MAX_BYTES.
            assert!(
                bytes_used <= APP_DATA_MAX_BYTES,
                "Ok path must never carry bytes_used > APP_DATA_MAX_BYTES (got {bytes_used})",
            );

            let warning_emitted = left.validation.warnings.iter().any(|warning| {
                matches!(
                    warning,
                    AppDataWarning::ApproachingSizeLimit { bytes_used: w_used, max_bytes }
                        if *w_used == bytes_used && *max_bytes == APP_DATA_MAX_BYTES
                )
            });
            let expected_warning = bytes_used >= expected_threshold;
            assert_eq!(
                warning_emitted, expected_warning,
                "ApproachingSizeLimit warning must fire iff bytes_used ({bytes_used}) >= floor({APP_DATA_APPROACHING_LIMIT_RATIO} * {APP_DATA_MAX_BYTES}) = {expected_threshold}",
            );
        }
        (Err(left), Err(right)) => {
            // Determinism on the error path: same kind of error.
            assert_eq!(
                std::mem::discriminant(&left),
                std::mem::discriminant(&right),
                "app_data_info error variant must be deterministic on identical input",
            );

            // If the error is TooLarge, its declared actual_bytes must exceed
            // APP_DATA_MAX_BYTES per the documented contract.
            if let AppDataError::TooLarge {
                actual_bytes,
                max_bytes,
            } = left
            {
                assert!(
                    actual_bytes > max_bytes,
                    "TooLarge must carry actual_bytes ({actual_bytes}) > max_bytes ({max_bytes})",
                );
                assert_eq!(
                    max_bytes, APP_DATA_MAX_BYTES,
                    "TooLarge max_bytes must equal the documented APP_DATA_MAX_BYTES",
                );
            }
        }
        _ => {
            panic!(
                "app_data_info Ok/Err classification must be deterministic on identical input"
            );
        }
    }
});

/// Reference implementation of the documented warning threshold so the
/// target asserts the public contract independently of the crate's private
/// helper. Matches the cast sequence in `approaching_size_limit`.
fn expected_threshold(max_bytes: usize) -> usize {
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    let floor = (max_bytes as f64 * APP_DATA_APPROACHING_LIMIT_RATIO) as usize;
    floor
}
