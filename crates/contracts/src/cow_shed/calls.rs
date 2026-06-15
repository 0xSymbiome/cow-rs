//! ABI calldata builders for COW Shed execution paths.

use alloy_primitives::{Address, B256, Bytes, U256};
use alloy_sol_types::SolCall;

use crate::RecoverableSignature;
use crate::cow_shed::Call;
use crate::cow_shed::bindings::COWShedFactory;

/// Encodes factory `executeHooks` calldata with an arbitrary owner signature.
///
/// The general entry point: `signature` is passed through unchanged as the
/// factory's `bytes` argument, so it accepts both a 65-byte EOA ECDSA signature
/// and an EIP-1271 contract-signature blob. On-chain, the proxy runs ECDSA
/// recovery when the signature is 65 bytes and the owner's `isValidSignature`
/// otherwise. For an externally owned account prefer
/// [`encode_execute_hooks_calldata_signed`], which takes the typed
/// [`RecoverableSignature`] and validates the recovery byte; for a
/// smart-contract (EIP-1271) owner, sign the
/// [`execute_hooks_typed_data_payload`](crate::cow_shed::execute_hooks_typed_data_payload)
/// with the owner's signer and pass the resulting blob here.
#[must_use]
pub fn encode_execute_hooks_calldata_with_signature(
    calls: &[Call],
    nonce: B256,
    deadline: U256,
    who: Address,
    signature: impl Into<Bytes>,
) -> Bytes {
    let call = COWShedFactory::executeHooksCall {
        calls: calls.to_vec(),
        nonce,
        deadline,
        user: who,
        signature: signature.into(),
    };
    Bytes::from(call.abi_encode())
}

/// Encodes factory `executeHooks` calldata from a canonical recoverable signature.
///
/// This is the ergonomic entry point: hand it the 65-byte `r || s || v`
/// [`RecoverableSignature`] a signer already produced (for example through
/// [`crate::cow_shed::CowShedHooks::sign`] or the SDK signing surface) and the
/// factory receives the canonical EOA signature shape directly — the only EOA
/// shape the deployed contract decodes. A wallet that hands you the [ERC-2098]
/// compact form instead is one parse away:
/// [`RecoverableSignature::parse_erc2098`].
///
/// [ERC-2098]: https://eips.ethereum.org/EIPS/eip-2098
#[must_use]
pub fn encode_execute_hooks_calldata_signed(
    calls: &[Call],
    nonce: B256,
    deadline: U256,
    who: Address,
    signature: &RecoverableSignature,
) -> Bytes {
    encode_execute_hooks_calldata_with_signature(
        calls,
        nonce,
        deadline,
        who,
        signature.to_bytes().to_vec(),
    )
}
