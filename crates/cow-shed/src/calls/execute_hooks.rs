use alloy_primitives::{Address, B256, Bytes, Signature, U256};
use alloy_sol_types::SolCall;
use cow_sdk_contracts::RecoverableSignature;

use crate::Call;
use crate::bindings::factory::COWShedFactory;

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
/// [`execute_hooks_typed_data_payload`](crate::execute_hooks_typed_data_payload)
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

/// Encodes factory `executeHooks` calldata with an EOA compact signature.
///
/// This is the advanced entry point that takes the pre-split [ERC-2098]
/// compact `(r_compact, vs)` halves. Most callers hold a canonical 65-byte
/// signature and should prefer [`encode_execute_hooks_calldata_signed`], which
/// accepts a [`RecoverableSignature`] directly and needs no manual compaction.
///
/// [ERC-2098]: https://eips.ethereum.org/EIPS/eip-2098
#[must_use]
pub fn encode_execute_hooks_calldata(
    calls: &[Call],
    nonce: B256,
    deadline: U256,
    r_compact: [u8; 32],
    vs: [u8; 32],
    who: Address,
) -> Bytes {
    encode_execute_hooks_calldata_with_signature(
        calls,
        nonce,
        deadline,
        who,
        eoa_signature_from_compact(&r_compact, &vs).to_vec(),
    )
}

/// Encodes factory `executeHooks` calldata from a canonical recoverable signature.
///
/// This is the ergonomic entry point: hand it the 65-byte `r || s || v`
/// [`RecoverableSignature`] a signer already produced (for example through
/// [`crate::CowShedHooks::sign`] or the SDK signing surface) and the factory
/// receives the canonical EOA signature shape directly. No manual ERC-2098
/// compaction or `v`-parity bit handling is required — contrast
/// [`encode_execute_hooks_calldata`], which takes the pre-split compact form
/// for advanced callers.
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

/// Splits a canonical [`RecoverableSignature`] into its [ERC-2098] compact
/// `(r, vs)` halves, folding the recovery parity into the high bit of `vs[0]`.
///
/// This is the inverse of [`eoa_signature_from_compact`]. Prefer
/// [`encode_execute_hooks_calldata_signed`] for the common path; reach for the
/// compact halves only when a caller needs the split form explicitly.
///
/// [ERC-2098]: https://eips.ethereum.org/EIPS/eip-2098
#[must_use]
pub fn compact_signature(signature: &RecoverableSignature) -> ([u8; 32], [u8; 32]) {
    let bytes = signature.to_bytes();
    let mut r = [0_u8; 32];
    r.copy_from_slice(&bytes[..32]);
    let mut vs = [0_u8; 32];
    vs.copy_from_slice(&bytes[32..64]);
    if bytes[64] == 28 {
        vs[0] |= 0x80;
    }
    (r, vs)
}

/// Decodes an [ERC-2098] compact signature `r || vs` into the canonical
/// 65-byte `r || s || v` layout with `v ∈ {27, 28}`.
///
/// Delegates to [`alloy_primitives::Signature::from_erc2098`] over the
/// 64-byte concatenation of `r_compact` and `vs`; the alloy primitive
/// extracts the y-parity bit from the high bit of `vs[0]`, masks it out
/// of the recovered `s`, and stores the canonical `Signature`.
/// [`Signature::as_bytes`] then emits the 65-byte `r || s || v` form
/// with `v = 27 + y_parity`.
///
/// [ERC-2098]: https://eips.ethereum.org/EIPS/eip-2098
#[must_use]
pub fn eoa_signature_from_compact(r_compact: &[u8; 32], vs: &[u8; 32]) -> [u8; 65] {
    let mut compact = [0_u8; 64];
    compact[..32].copy_from_slice(r_compact);
    compact[32..].copy_from_slice(vs);
    Signature::from_erc2098(&compact).as_bytes()
}
