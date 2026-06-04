//! Deterministic COW Shed proxy address derivation.

pub mod proxy_code;

use alloy_primitives::{Address, keccak256};
use alloy_sol_types::SolValue;
use cow_sdk_contracts::DeploymentChainId;

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
const V1_0_0_FACTORY: Address = Address::new([
    0x00, 0xe9, 0x89, 0xb8, 0x77, 0x00, 0x51, 0x41, 0x18, 0xfa, 0x55, 0x32, 0x6c, 0xd1, 0xcc, 0xe8,
    0x2f, 0xae, 0xbe, 0xf6,
]);
const V1_0_1_DEFAULT_FACTORY: Address = Address::new([
    0x31, 0x2f, 0x92, 0xfe, 0x5f, 0x17, 0x10, 0x40, 0x8b, 0x20, 0xd5, 0x2a, 0x37, 0x4f, 0xa2, 0x9e,
    0x09, 0x9c, 0xfa, 0x86,
]);

/// Returns the deterministic proxy address for a user and factory.
///
/// Delegates the EIP-1014 byte assembly
/// (`0xff || factory || salt || init_code_hash`) and the trailing
/// keccak256 to [`alloy_primitives::Address::create2`]; the
/// `parity/fixtures/cow_shed/proxy_addresses.json` rows lock the
/// per-chain, per-user byte contract.
#[must_use]
pub fn proxy_of(version: CowShedVersion, factory: Address, user: Address) -> Address {
    let implementation = implementation_for(version, factory);
    let init_code_hash = init_code_hash(version, implementation, user);
    factory.create2(user.into_word(), init_code_hash)
}

/// Returns the implementation used by a version and factory pair.
#[must_use]
pub const fn implementation_for(version: CowShedVersion, factory: Address) -> Address {
    match version {
        CowShedVersion::V1_0_0 => V1_0_0_IMPLEMENTATION,
        CowShedVersion::V1_0_1 if factory.const_eq(&V1_0_1_GNOSIS_FACTORY) => {
            V1_0_1_GNOSIS_IMPLEMENTATION
        }
        CowShedVersion::V1_0_1 => V1_0_1_DEFAULT_IMPLEMENTATION,
    }
}

/// Returns the CREATE2 init-code hash for a proxy constructor pair.
///
/// Concatenates the embedded proxy creation code with the canonical ABI
/// encoding of the `(implementation, user)` constructor tuple (two
/// 32-byte left-padded address words) via
/// [`alloy_sol_types::SolValue::abi_encode`], then hashes via
/// [`alloy_primitives::keccak256`].
#[must_use]
pub fn init_code_hash(version: CowShedVersion, implementation: Address, user: Address) -> [u8; 32] {
    let mut init_code = proxy_creation_code(version).to_vec();
    init_code.extend_from_slice(&(implementation, user).abi_encode());
    keccak256(&init_code).0
}

/// Returns the canonical COW Shed factory address for a chain and version.
///
/// `chain` accepts either a [`cow_sdk_core::SupportedChainId`] (what a trading
/// flow already holds) or a [`DeploymentChainId`] directly — the same
/// `impl Into<DeploymentChainId>` shape the `cow-sdk-contracts` `Registry` uses.
/// `DeploymentChainId` is the canonical deployment domain because it covers
/// chains where COW Shed is deployed but the runtime API is not (notably Lens,
/// chain id 232).
///
/// Every `1.0.1` chain shares the canonical factory except Gnosis Chain, which
/// carries a distinct factory (and implementation) deployment; the `1.0.0`
/// generation shares a single legacy factory. The returned address is the
/// CREATE2 deployer that [`proxy_of`] derives against. The per-chain factory
/// and implementation addresses are tracked by the `cow-sdk-contracts`
/// deployment registry.
#[must_use]
pub fn cow_shed_factory(chain: impl Into<DeploymentChainId>, version: CowShedVersion) -> Address {
    let chain = chain.into();
    match version {
        CowShedVersion::V1_0_0 => V1_0_0_FACTORY,
        CowShedVersion::V1_0_1 if chain == DeploymentChainId::GnosisChain => V1_0_1_GNOSIS_FACTORY,
        CowShedVersion::V1_0_1 => V1_0_1_DEFAULT_FACTORY,
    }
}

/// Returns the COW Shed implementation address for a chain and version.
///
/// Resolves the chain's factory via [`cow_shed_factory`] and routes through
/// [`implementation_for`], so the Gnosis-specific implementation is selected
/// automatically. `chain` accepts a [`cow_sdk_core::SupportedChainId`] or a
/// [`DeploymentChainId`].
#[must_use]
pub fn cow_shed_implementation(
    chain: impl Into<DeploymentChainId>,
    version: CowShedVersion,
) -> Address {
    implementation_for(version, cow_shed_factory(chain, version))
}

/// Returns the deterministic proxy address for a user on a chain.
///
/// Resolves the chain's canonical factory via [`cow_shed_factory`] and then
/// delegates to [`proxy_of`], so callers never need to know or hardcode the
/// per-chain factory address. Gnosis Chain's distinct factory/implementation
/// pair is handled transparently. `chain` accepts a
/// [`cow_sdk_core::SupportedChainId`] or a [`DeploymentChainId`].
#[must_use]
pub fn proxy_for(
    chain: impl Into<DeploymentChainId>,
    version: CowShedVersion,
    user: Address,
) -> Address {
    proxy_of(version, cow_shed_factory(chain, version), user)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{Address, CowShedVersion, DeploymentChainId, proxy_for};

    /// ADR 0049: distinct `CowShedVersion` variants must derive distinct proxy
    /// addresses for the same user, so a discovery flow can tell a user's
    /// per-version proxies apart.
    #[test]
    fn distinct_versions_derive_distinct_proxies() {
        let user = Address::new([0x11_u8; 20]);
        let proxies: BTreeSet<Address> = CowShedVersion::ALL
            .into_iter()
            .map(|version| proxy_for(DeploymentChainId::Mainnet, version, user))
            .collect();
        assert_eq!(
            proxies.len(),
            CowShedVersion::ALL.len(),
            "each supported COW Shed version must derive a distinct proxy"
        );
    }
}
