//! HTTP transport injection point shared across `cow-sdk` crates.
//!
//! The [`HttpTransport`] trait is the production seam that downstream clients
//! use to dispatch REST requests without committing to a concrete backend.
//! Every method is `async` so implementations can bridge either a native
//! runtime (through [`ReqwestTransport`]) or a browser runtime (through a
//! `JsFuture`-backed adapter in `cow-sdk-transport-wasm`).
//!
//! The companion [`TransportError`] enum is the typed failure surface for
//! transport adapters. Native adapters that bridge `reqwest::Error` classify
//! each failure through [`TransportErrorClass`] before wrapping and call
//! [`reqwest::Error::without_url`] to keep endpoint URLs out of the error
//! text. Callers that want to partition telemetry or shape retry policy on
//! the failure category match on the [`class`](TransportError) of the
//! [`TransportError::Transport`] variant.

mod http;

#[cfg(not(target_arch = "wasm32"))]
pub mod reqwest;

pub use http::{HttpTransport, TransportError};

#[cfg(not(target_arch = "wasm32"))]
pub use self::reqwest::{ReqwestTransport, ReqwestTransportConfig, classify_reqwest_error};

pub use crate::validation::TransportErrorClass;
