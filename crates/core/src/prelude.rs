//! Convenience prelude bringing the canonical cow identity newtypes into
//! scope.
//!
//! The prelude re-exports the six cow primitive newtypes
//! ([`Address`], [`AppDataHash`], [`Amount`], [`Hash32`], [`HexData`],
//! [`OrderUid`]). Each is a strict `#[repr(transparent)]` wrapper around
//! the corresponding `alloy_primitives` type per ADR 0052, carrying a
//! cow-owned accessor surface that lets callers move between the cow
//! newtype and the underlying alloy primitive at zero runtime cost:
//!
//! - The four byte-typed newtypes ([`Address`], [`Hash32`],
//!   [`HexData`], [`OrderUid`]) and [`AppDataHash`] expose
//!   `new`, `from_bytes`, `to_hex_string`, `write_into`, `as_slice`,
//!   `as_alloy`, `into_alloy`, `zero`, `is_zero`, and `byte_length`.
//! - [`Amount`] wraps [`alloy_primitives::U256`] and exposes `new`,
//!   `from_units` / `parse_units` / `format_units` (exact numeric and
//!   decimal token-amount I/O), `from_u256`, `as_u256`, `into_u256`,
//!   `zero`, `is_zero`, `checked_*`, and `saturating_*`.

pub use crate::types::{Address, Amount, AppDataHash, Hash32, HexData, OrderUid};

/// The prelude pairs [`Address`] with its literal macro [`address!`](crate::address),
/// matching std's `vec!`-beside-`Vec` prelude precedent, so importing the
/// prelude is enough to write compile-time validated address constants.
pub use crate::address;
