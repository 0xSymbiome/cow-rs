//! Redaction wrappers and response-body sanitization helpers for
//! secret-bearing SDK surfaces.
//!
//! `Redacted<T>` and the URL-map wrappers keep the inner value available for
//! deliberate use while preventing accidental leakage through
//! [`std::fmt::Debug`], [`std::fmt::Display`], and [`serde::Serialize`].
//! Every redacted value representation emits the literal string `"[redacted]"`
//! regardless of the wrapped payload. [`Redacted::into_inner`],
//! [`Redacted::as_inner`], [`RedactedUrlMap::as_inner`], and
//! [`RedactedOptionalUrlMap::as_inner`] surface the underlying value when the
//! caller has explicit intent. [`redact_response_body`] sanitizes
//! credential-shaped response snippets before they enter public diagnostics.

mod body;
mod wrappers;

pub use self::body::*;
pub use self::wrappers::*;

/// The placeholder emitted in every redacted representation.
pub const REDACTED_PLACEHOLDER: &str = "[redacted]";
