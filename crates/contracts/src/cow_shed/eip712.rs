//! COW Shed EIP-712 domain, hashing, and typed-data helpers.

use alloy_primitives::{Address, B256, U256};
use alloy_sol_types::{Eip712Domain, SolStruct};
use cow_sdk_core::{
    Address as CoreAddress, ChainId, TypedDataDomain, TypedDataField, TypedDataPayload,
    TypedDataTypes,
};

use crate::cow_shed::CowShedVersion;
use crate::cow_shed::types::Call;

pub use crate::cow_shed::bindings::{Call as SolCall, ExecuteHooks};

const DOMAIN_NAME: &str = "COWShed";
const PRIMARY_TYPE: &str = "ExecuteHooks";

/// Builds the COW Shed per-proxy [`alloy_sol_types::Eip712Domain`] value.
///
/// The domain composes the canonical `EIP712Domain(string name,string
/// version,uint256 chainId,address verifyingContract)` shape with the
/// COW Shed name (`"COWShed"`), the version string of the requested
/// [`CowShedVersion`], the supplied `chain` id, and the proxy
/// `verifyingContract`. Pass the returned domain to
/// [`execute_hooks_signing_hash`] or to any other
/// [`alloy_sol_types::SolStruct::eip712_signing_hash`] caller that targets the
/// COW Shed signing surface. The `parity/fixtures/cow_shed/domain_separator.json`
/// rows lock the per-chain byte contract of the resulting
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
/// [`alloy_sol_types::Eip712Domain::separator`], which composes the canonical
/// `EIP712Domain(string name,string version,uint256 chainId,address
/// verifyingContract)` type hash with the packed `(name_hash, version_hash,
/// chain_id_word, verifying_contract_word)` preimage and returns
/// `keccak256(type_hash || encoded_data)`. The
/// `parity/fixtures/cow_shed/domain_separator.json` rows lock the per-chain
/// byte contract.
#[must_use]
pub fn cow_shed_domain_separator(chain: ChainId, version: CowShedVersion, proxy: Address) -> B256 {
    cow_shed_eip712_domain(chain, version, proxy).separator()
}

/// Computes the COW Shed `ExecuteHooks` EIP-712 signing hash.
///
/// Delegates to [`alloy_sol_types::SolStruct::eip712_signing_hash`] on the
/// macro-emitted [`ExecuteHooks`] struct declared in
/// [`crate::cow_shed::bindings`]. The macro emits the canonical EIP-712 envelope
/// per the specification (the `0x19` prefix followed by the `0x01` typed-data
/// version, then the domain separator and the struct hash, all routed through
/// [`alloy_primitives::keccak256`]). The
/// `parity/fixtures/cow_shed/execute_hooks_digest.json` rows lock the per-row
/// byte contract.
///
/// The `domain` argument is the value returned by [`cow_shed_eip712_domain`]
/// for the target chain, version, and proxy address.
#[must_use]
pub fn execute_hooks_signing_hash(
    domain: &Eip712Domain,
    calls: &[Call],
    nonce: B256,
    deadline: U256,
) -> B256 {
    ExecuteHooks {
        calls: calls.to_vec(),
        nonce,
        deadline,
    }
    .eip712_signing_hash(domain)
}

/// Builds the SDK [`TypedDataPayload`] for a COW Shed `ExecuteHooks` message.
///
/// The payload carries the full EIP-712 type map (`EIP712Domain`, `Call`,
/// `ExecuteHooks`), the per-proxy domain, and a canonical JSON message whose
/// field encodings match the SDK signer's dynamic typed-data path: addresses
/// and `bytes`/`bytes32` as lowercase `0x` hex, `uint256` as decimal strings,
/// and booleans as JSON booleans. Sign it with any [`cow_sdk_core::Signer`]
/// through `sign_typed_data_payload`; the resulting digest is byte-identical
/// to [`execute_hooks_signing_hash`], so the EOA recovers to the proxy owner
/// on-chain.
///
/// `proxy` must be the signer-owner's COW Shed proxy (the EIP-712
/// `verifyingContract`), as derived by [`crate::cow_shed::proxy_for`] or
/// [`crate::cow_shed::proxy_of`].
#[must_use]
pub fn execute_hooks_typed_data_payload(
    chain: ChainId,
    version: CowShedVersion,
    proxy: Address,
    calls: &[Call],
    nonce: B256,
    deadline: U256,
) -> TypedDataPayload {
    let mut types = TypedDataTypes::new();
    types.insert("EIP712Domain".to_owned(), domain_fields());
    types.insert("Call".to_owned(), call_fields());
    types.insert(PRIMARY_TYPE.to_owned(), execute_hooks_fields());

    let domain = TypedDataDomain::new(
        DOMAIN_NAME.to_owned(),
        version.version_str().to_owned(),
        chain,
        CoreAddress::from_bytes(proxy.into_array()),
    );

    let message = serde_json::json!({
        "calls": calls.iter().map(call_to_json).collect::<Vec<_>>(),
        "nonce": format!("{nonce:#x}"),
        "deadline": deadline.to_string(),
    })
    .to_string();

    TypedDataPayload::new(domain, PRIMARY_TYPE.to_owned(), types, message)
}

fn domain_fields() -> Vec<TypedDataField> {
    vec![
        TypedDataField::new("name".to_owned(), "string".to_owned()),
        TypedDataField::new("version".to_owned(), "string".to_owned()),
        TypedDataField::new("chainId".to_owned(), "uint256".to_owned()),
        TypedDataField::new("verifyingContract".to_owned(), "address".to_owned()),
    ]
}

fn call_fields() -> Vec<TypedDataField> {
    vec![
        TypedDataField::new("target".to_owned(), "address".to_owned()),
        TypedDataField::new("value".to_owned(), "uint256".to_owned()),
        TypedDataField::new("callData".to_owned(), "bytes".to_owned()),
        TypedDataField::new("allowFailure".to_owned(), "bool".to_owned()),
        TypedDataField::new("isDelegateCall".to_owned(), "bool".to_owned()),
    ]
}

fn execute_hooks_fields() -> Vec<TypedDataField> {
    vec![
        TypedDataField::new("calls".to_owned(), "Call[]".to_owned()),
        TypedDataField::new("nonce".to_owned(), "bytes32".to_owned()),
        TypedDataField::new("deadline".to_owned(), "uint256".to_owned()),
    ]
}

fn call_to_json(call: &Call) -> serde_json::Value {
    serde_json::json!({
        "target": format!("{:#x}", call.target),
        "value": call.value.to_string(),
        "callData": call.callData.to_string(),
        "allowFailure": call.allowFailure,
        "isDelegateCall": call.isDelegateCall,
    })
}
