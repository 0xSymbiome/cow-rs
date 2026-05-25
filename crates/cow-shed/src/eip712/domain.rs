use alloy_primitives::{Address, B256, U256};
use alloy_sol_types::Eip712Domain;
use cow_sdk_core::ChainId;

use crate::CowShedVersion;

const DOMAIN_NAME: &str = "COWShed";

/// Builds the COW Shed per-proxy [`alloy_sol_types::Eip712Domain`] value.
///
/// The domain composes the canonical `EIP712Domain(string name,string
/// version,uint256 chainId,address verifyingContract)` shape with the
/// COW Shed name (`"COWShed"`), the version string of the requested
/// [`CowShedVersion`], the supplied `chain` id, and the proxy
/// `verifyingContract`. Pass the returned domain to
/// [`execute_hooks_signing_hash`](super::execute_hooks_signing_hash) or
/// to any other [`alloy_sol_types::SolStruct::eip712_signing_hash`]
/// caller that targets the COW Shed signing surface. The
/// `parity/fixtures/cow_shed/domain_separator.json` rows lock the
/// per-chain byte contract of the resulting
/// [`Eip712Domain::separator`].
#[must_use]
pub fn cow_shed_eip712_domain(
    chain: ChainId,
    version: CowShedVersion,
    proxy: Address,
) -> Eip712Domain {
    Eip712Domain {
        name: Some(DOMAIN_NAME.into()),
        version: Some(version.version_str().into()),
        chain_id: Some(U256::from(chain)),
        verifying_contract: Some(proxy),
        salt: None,
    }
}

/// Computes the COW Shed per-proxy EIP-712 domain separator.
///
/// Delegates to [`cow_shed_eip712_domain`] followed by
/// [`alloy_sol_types::Eip712Domain::separator`], which composes the
/// canonical `EIP712Domain(string name,string version,uint256
/// chainId,address verifyingContract)` type hash with the packed
/// `(name_hash, version_hash, chain_id_word,
/// verifying_contract_word)` preimage and returns
/// `keccak256(type_hash || encoded_data)`. The
/// `parity/fixtures/cow_shed/domain_separator.json` rows lock the
/// per-chain byte contract.
#[must_use]
pub fn cow_shed_domain_separator(chain: ChainId, version: CowShedVersion, proxy: Address) -> B256 {
    cow_shed_eip712_domain(chain, version, proxy).separator()
}
