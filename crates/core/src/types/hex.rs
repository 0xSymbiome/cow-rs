/// Hex character count for an EVM address without the `0x` prefix.
pub const EVM_ADDRESS_HEX_CHARS: usize = 40;
/// Hex character count for a 32-byte app-data hash without the `0x` prefix.
pub const APP_DATA_HASH_HEX_CHARS: usize = 64;
/// Hex character count for an order UID without the `0x` prefix.
pub const ORDER_UID_HEX_CHARS: usize = 112;
/// Hex character count for a 32-byte hash without the `0x` prefix.
pub const HASH32_HEX_CHARS: usize = 64;
/// Maximum bit width accepted for unsigned protocol quantities.
pub const U256_MAX_BITS: u64 = 256;

const HEX_ALPHABET: [u8; 16] = *b"0123456789abcdef";

/// Returns the canonical `0x`-prefixed lowercase hex encoding of a 20-byte input.
#[must_use]
pub const fn hex_encode_20(bytes: [u8; 20]) -> [u8; 42] {
    let mut out = [0u8; 42];
    out[0] = b'0';
    out[1] = b'x';
    let mut i = 0;
    while i < 20 {
        let byte = bytes[i];
        out[2 + 2 * i] = HEX_ALPHABET[(byte >> 4) as usize];
        out[2 + 2 * i + 1] = HEX_ALPHABET[(byte & 0x0F) as usize];
        i += 1;
    }
    out
}

/// Returns the canonical `0x`-prefixed lowercase hex encoding of a 32-byte input.
#[must_use]
pub const fn hex_encode_32(bytes: [u8; 32]) -> [u8; 66] {
    let mut out = [0u8; 66];
    out[0] = b'0';
    out[1] = b'x';
    let mut i = 0;
    while i < 32 {
        let byte = bytes[i];
        out[2 + 2 * i] = HEX_ALPHABET[(byte >> 4) as usize];
        out[2 + 2 * i + 1] = HEX_ALPHABET[(byte & 0x0F) as usize];
        i += 1;
    }
    out
}

/// Returns the canonical `0x`-prefixed lowercase hex encoding of a 56-byte input.
#[must_use]
pub const fn hex_encode_56(bytes: [u8; 56]) -> [u8; 114] {
    let mut out = [0u8; 114];
    out[0] = b'0';
    out[1] = b'x';
    let mut i = 0;
    while i < 56 {
        let byte = bytes[i];
        out[2 + 2 * i] = HEX_ALPHABET[(byte >> 4) as usize];
        out[2 + 2 * i + 1] = HEX_ALPHABET[(byte & 0x0F) as usize];
        i += 1;
    }
    out
}

/// Decodes a `0x`-prefixed hex string literal into a fixed-length byte array at compile time.
///
/// Intended for converting the embedded protocol-address hex literals to their
/// raw byte form inside `const` initialisers.
///
/// # Panics
///
/// Panics at compile time when the input is not exactly 42 characters long,
/// is missing the `0x` prefix, or contains a non-hex character.
#[must_use]
pub const fn hex_decode_20(hex: &str) -> [u8; 20] {
    let bytes = hex.as_bytes();
    assert!(
        bytes.len() == 42,
        "hex_decode_20 requires a 42-character input"
    );
    assert!(
        bytes[0] == b'0' && bytes[1] == b'x',
        "hex_decode_20 requires a 0x prefix"
    );
    let mut out = [0u8; 20];
    let mut i = 0;
    while i < 20 {
        out[i] = (decode_nibble(bytes[2 + 2 * i]) << 4) | decode_nibble(bytes[2 + 2 * i + 1]);
        i += 1;
    }
    out
}

/// Decodes a `0x`-prefixed 32-byte hex string literal at compile time.
///
/// # Panics
///
/// Panics at compile time when the input is not exactly 66 characters long,
/// is missing the `0x` prefix, or contains a non-hex character.
#[must_use]
pub const fn hex_decode_32(hex: &str) -> [u8; 32] {
    let bytes = hex.as_bytes();
    assert!(
        bytes.len() == 66,
        "hex_decode_32 requires a 66-character input"
    );
    assert!(
        bytes[0] == b'0' && bytes[1] == b'x',
        "hex_decode_32 requires a 0x prefix"
    );
    let mut out = [0u8; 32];
    let mut i = 0;
    while i < 32 {
        out[i] = (decode_nibble(bytes[2 + 2 * i]) << 4) | decode_nibble(bytes[2 + 2 * i + 1]);
        i += 1;
    }
    out
}

/// Decodes a `0x`-prefixed 56-byte hex string literal at compile time.
///
/// # Panics
///
/// Panics at compile time when the input is not exactly 114 characters long,
/// is missing the `0x` prefix, or contains a non-hex character.
#[must_use]
pub const fn hex_decode_56(hex: &str) -> [u8; 56] {
    let bytes = hex.as_bytes();
    assert!(
        bytes.len() == 114,
        "hex_decode_56 requires a 114-character input"
    );
    assert!(
        bytes[0] == b'0' && bytes[1] == b'x',
        "hex_decode_56 requires a 0x prefix"
    );
    let mut out = [0u8; 56];
    let mut i = 0;
    while i < 56 {
        out[i] = (decode_nibble(bytes[2 + 2 * i]) << 4) | decode_nibble(bytes[2 + 2 * i + 1]);
        i += 1;
    }
    out
}

/// Decodes one ASCII hex nibble for compile-time hex literal helpers.
///
/// # Panics
///
/// Panics when `c` is not an ASCII hex digit. Public compile-time decoders
/// document that invalid embedded literals fail during constant evaluation.
const fn decode_nibble(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        b'A'..=b'F' => c - b'A' + 10,
        // SAFETY: this is the deliberate assertion used by the const hex
        // decoders to reject non-hex crate-owned literals at compile time.
        _ => panic!("hex nibble must be 0-9, a-f, or A-F"),
    }
}
