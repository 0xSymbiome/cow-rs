//! Deterministic COW Shed proxy address derivation.

pub mod proxy_code;

use alloy_primitives::{Address, keccak256};

use crate::CowShedVersion;
use proxy_code::proxy_creation_code;

const V1_0_0_IMPLEMENTATION: Address = Address::new([
    0x2c, 0xff, 0xa8, 0xcf, 0x11, 0xb9, 0x0c, 0x9f, 0x43, 0x75, 0x67, 0xb8, 0x63, 0x52, 0x16, 0x9d,
    0xf4, 0x00, 0x9f, 0x73,
]);
const V1_0_1_DEFAULT_IMPLEMENTATION: Address = Address::new([
    0xa2, 0x70, 0x4c, 0xf5, 0x62, 0xad, 0x41, 0x8b, 0xf0, 0x45, 0x3f, 0x4b, 0x66, 0x2e, 0xbf, 0x6a,
    0x24, 0x89, 0xed, 0x88,
]);
const V1_0_1_GNOSIS_FACTORY: Address = Address::new([
    0x4f, 0x43, 0x50, 0xbf, 0x2c, 0x74, 0xaa, 0xcd, 0x50, 0x8d, 0x59, 0x8a, 0x1b, 0xa9, 0x4e, 0xf8,
    0x43, 0x78, 0x79, 0x3d,
]);
const V1_0_1_GNOSIS_IMPLEMENTATION: Address = Address::new([
    0x62, 0xd3, 0xa7, 0xff, 0x48, 0xf9, 0xae, 0x1c, 0x28, 0xa9, 0x55, 0x2a, 0x05, 0x54, 0x82, 0xf8,
    0xc6, 0x37, 0x87, 0xf8,
]);

/// Returns the deterministic proxy address for a user and factory.
#[must_use]
pub fn proxy_of(version: CowShedVersion, factory: Address, user: Address) -> Address {
    let implementation = implementation_for(version, factory);
    let init_code_hash = init_code_hash(version, implementation, user);
    let salt = user_salt(user);

    let mut payload = Vec::with_capacity(1 + 20 + 32 + 32);
    payload.push(0xff);
    payload.extend_from_slice(factory.as_slice());
    payload.extend_from_slice(&salt);
    payload.extend_from_slice(&init_code_hash);

    let hash = keccak256(&payload);
    Address::from_slice(&hash.as_slice()[12..])
}

/// Returns the implementation used by a version and factory pair.
#[must_use]
pub const fn implementation_for(version: CowShedVersion, factory: Address) -> Address {
    match version {
        CowShedVersion::V1_0_0 => V1_0_0_IMPLEMENTATION,
        CowShedVersion::V1_0_1 if address_eq(factory, V1_0_1_GNOSIS_FACTORY) => {
            V1_0_1_GNOSIS_IMPLEMENTATION
        }
        CowShedVersion::V1_0_1 => V1_0_1_DEFAULT_IMPLEMENTATION,
    }
}

/// Returns the CREATE2 salt used by COW Shed factories.
#[must_use]
pub fn user_salt(user: Address) -> [u8; 32] {
    let mut salt = [0_u8; 32];
    salt[12..].copy_from_slice(user.as_slice());
    salt
}

/// Returns the CREATE2 init-code hash for a proxy constructor pair.
#[must_use]
pub fn init_code_hash(version: CowShedVersion, implementation: Address, user: Address) -> [u8; 32] {
    let mut init_code = Vec::with_capacity(proxy_creation_code(version).len() + 64);
    init_code.extend_from_slice(proxy_creation_code(version));
    init_code.extend_from_slice(&address_word(implementation));
    init_code.extend_from_slice(&address_word(user));
    keccak256(&init_code).0
}

const fn address_eq(left: Address, right: Address) -> bool {
    let left = left.into_array();
    let right = right.into_array();
    let mut index = 0;
    while index < 20 {
        if left[index] != right[index] {
            return false;
        }
        index += 1;
    }
    true
}

pub(crate) fn address_word(address: Address) -> [u8; 32] {
    let mut out = [0_u8; 32];
    out[12..].copy_from_slice(address.as_slice());
    out
}
