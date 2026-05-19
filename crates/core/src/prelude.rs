//! Convenience prelude bringing the canonical cow identity newtypes into
//! scope.
//!
//! The prelude re-exports the four byte-typed cow newtypes
//! ([`Address`], [`Hash32`], [`HexData`], [`OrderUid`]). Each is a
//! strict `#[repr(transparent)]` wrapper around the corresponding
//! `alloy_primitives` type per ADR 0052,
//! carrying the canonical accessor surface (`new`, `from_bytes`,
//! `to_hex_string`, `write_into`, `as_slice`, `as_alloy`, `into_alloy`,
//! `zero`, `is_zero`, `byte_length`) as inherent methods on the newtype.
//!
//! `AppDataHash`, `Amount`, and `SignedAmount` will join this re-export
//! hub when their strict-newtype migration lands in a later cascade
//! boundary.

pub use crate::types::{Address, Hash32, HexData, OrderUid};
