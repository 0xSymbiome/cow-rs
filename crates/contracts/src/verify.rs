//! On-chain EIP-1271 signature verification with optional caching.
//!
//! [`verify_eip1271_signature_async`] orchestrates the canonical
//! `isValidSignature` dispatch against the verifier contract through an
//! injected [`cow_sdk_core::AsyncProvider`], consulting an
//! [`Eip1271VerificationCache`] before reaching the chain. The trait is
//! defined here so the contracts crate can take it as a parameter
//! without depending on its sibling crates; downstream consumers
//! typically reach for the trait through `cow_sdk_signing::cache` and
//! the `NoopEip1271VerificationCache` and `InMemoryEip1271VerificationCache`
//! impls shipped by signing.
//!
//! # Cached-value semantics
//!
//! The cache stores `bool` values with one mapping:
//!
//! - `true` corresponds to a successful magic-value match (`Ok(())` from
//!   the verifier).
//! - `false` corresponds to a magic-value mismatch
//!   (`Err(ContractsError::Eip1271MagicValueMismatch { .. })`).
//!
//! Every other failure mode (transport, missing contract code,
//! serialization, hex decode) is **never cached** — those probes must
//! re-hit the chain on the next call so the caller observes the live
//! state of the on-chain verifier.

use std::fmt;

use cow_sdk_core::{Address, AsyncProvider};

use crate::ContractsError;
use crate::signature::{
    EIP1271_IS_VALID_SIGNATURE_ABI_JSON, EIP1271_MAGICVALUE_BYTES, Eip1271VerificationRequest,
    decode_magic_value_response, ensure_contract_code_async,
};

/// Optional caching seam consumed by [`verify_eip1271_signature_async`].
///
/// Implementations carry the boolean outcome of an EIP-1271
/// magic-value check keyed by the `(verifier, digest)` pair. The trait
/// is `Send + Sync + 'static` so the cache may be shared across
/// `tokio` tasks and across consumer crates without lifetime juggling.
pub trait Eip1271VerificationCache: Send + Sync + 'static {
    /// Returns the cached magic-value-check outcome for the
    /// `(verifier, digest)` pair, if a non-expired entry is present.
    ///
    /// `Some(true)` corresponds to a previously observed `Ok(())`,
    /// `Some(false)` corresponds to a previously observed
    /// `Err(ContractsError::Eip1271MagicValueMismatch { .. })`, and
    /// `None` indicates the cache has nothing reusable for the key.
    fn get(&self, verifier: Address, digest: [u8; 32]) -> Option<bool>;

    /// Inserts the magic-value-check outcome for the
    /// `(verifier, digest)` pair into the cache.
    ///
    /// Implementations are free to evict pre-existing entries (TTL
    /// expiry, capacity bounds, eviction policy) at insert time.
    fn put(&self, verifier: Address, digest: [u8; 32], result: bool);
}

/// Verifies an EIP-1271 signature using an asynchronous provider, with
/// an injected [`Eip1271VerificationCache`] consulted before any
/// on-chain call.
///
/// # Errors
///
/// Returns [`ContractsError`] if the digest cannot be decoded, the
/// verifier has no code, the provider call fails, or the verifier
/// response is malformed or does not match the expected magic value.
pub async fn verify_eip1271_signature_async<P, C>(
    provider: &P,
    request: &Eip1271VerificationRequest,
    cache: &C,
) -> Result<(), ContractsError>
where
    P: AsyncProvider,
    P::Error: fmt::Display,
    C: Eip1271VerificationCache + ?Sized,
{
    let digest_key = decode_digest_key(&request.digest)?;

    if let Some(cached) = cache.get(request.verifier.clone(), digest_key) {
        return cache_hit_to_result(cached);
    }

    ensure_contract_code_async(provider, &request.verifier).await?;
    let raw = provider
        .read_contract(&cow_sdk_core::ContractCall {
            address: request.verifier.clone(),
            method: "isValidSignature".to_owned(),
            abi_json: EIP1271_IS_VALID_SIGNATURE_ABI_JSON.to_owned(),
            args_json: serde_json::to_string(&(
                request.digest.as_str(),
                request.signature.as_str(),
            ))?,
        })
        .await
        .map_err(|error| ContractsError::Eip1271Provider {
            operation: "read_contract",
            message: error.to_string(),
        })?;

    let outcome = ensure_magic_value(&raw);
    match &outcome {
        Ok(()) => cache.put(request.verifier.clone(), digest_key, true),
        Err(ContractsError::Eip1271MagicValueMismatch { .. }) => {
            cache.put(request.verifier.clone(), digest_key, false);
        }
        Err(_) => {}
    }
    outcome
}

const fn cache_hit_to_result(cached: bool) -> Result<(), ContractsError> {
    if cached {
        Ok(())
    } else {
        Err(ContractsError::Eip1271MagicValueMismatch {
            expected: EIP1271_MAGICVALUE_BYTES,
            actual: [0u8; 4],
        })
    }
}

fn decode_digest_key(digest: &cow_sdk_core::Hash32) -> Result<[u8; 32], ContractsError> {
    let stripped = digest
        .as_str()
        .strip_prefix("0x")
        .ok_or(ContractsError::InvalidHexPrefix { field: "digest" })?;
    let bytes = hex::decode(stripped).map_err(|source| ContractsError::DecodeHex {
        field: "digest",
        source,
    })?;
    if bytes.len() != 32 {
        return Err(ContractsError::InvalidDecodedLength {
            field: "digest",
            expected: 32,
            actual: bytes.len(),
        });
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn ensure_magic_value(raw: &str) -> Result<(), ContractsError> {
    let actual = decode_magic_value_response(raw)?;
    if actual == EIP1271_MAGICVALUE_BYTES {
        Ok(())
    } else {
        Err(ContractsError::Eip1271MagicValueMismatch {
            expected: EIP1271_MAGICVALUE_BYTES,
            actual,
        })
    }
}
