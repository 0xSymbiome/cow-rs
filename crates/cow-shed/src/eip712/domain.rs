use alloy_primitives::{Address, B256, U256};
use alloy_sol_types::Eip712Domain;
use cow_sdk_core::ChainId;

use crate::CowShedVersion;

const DOMAIN_NAME: &str = "COWShed";

/// Computes the COW Shed per-proxy EIP-712 domain separator.
///
/// Delegates to [`alloy_sol_types::Eip712Domain::separator`], which composes
/// the canonical `EIP712Domain(string name,string version,uint256
/// chainId,address verifyingContract)` type hash with the packed
/// `(name_hash, version_hash, chain_id_word, verifying_contract_word)`
/// preimage and returns `keccak256(type_hash || encoded_data)`.
/// Byte-identical to the prior in-crate encoder; verified by the shared
/// parity fixture under `parity/fixtures/cow_shed/domain_separator.json`.
#[must_use]
pub fn cow_shed_domain_separator(chain: ChainId, version: CowShedVersion, proxy: Address) -> B256 {
    Eip712Domain {
        name: Some(DOMAIN_NAME.into()),
        version: Some(version.version_str().into()),
        chain_id: Some(U256::from(chain)),
        verifying_contract: Some(proxy),
        salt: None,
    }
    .separator()
}
