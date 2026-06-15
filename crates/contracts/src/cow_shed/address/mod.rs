//! Deterministic COW Shed proxy address derivation.
//!
//! The deployed COW Shed factory and implementation are identical on every
//! supported chain for a given [`CowShedVersion`] (deterministic CREATE2
//! deployments, the same posture as the GPv2 settlement registry), so the
//! lookups here are keyed by version alone and the derived proxy address is
//! chain-independent. Chain id enters the COW Shed story only through the
//! EIP-712 signing domain ([`crate::cow_shed::eip712`]).

use alloy_primitives::{Address, address, keccak256};
use alloy_sol_types::SolValue;

use crate::cow_shed::CowShedVersion;

/// COW Shed proxy creation code for `1.0.0`.
const V1_0_0_PROXY_CREATION_CODE: &[u8] = include_bytes!("proxy-creation-code/v1.0.0.bin");

/// COW Shed proxy creation code for `1.0.1`.
const V1_0_1_PROXY_CREATION_CODE: &[u8] = include_bytes!("proxy-creation-code/v1.0.1.bin");

/// Returns the proxy creation code for a supported COW Shed version.
const fn proxy_creation_code(version: CowShedVersion) -> &'static [u8] {
    match version {
        CowShedVersion::V1_0_0 => V1_0_0_PROXY_CREATION_CODE,
        CowShedVersion::V1_0_1 => V1_0_1_PROXY_CREATION_CODE,
    }
}

/// `COWShedFactory` `1.0.0` deployment — identical on every supported chain.
const V1_0_0_FACTORY: Address = address!("0x00E989b87700514118Fa55326CD1cCE82faebEF6");
/// `COWShedFactory` `1.0.1` deployment — identical on every supported chain.
const V1_0_1_FACTORY: Address = address!("0x312f92fe5f1710408B20D52A374fa29e099cFA86");
/// `COWShed` `1.0.0` implementation — identical on every supported chain.
const V1_0_0_IMPLEMENTATION: Address = address!("0x2CFFA8cf11B90C9F437567b86352169dF4009F73");
/// `COWShed` `1.0.1` implementation — identical on every supported chain.
const V1_0_1_IMPLEMENTATION: Address = address!("0xa2704cF562AD418Bf0453F4B662ebf6A2489eD88");

/// Returns the canonical COW Shed factory address for a version.
///
/// The factory is a deterministic deployment, identical on every supported
/// chain, so the lookup needs no chain id. The
/// `parity/fixtures/cow_shed/deployments.json` rows pin the per-version pair.
#[must_use]
pub const fn cow_shed_factory(version: CowShedVersion) -> Address {
    match version {
        CowShedVersion::V1_0_0 => V1_0_0_FACTORY,
        CowShedVersion::V1_0_1 => V1_0_1_FACTORY,
    }
}

/// Returns the canonical COW Shed implementation address for a version.
///
/// Identical on every supported chain, mirroring [`cow_shed_factory`]. This is
/// the implementation the canonical factory passes to the proxy constructor,
/// so it participates in the CREATE2 [`init_code_hash`].
#[must_use]
pub const fn cow_shed_implementation(version: CowShedVersion) -> Address {
    match version {
        CowShedVersion::V1_0_0 => V1_0_0_IMPLEMENTATION,
        CowShedVersion::V1_0_1 => V1_0_1_IMPLEMENTATION,
    }
}

/// Returns the CREATE2 init-code hash for a proxy constructor pair.
///
/// Concatenates the embedded proxy creation code with the canonical ABI
/// encoding of the `(implementation, user)` constructor tuple (two
/// 32-byte left-padded address words) via
/// [`alloy_sol_types::SolValue::abi_encode`], then hashes via
/// [`alloy_primitives::keccak256`]. Combine with
/// [`alloy_primitives::Address::create2`] to derive a proxy for a fully
/// custom factory/implementation pair (the TS arbiter's custom-options
/// path); [`proxy_of`] and [`proxy_for`] cover the canonical pairs.
#[must_use]
pub fn init_code_hash(version: CowShedVersion, implementation: Address, user: Address) -> [u8; 32] {
    let mut init_code = proxy_creation_code(version).to_vec();
    init_code.extend_from_slice(&(implementation, user).abi_encode());
    keccak256(&init_code).0
}

/// Returns the deterministic proxy address for a user under an explicit factory.
///
/// Pairs `factory` with the version's canonical implementation and creation
/// code, then delegates the EIP-1014 byte assembly
/// (`0xff || factory || salt || init_code_hash`) and the trailing keccak256 to
/// [`alloy_primitives::Address::create2`]. Reach for this when targeting a
/// re-deployed factory of a supported generation; for a custom implementation,
/// use [`init_code_hash`] with [`alloy_primitives::Address::create2`] directly.
/// The `parity/fixtures/cow_shed/proxy_addresses.json` rows lock the byte
/// contract.
#[must_use]
pub fn proxy_of(version: CowShedVersion, factory: Address, user: Address) -> Address {
    let implementation = cow_shed_implementation(version);
    let init_code_hash = init_code_hash(version, implementation, user);
    factory.create2(user.into_word(), init_code_hash)
}

/// Returns the deterministic proxy ("shed") address for a user.
///
/// Resolves the version's canonical factory via [`cow_shed_factory`] and
/// delegates to [`proxy_of`]. The result is chain-independent: every CREATE2
/// input (factory, creation code, implementation, user-as-salt) is fixed per
/// version, which is why the deployed proxy for a user is the same address on
/// every supported chain.
#[must_use]
pub fn proxy_for(version: CowShedVersion, user: Address) -> Address {
    proxy_of(version, cow_shed_factory(version), user)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{Address, CowShedVersion, proxy_for};

    /// ADR 0049: distinct `CowShedVersion` variants must derive distinct proxy
    /// addresses for the same user, so a discovery flow can tell a user's
    /// per-version proxies apart.
    #[test]
    fn distinct_versions_derive_distinct_proxies() {
        let user = Address::new([0x11_u8; 20]);
        let proxies: BTreeSet<Address> = CowShedVersion::ALL
            .into_iter()
            .map(|version| proxy_for(version, user))
            .collect();
        assert_eq!(
            proxies.len(),
            CowShedVersion::ALL.len(),
            "each supported COW Shed version must derive a distinct proxy"
        );
    }
}
