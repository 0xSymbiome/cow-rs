//! Environment, address-book, and HTTP client policy types shared across crates.

use std::time::Duration;

use crate::{address, types::Address};

pub use self::{chains::*, env::*, hosts::*, http::*, protocol::*};

mod chains;
mod env;
mod hosts;
mod http;
mod protocol;

/// The EIP-7528 native-asset sentinel (`0xeeee…eeee`): the address `CoW`
/// Protocol uses to represent the chain's native currency, e.g. as the sell
/// token for native-currency (`EthFlow`) sells.
///
/// Typed as [`Address`], so call sites compare and assign it directly; the
/// canonical lowercase wire form per PROP-WB-004 is available through
/// [`Address::to_hex_string`].
pub const NATIVE_CURRENCY_ADDRESS: Address = address!("0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");
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
