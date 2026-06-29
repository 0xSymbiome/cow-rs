//! On-chain EIP-1271 signature verification with optional caching.
//!
//! [`verify_eip1271_signature_cached`] orchestrates the canonical
//! `isValidSignature` dispatch against the verifier contract through an
//! injected [`cow_sdk_core::Provider`], consulting an
//! [`Eip1271Cache`] before reaching the chain. The trait is
//! defined here so the contracts crate can take it as a parameter
//! without depending on its sibling crates; callers typically reach for
//! the trait through `cow_sdk_signing::cache` and the always-available
//! `NoopEip1271Cache` implementation, or provide their own.
//!
//! # Cache key
//!
//! The cache key is the full probe identity `(verifier, digest,
//! signature_hash)`, where `signature_hash` is the `keccak256` of the
//! signature bytes. The on-chain `isValidSignature(hash, signature)`
//! contract — and the upstream services validator — make the verdict a
//! function of the signature as well as the digest, so the signature
//! must be part of the key. Omitting it would let the cache return a
//! verdict recorded for a *different* signature on the same digest.
//!
//! # Cached-value semantics (positive-only)
//!
//! The cache is a *set* of probes observed VALID, not a `bool` map. Only
//! a successful magic-value match (`Ok(())`) is recorded. A magic-value
//! mismatch and every other failure mode (transport, missing contract
//! code, serialization, hex decode) are **never recorded** — those
//! probes re-hit the chain on the next call. Two safety properties
//! follow:
//!
//! - a transient transport failure cannot pin a signer into a
//!   "rejected" state; and
//! - a not-yet-valid signature (for example a pre-sign/staged order that
//!   becomes valid on-chain within the TTL) is never blocked by a stale
//!   negative entry — the next probe observes the live activation.
//!
//! A cache implementation's TTL, if any, bounds the only residual
//! staleness, an optimistic VALID that survives an on-chain revocation
//! until the entry expires; the cache is never an authoritative view of
//! mutable on-chain state, and on-chain settlement re-checks the
//! signature regardless.

use std::fmt;

use alloy_primitives::keccak256;
use cow_sdk_core::{Address, Provider};

use crate::ContractsError;
use crate::signature::{
    EIP1271_IS_VALID_SIGNATURE_ABI_JSON, Eip1271VerificationRequest, ensure_contract_code,
    ensure_magic_value, hash32_bytes,
};

/// Optional caching seam consumed by [`verify_eip1271_signature_cached`].
///
/// Implementations record the positive outcome of an EIP-1271 magic-value
/// check, keyed by the full probe identity
/// `(verifier, digest, signature_hash)`. The cache is a set of probes
/// known to have verified VALID: there is no negative cache, so a probe
/// that is not present (or has expired) re-hits the chain. The trait is
/// `Send + Sync + 'static` so the cache may be shared across `tokio`
/// tasks and across consumer crates without lifetime juggling.
pub trait Eip1271Cache: Send + Sync + 'static {
    /// Returns `true` iff the `(verifier, digest, signature_hash)` probe
    /// was recorded VALID by a previous [`record_valid`] and the entry has
    /// not expired. A `false` return means "unknown" — the caller must
    /// re-check the chain; it never means "known invalid".
    ///
    /// `signature_hash` is the `keccak256` of the signature bytes the
    /// verifier consumes.
    ///
    /// [`record_valid`]: Eip1271Cache::record_valid
    fn contains_valid(&self, verifier: Address, digest: [u8; 32], signature_hash: [u8; 32])
    -> bool;

    /// Records that the `(verifier, digest, signature_hash)` probe verified
    /// VALID (the verifier returned the EIP-1271 magic value).
    ///
    /// Only positive outcomes reach this method; implementations are free
    /// to evict pre-existing entries (TTL expiry, capacity bounds, eviction
    /// policy) at record time.
    fn record_valid(&self, verifier: Address, digest: [u8; 32], signature_hash: [u8; 32]);
}

/// Zero-sized [`Eip1271Cache`] that never records anything.
///
/// Every [`contains_valid`](Eip1271Cache::contains_valid) call returns `false`
/// and every [`record_valid`](Eip1271Cache::record_valid) call is a no-op, so a
/// caller gets one verifier dispatch per probe with no caching. It is the
/// default backing for the uncached [`verify_eip1271_signature`], and it lets
/// consumers keep the cache parameter on [`verify_eip1271_signature_cached`]
/// mandatory without paying any allocation or synchronization overhead. It lives
/// here next to the trait so the contracts crate owns the always-available
/// implementation; `cow_sdk_signing::cache` re-exports it.
///
/// [`verify_eip1271_signature`]: crate::signature::verify_eip1271_signature
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NoopEip1271Cache;

