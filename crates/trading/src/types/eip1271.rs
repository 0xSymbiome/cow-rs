use serde::{Deserialize, Serialize};

use cow_sdk_core::{Address, HexData};

/// Explicit verifier and signature payload for EIP-1271 verification helpers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Eip1271VerificationParameters {
    /// Smart-account verifier address.
    pub verifier: Address,
    /// Signature bytes supplied to the verifier contract.
    pub signature: HexData,
}

impl Eip1271VerificationParameters {
    /// Creates explicit verifier and signature payload for EIP-1271 verification helpers.
    #[must_use]
    pub const fn new(verifier: Address, signature: HexData) -> Self {
        Self {
            verifier,
            signature,
        }
    }
}
