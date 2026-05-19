//! On-chain EIP-1271 signature verification with optional caching.
//!
//! [`verify_eip1271_signature_async`] orchestrates the canonical
//! `isValidSignature` dispatch against the verifier contract through an
//! injected [`cow_sdk_core::AsyncProvider`], consulting an
//! [`Eip1271VerificationCache`] before reaching the chain. The trait is
//! defined here so the contracts crate can take it as a parameter
//! without depending on its sibling crates; callers typically reach for
//! the trait through `cow_sdk_signing::cache` and the
//! `NoopEip1271VerificationCache` and
//! `InMemoryEip1271VerificationCache` implementations in the signing
//! crate.
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
    let digest_key = decode_digest_key(&request.digest);

    if let Some(cached) = cache.get(request.verifier, digest_key) {
        #[cfg(feature = "tracing")]
        tracing::debug!(
            target: "cow_sdk::verify_eip1271",
            cache_status = "hit",
            verification_result = if cached { "valid" } else { "invalid" },
        );
        return cache_hit_to_result(cached);
    }

    #[cfg(feature = "tracing")]
    tracing::debug!(
        target: "cow_sdk::verify_eip1271",
        cache_status = "miss",
    );

    let code_result = ensure_contract_code_async(provider, &request.verifier).await;
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
    if let Some(cached) = cacheable_verification_outcome(&outcome) {
        cache.put(request.verifier, digest_key, cached);
        #[cfg(feature = "tracing")]
        tracing::debug!(
            target: "cow_sdk::verify_eip1271",
            cache_status = "store",
            verification_result = if cached { "valid" } else { "invalid" },
        );
    } else {
        #[cfg(feature = "tracing")]
        emit_cache_skip_event();
    }
    outcome
}

const fn cacheable_verification_outcome(outcome: &Result<(), ContractsError>) -> Option<bool> {
    match outcome {
        Ok(()) => Some(true),
        Err(ContractsError::Eip1271MagicValueMismatch { .. }) => Some(false),
        Err(
            ContractsError::Core(_)
            | ContractsError::Cancelled
            | ContractsError::UnsupportedChain(_)
            | ContractsError::InvalidOrderUidLength { .. }
            | ContractsError::InvalidNumeric { .. }
            | ContractsError::NumericOverflow { .. }
            | ContractsError::InvalidFlags(_)
            | ContractsError::UnsupportedSigningScheme(_)
            | ContractsError::InvalidEip1271SignatureData
            | ContractsError::UnsupportedEip1271Verifier { .. }
            | ContractsError::Eip1271Provider { .. }
            | ContractsError::MalformedEip1271Response { .. }
            | ContractsError::MissingClearingPrice { .. }
            | ContractsError::MissingExecutedAmount
            | ContractsError::MissingTrade
            | ContractsError::ZeroReceiver
            | ContractsError::InvalidTokenIndex { .. }
            | ContractsError::ForbiddenInteractionTarget { .. }
            | ContractsError::Provider { .. }
            | ContractsError::Abi(_)
            | ContractsError::DecodeHex { .. }
            | ContractsError::InvalidHexPrefix { .. }
            | ContractsError::InvalidDecodedLength { .. }
            | ContractsError::Serialization(_)
            | ContractsError::InvalidSignatureLength { .. }
            | ContractsError::InvalidSignatureRecoveryByte { .. }
            | ContractsError::SignatureSchemeNotEcdsa
            | ContractsError::SignatureRecovery { .. },
        ) => None,
    }
}

#[cfg(feature = "tracing")]
fn emit_cache_skip_event() {
    tracing::debug!(
        target: "cow_sdk::verify_eip1271",
        cache_status = "skip",
        verification_result = "error",
    );
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

const fn decode_digest_key(digest: &cow_sdk_core::Hash32) -> [u8; 32] {
    digest.into_alloy().0
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
