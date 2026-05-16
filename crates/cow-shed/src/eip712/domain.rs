use alloy_primitives::{Address, B256, U256, keccak256};
use cow_sdk_core::ChainId;

use crate::CowShedVersion;
use crate::address::address_word;
use crate::eip712::EIP712_DOMAIN_TYPE_HASH;

const DOMAIN_NAME: &str = "COWShed";

/// Computes the COW Shed per-proxy EIP-712 domain separator.
#[must_use]
pub fn cow_shed_domain_separator(chain: ChainId, version: CowShedVersion, proxy: Address) -> B256 {
    let mut encoded = Vec::with_capacity(32 * 5);
    encoded.extend_from_slice(EIP712_DOMAIN_TYPE_HASH.as_slice());
    encoded.extend_from_slice(keccak256(DOMAIN_NAME.as_bytes()).as_slice());
    encoded.extend_from_slice(keccak256(version.version_str().as_bytes()).as_slice());
    encoded.extend_from_slice(&U256::from(chain).to_be_bytes::<32>());
    encoded.extend_from_slice(&address_word(proxy));
    keccak256(encoded)
}
