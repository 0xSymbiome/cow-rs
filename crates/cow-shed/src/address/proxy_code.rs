//! Version-keyed COW Shed proxy creation code.

use crate::CowShedVersion;

/// COW Shed proxy creation code for `1.0.0`.
pub const V1_0_0_PROXY_CREATION_CODE: &[u8] = include_bytes!("proxy-creation-code/v1.0.0.bin");

/// COW Shed proxy creation code for `1.0.1`.
pub const V1_0_1_PROXY_CREATION_CODE: &[u8] = include_bytes!("proxy-creation-code/v1.0.1.bin");

/// Returns the proxy creation code for a supported COW Shed version.
#[must_use]
pub const fn proxy_creation_code(version: CowShedVersion) -> &'static [u8] {
    match version {
        CowShedVersion::V1_0_0 => V1_0_0_PROXY_CREATION_CODE,
        CowShedVersion::V1_0_1 => V1_0_1_PROXY_CREATION_CODE,
    }
}