impl Eip1271Cache for NoopEip1271Cache {
    fn contains_valid(
        &self,
        _verifier: Address,
        _digest: [u8; 32],
        _signature_hash: [u8; 32],
    ) -> bool {
        false
    }

    fn record_valid(&self, _verifier: Address, _digest: [u8; 32], _signature_hash: [u8; 32]) {}
}

/// Verifies an EIP-1271 signature using an asynchronous provider, with
/// an injected [`Eip1271Cache`] consulted before any
/// on-chain call.
///
/// ## Note
///
/// This verifier does NOT simulate the order's pre-interactions before
/// checking the EIP-1271 signature. Upstream services perform the signature
/// check against a simulated state where the order's pre-interactions have
/// been executed, so a watchtower or off-chain re-verifier built on this
/// helper may see results diverge from services for orders whose verification
/// depends on a pre-interaction (for example, a smart-account that grants the
/// verifier access via a pre-interaction).
///
/// Consumers that need pre-interaction-aware verification should run the
/// pre-interaction simulation at their own RPC seam before calling this helper.
///
/// # Errors
///
/// Returns [`ContractsError`] if the digest cannot be decoded, the
/// verifier has no code, the provider call fails, or the verifier
/// response is malformed or does not match the expected magic value.
#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        skip_all,
        name = "verify.eip1271",
        target = "cow_sdk::verify_eip1271",
        fields(verifier = %request.verifier),
    ),
)]
pub async fn verify_eip1271_signature_cached<P, C>(
    provider: &P,
    request: &Eip1271VerificationRequest,
    cache: &C,
) -> Result<(), ContractsError>
where
    P: Provider,
    P::Error: fmt::Display,
    C: Eip1271Cache + ?Sized,
{
    let digest_key = hash32_bytes(&request.digest);
    let signature_key = keccak256(request.signature.as_slice()).0;

    if cache.contains_valid(request.verifier, digest_key, signature_key) {
        #[cfg(feature = "tracing")]
        tracing::debug!(
            target: "cow_sdk::verify_eip1271",
            cache_status = "hit",
            verification_result = "valid",
        );
        return Ok(());
    }

    #[cfg(feature = "tracing")]
    tracing::debug!(
        target: "cow_sdk::verify_eip1271",
        cache_status = "miss",
    );

    let code_result = ensure_contract_code(provider, &request.verifier).await;
    #[cfg(feature = "tracing")]
    if code_result.is_err() {
        emit_cache_skip_event();
    }
    code_result?;

    let args_json = match serde_json::to_string(&(
        request.digest.to_hex_string(),
        request.signature.to_hex_string(),
    )) {
        Ok(args_json) => args_json,
        Err(error) => {
            #[cfg(feature = "tracing")]
            emit_cache_skip_event();
            return Err(ContractsError::from(error));
        }
    };
    let raw = provider
        .read_contract(&cow_sdk_core::ContractCall::new(
            request.verifier,
            "isValidSignature".to_owned(),
            EIP1271_IS_VALID_SIGNATURE_ABI_JSON.to_owned(),
            args_json,
        ))
        .await
        .map_err(|error| {
            #[cfg(feature = "tracing")]
            emit_cache_skip_event();
            ContractsError::Eip1271Provider {
                operation: "read_contract",
                message: error.to_string().into(),
            }
        })?;

    let outcome = ensure_magic_value(&raw);
    if outcome.is_ok() {
        // Positive-only: only a verified VALID outcome is recorded.
        cache.record_valid(request.verifier, digest_key, signature_key);
        #[cfg(feature = "tracing")]
        tracing::debug!(
            target: "cow_sdk::verify_eip1271",
            cache_status = "store",
            verification_result = "valid",
        );
    }
    #[cfg(feature = "tracing")]
    if let Err(error) = &outcome {
        if matches!(error, ContractsError::Eip1271MagicValueMismatch { .. }) {
            tracing::debug!(
                target: "cow_sdk::verify_eip1271",
                cache_status = "skip",
                verification_result = "invalid",
            );
        } else {
            emit_cache_skip_event();
        }
    }
    outcome
}

#[cfg(feature = "tracing")]
fn emit_cache_skip_event() {
    tracing::debug!(
        target: "cow_sdk::verify_eip1271",
        cache_status = "skip",
        verification_result = "error",
    );
}
