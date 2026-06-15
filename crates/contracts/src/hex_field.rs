//! Hex decode helpers for `0x`-prefixed payloads inside the contracts
//! boundary.
//!
//! The two functions in this module decode `0x`-prefixed hexadecimal
//! strings into raw bytes. They raise typed [`ContractsError`]
//! variants that carry a `&'static str` `field` discriminator so the
//! caller can identify which input failed validation:
//!
//! - [`ContractsError::InvalidHexPrefix`] when `value` is not
//!   `0x`-prefixed.
//! - [`ContractsError::DecodeHex`] when the payload contains non-hex
//!   characters or has odd length. The underlying
//!   [`alloy_primitives::hex::FromHexError`] is preserved through
//!   `#[source]` so consumers can introspect the exact decoder
//!   failure (`OddLength`, `InvalidHexCharacter`, etc.).
//! - [`ContractsError::InvalidDecodedLength`] (exact-length variant
//!   only) when the decoded byte length does not match the const
//!   generic `N`.
//!
//! Use [`crate::hex_field::decode_hex_field`] when the caller validates
//! the length itself, and [`crate::hex_field::decode_hex_field_exact`]
//! when the length is known at compile time and a `[u8; N]` return is
//! preferred.

use crate::errors::ContractsError;

/// Decodes a `0x`-prefixed hexadecimal string into raw bytes.
///
/// The decoder is case-insensitive and does **not** pad odd-length
/// payloads: the underlying [`alloy_primitives::hex::decode`] rejects
/// odd-length input by raising
/// [`alloy_primitives::hex::FromHexError::OddLength`], which surfaces
/// through the [`ContractsError::DecodeHex`] source chain.
///
/// The byte length of the decoded payload is **not** validated by
/// this helper. Callers that need a length check should use
/// [`decode_hex_field_exact`] (which moves the length check into the
/// return type) or apply their own validation at the call site.
///
/// # Errors
///
/// - [`ContractsError::InvalidHexPrefix`] when `value` is not
///   `0x`-prefixed.
/// - [`ContractsError::DecodeHex`] when the payload contains non-hex
///   characters or has odd length. The underlying decoder error is
///   preserved through `#[source]`.
#[must_use = "decoded bytes carry the only signal of decode success"]
pub fn decode_hex_field(field: &'static str, value: &str) -> Result<Vec<u8>, ContractsError> {
    let stripped = value
        .strip_prefix("0x")
        .ok_or(ContractsError::InvalidHexPrefix { field })?;
    alloy_primitives::hex::decode(stripped)
        .map_err(|source| ContractsError::DecodeHex { field, source })
}

/// Decodes a `0x`-prefixed hexadecimal string into raw bytes, refusing any
/// payload whose decoded length would exceed `max_decoded_bytes`.
///
/// The bound is checked against the encoded length **before** the decoder
/// allocates, so an oversized payload is rejected without first materializing
/// the byte buffer. This guards decode-time allocation for inputs that do not
/// arrive through the response-capped transport, such as fixtures, fuzz
/// inputs, or third-party callers deserializing data of unknown origin.
///
/// # Errors
///
/// - [`ContractsError::InvalidHexPrefix`] when `value` is not `0x`-prefixed.
/// - [`ContractsError::FieldTooLarge`] when the payload would decode to more
///   than `max_decoded_bytes` bytes.
/// - [`ContractsError::DecodeHex`] when the payload contains non-hex
///   characters or has odd length.
#[must_use = "decoded bytes carry the only signal of decode success"]
pub fn decode_hex_field_bounded(
    field: &'static str,
    value: &str,
    max_decoded_bytes: usize,
) -> Result<Vec<u8>, ContractsError> {
    let stripped = value
        .strip_prefix("0x")
        .ok_or(ContractsError::InvalidHexPrefix { field })?;
    // Two hex characters encode one byte, so reject before allocating when the
    // encoded length already exceeds the decoded-byte budget.
    if stripped.len() > max_decoded_bytes.saturating_mul(2) {
        return Err(ContractsError::FieldTooLarge {
            field,
            max_bytes: max_decoded_bytes,
        });
    }
    alloy_primitives::hex::decode(stripped)
        .map_err(|source| ContractsError::DecodeHex { field, source })
}

