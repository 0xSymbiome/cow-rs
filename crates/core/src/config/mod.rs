//! Environment, address-book, and HTTP client policy types shared across crates.

use std::time::Duration;

pub use self::{chains::*, env::*, hosts::*, http::*, protocol::*};

mod chains;
mod env;
mod hosts;
mod http;
mod protocol;

/// All supported `CoW` API environments.
pub const ENVS_LIST: [CowEnv; 2] = [CowEnv::Prod, CowEnv::Staging];
/// Sentinel address used by `CoW` Protocol to represent the native chain asset.
pub const EVM_NATIVE_CURRENCY_ADDRESS: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";
/// Default timeout applied to HTTP-backed SDK clients.
pub const DEFAULT_HTTP_TIMEOUT: Duration = Duration::from_secs(10);
/// Default user-agent applied by the native HTTP transport.
pub const DEFAULT_USER_AGENT: &str = concat!("cow-sdk/", env!("CARGO_PKG_VERSION"));
/// Default TCP keepalive applied by the native HTTP transport.
pub const DEFAULT_TCP_KEEPALIVE: Duration = Duration::from_secs(60);
/// Maximum valid-to timestamp accepted by the protocol `uint32` field.
pub const MAX_VALID_TO_EPOCH: u32 = 4_294_967_295;

const TOKEN_LIST_IMAGES_PATH: &str = "https://files.cow.fi/token-lists/images";
