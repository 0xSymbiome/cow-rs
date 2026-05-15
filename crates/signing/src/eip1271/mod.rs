//! EIP-1271 custom-signature provider boundary.

mod error;
mod provider;

pub use error::Eip1271SignatureError;
pub use provider::Eip1271SignatureProvider;
