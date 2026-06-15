//! EIP-1271 custom-signature provider boundary.

mod error;
mod provider;
pub(crate) mod sol_types;

pub use error::Eip1271SignatureError;
pub use provider::Eip1271Signer;
pub use sol_types::{OnchainOrder, OrderAndSignature};
