//! COW Shed `ExecuteHooks` typed-data payload construction.
//!
//! The payload produced here is the bridge between the macro-emitted
//! [`ExecuteHooks`](super::ExecuteHooks) signing surface and the SDK's owned
//! [`cow_sdk_core::Signer`] trait. Signing it via `sign_typed_data_payload`
//! yields a signature over the same digest as
//! [`execute_hooks_signing_hash`](super::execute_hooks_signing_hash), so the
//! EOA recovers to the proxy owner on-chain.

use alloy_primitives::{Address, B256, U256};
use cow_sdk_core::{
    Address as CoreAddress, ChainId, TypedDataDomain, TypedDataField, TypedDataPayload,
    TypedDataTypes,
};

use crate::{Call, CowShedVersion};

const DOMAIN_NAME: &str = "COWShed";
const PRIMARY_TYPE: &str = "ExecuteHooks";

/// Builds the SDK [`TypedDataPayload`] for a COW Shed `ExecuteHooks` message.
///
/// The payload carries the full EIP-712 type map (`EIP712Domain`, `Call`,
/// `ExecuteHooks`), the per-proxy domain, and a canonical JSON message whose
/// field encodings match the SDK signer's dynamic typed-data path: addresses
/// and `bytes`/`bytes32` as lowercase `0x` hex, `uint256` as decimal strings,
/// and booleans as JSON booleans. Sign it with any [`cow_sdk_core::Signer`]
/// through `sign_typed_data_payload`; the resulting digest is byte-identical
/// to [`execute_hooks_signing_hash`](super::execute_hooks_signing_hash), so the
/// EOA recovers to the proxy owner on-chain.
///
/// `proxy` must be the signer-owner's COW Shed proxy (the EIP-712
/// `verifyingContract`), as derived by [`crate::proxy_for`] or
/// [`crate::proxy_of`].
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
