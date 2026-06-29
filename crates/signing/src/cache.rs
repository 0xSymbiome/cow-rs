//! Caching seam for EIP-1271 signature verification.
//!
//! Re-exports the [`Eip1271Cache`] trait and the always-available
//! [`NoopEip1271Cache`] from `cow-sdk-contracts`, so signing consumers reach the
//! seam through one module. To memoize verification, implement the two-method
//! trait over a store of your choice and pass it to
//! [`cow_sdk_contracts::verify_eip1271_signature_cached`]; the recording policy
//! (positive-only, keyed on `(verifier, digest, signature_hash)`) is documented
//! on [`Eip1271Cache`].

pub use cow_sdk_contracts::{Eip1271Cache, NoopEip1271Cache};