/// Decodes a `0x`-prefixed hexadecimal string into a fixed-size byte
/// array.
///
/// The const generic `N` is the required decoded byte length. The
/// helper raises [`ContractsError::InvalidDecodedLength`] when the
/// decoded payload does not match `N`, so callers receive a typed
/// `[u8; N]` rather than a `Vec<u8>` that still needs a runtime
/// length check.
///
/// # Errors
///
/// - [`ContractsError::InvalidHexPrefix`] when `value` is not
///   `0x`-prefixed.
/// - [`ContractsError::DecodeHex`] when the payload contains non-hex
///   characters or has odd length.
/// - [`ContractsError::InvalidDecodedLength`] when the decoded byte
///   length does not match `N`.
#[must_use = "decoded byte array carries the only signal of decode success"]
pub fn decode_hex_field_exact<const N: usize>(
    field: &'static str,
    value: &str,
) -> Result<[u8; N], ContractsError> {
    let bytes = decode_hex_field(field, value)?;
    let actual = bytes.len();
    <[u8; N]>::try_from(bytes).map_err(|_| ContractsError::InvalidDecodedLength {
        field,
        expected: N,
        actual,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_hex_field_accepts_lowercase_payload() {
        let bytes = decode_hex_field("field", "0xdeadbeef").unwrap();
        assert_eq!(bytes, vec![0xde, 0xad, 0xbe, 0xef]);
    }

    #[test]
    fn decode_hex_field_accepts_mixed_case_payload() {
        let bytes = decode_hex_field("field", "0xDeAdBeEf").unwrap();
        assert_eq!(bytes, vec![0xde, 0xad, 0xbe, 0xef]);
    }

    #[test]
    fn decode_hex_field_rejects_missing_prefix() {
        let error = decode_hex_field("appData", "deadbeef").unwrap_err();
        assert!(matches!(
            error,
            ContractsError::InvalidHexPrefix { field: "appData" }
        ));
    }

    #[test]
    fn decode_hex_field_rejects_odd_length() {
        let error = decode_hex_field("appData", "0xdeadbee").unwrap_err();
        let ContractsError::DecodeHex { field, source } = error else {
            panic!("expected DecodeHex variant");
        };
        assert_eq!(field, "appData");
        assert!(matches!(
            source,
            alloy_primitives::hex::FromHexError::OddLength
        ));
    }

    #[test]
    fn decode_hex_field_rejects_non_hex_characters() {
        let error = decode_hex_field("appData", "0xzzzz").unwrap_err();
        assert!(matches!(
            error,
            ContractsError::DecodeHex {
                field: "appData",
                ..
            }
        ));
    }

    #[test]
    fn decode_hex_field_exact_returns_array_for_matching_length() {
        let bytes: [u8; 4] = decode_hex_field_exact("magicValue", "0xdeadbeef").unwrap();
        assert_eq!(bytes, [0xde, 0xad, 0xbe, 0xef]);
    }

    #[test]
    fn decode_hex_field_exact_rejects_wrong_length() {
        let error = decode_hex_field_exact::<4>("magicValue", "0xdeadbe").unwrap_err();
        assert!(matches!(
            error,
            ContractsError::InvalidDecodedLength {
                field: "magicValue",
                expected: 4,
                actual: 3,
            }
        ));
    }

    #[test]
    fn decode_hex_field_exact_propagates_prefix_error() {
        let error = decode_hex_field_exact::<32>("storageSlot", "deadbeef").unwrap_err();
        assert!(matches!(
            error,
            ContractsError::InvalidHexPrefix {
                field: "storageSlot"
            }
        ));
    }

    #[test]
    fn decode_hex_field_bounded_accepts_payload_at_the_limit() {
        // Two hex characters per byte, so a 4-byte limit accepts an 8-char
        // payload exactly.
        let bytes = decode_hex_field_bounded("signature", "0xdeadbeef", 4).unwrap();
        assert_eq!(bytes, vec![0xde, 0xad, 0xbe, 0xef]);
    }

    #[test]
    fn decode_hex_field_bounded_rejects_payload_over_the_limit() {
        // Five decoded bytes exceed the four-byte limit and are refused before
        // the decoder allocates.
        let error = decode_hex_field_bounded("signature", "0xdeadbeef00", 4).unwrap_err();
        assert!(matches!(
            error,
            ContractsError::FieldTooLarge {
                field: "signature",
                max_bytes: 4,
            }
        ));
    }

    #[test]
    fn decode_hex_field_bounded_still_requires_the_prefix() {
        let error = decode_hex_field_bounded("signature", "deadbeef", 4).unwrap_err();
        assert!(matches!(
            error,
            ContractsError::InvalidHexPrefix { field: "signature" }
        ));
    }
}
