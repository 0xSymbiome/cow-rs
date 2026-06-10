//! Environment, address-book, and HTTP client policy types shared across crates.

use std::time::Duration;

pub use self::{chains::*, env::*, hosts::*, http::*, protocol::*};

mod chains;
mod env;
mod hosts;
mod http;
mod protocol;

/// Sentinel address used by `CoW` Protocol to represent the native chain asset.
///
/// Stored in the canonical lowercase 0x-prefixed wire form per PROP-WB-004 so it
/// compares byte-identically against any [`Address`](crate::Address) constructed
/// from the same logical value; alloy address checksum casing is parsed and
/// normalized by the cow newtype at construction.
pub const EVM_NATIVE_CURRENCY_ADDRESS: &str = "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
/// Default timeout applied to HTTP-backed SDK clients.
pub const DEFAULT_HTTP_TIMEOUT: Duration = Duration::from_secs(10);
/// Default maximum number of bytes the HTTP transport buffers from a single
/// response body before rejecting it. Applied per client; the transport
/// policy sets tighter values for untrusted sources.
pub const DEFAULT_MAX_RESPONSE_BYTES: usize = 10 * 1024 * 1024;
/// Default user-agent applied by the native HTTP transport.
pub const DEFAULT_USER_AGENT: &str = concat!("cow-sdk/", env!("CARGO_PKG_VERSION"));
/// Default TCP keepalive applied by the native HTTP transport.
pub const DEFAULT_TCP_KEEPALIVE: Duration = Duration::from_secs(60);
/// Maximum valid-to timestamp accepted by the protocol `uint32` field.
pub const MAX_VALID_TO_EPOCH: u32 = 4_294_967_295;

const TOKEN_LIST_IMAGES_PATH: &str = "https://files.cow.fi/token-lists/images";
