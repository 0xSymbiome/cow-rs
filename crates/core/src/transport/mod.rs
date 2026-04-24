//! HTTP transport injection point shared across `cow-sdk` crates.
//!
//! The [`HttpTransport`] trait is the production seam that downstream clients
//! use to dispatch REST requests without committing to a concrete backend.
//! Every method is `async` so implementations can bridge either a native
//! runtime (through [`ReqwestTransport`]) or a browser runtime (through a
//! `JsFuture`-backed adapter in `cow-sdk-transport-wasm`).
//!
//! Every method carries the per-call header set and an optional per-call
//! timeout so typed consumers compose one injection point without holding a
//! parallel HTTP client for header or deadline overrides. Adapters surface
//! non-2xx responses through [`TransportError::HttpStatus`] so the calling
//! layer receives the numeric status, response headers, and raw response body
//! together through the typed error channel.
//!
//! The companion [`TransportError`] enum is the typed failure surface for
//! transport adapters. Native adapters that bridge `reqwest::Error` classify
//! each failure through [`TransportErrorClass`] before wrapping and call
//! [`reqwest::Error::without_url`] to keep endpoint URLs out of the error
//! text. Callers that want to partition telemetry or shape retry policy on
//! the failure category match on the [`class`](TransportError::class) of the
//! [`TransportError::Transport`] variant; callers that need the numeric
//! HTTP status on a non-success response match on
//! [`TransportError::HttpStatus`].

mod error;
mod http;

#[cfg(not(target_arch = "wasm32"))]
pub mod reqwest;

pub use error::TransportError;
pub use http::HttpTransport;

#[cfg(not(target_arch = "wasm32"))]
pub use self::reqwest::{ReqwestTransport, ReqwestTransportConfig, classify_reqwest_error};

pub use crate::validation::TransportErrorClass;
