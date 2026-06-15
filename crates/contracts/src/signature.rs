use std::fmt;

use alloy_primitives::{B256, Signature as AlloySignature};
use alloy_sol_types::{SolCall, sol};
use serde::{Deserialize, Serialize};

use cow_sdk_core::{Address, Hash32, HexData, Provider};

use crate::ContractsError;
use crate::hex_field::{decode_hex_field_bounded, decode_hex_field_exact};

sol! {
    /// EIP-1271 smart-account signature-validation interface.
    ///
    /// Reproduces the canonical surface defined by
    /// [EIP-1271](https://eips.ethereum.org/EIPS/eip-1271). Verifier contracts
    /// return the 4-byte function selector
    /// `keccak256("isValidSignature(bytes32,bytes)")[..4]` on a successful
    /// validation; the cow signature path compares the decoded response against
    /// [`IERC1271::isValidSignatureCall::SELECTOR`], which doubles as both the
    /// dispatch selector and the success magic value.
    ///
    /// Mirrors cowdao-grants/cow-shed `src/interfaces/IERC1271.sol`, pinned by
    /// commit in `parity/source-lock.yaml`; the selector is proven by the crate
    /// parity tests.
    interface IERC1271 {
        function isValidSignature(bytes32 hash, bytes calldata signature) external view returns (bytes4);
    }
}

pub(crate) const EIP1271_IS_VALID_SIGNATURE_ABI_JSON: &str = r#"[{"type":"function","name":"isValidSignature","inputs":[{"name":"hash","type":"bytes32"},{"name":"signature","type":"bytes"}],"outputs":[{"name":"","type":"bytes4"}],"stateMutability":"view"}]"#;

/// Maximum decoded byte length accepted for a signature hex field.
///
/// Set to the upstream orderbook request-body limit, so it can never reject a
/// signature the orderbook would accept: a signed order, signature included,
/// must already fit within that request body. The bound exists to refuse
/// oversized non-transport input — fixtures, fuzz data, or third-party callers
/// — before the hex decoder allocates a large buffer.
pub const MAX_SIGNATURE_HEX_BYTES: usize = 16 * 1024;

/// Supported `CoW` signing schemes.
#[doc(alias = "Scheme")]
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum SigningScheme {
    /// EIP-712 typed-data signature.
    Eip712 = 0,
    /// `eth_sign` style message signature.
    EthSign = 1,
    /// EIP-1271 smart-account signature.
    Eip1271 = 2,
    /// Pre-sign on-chain approval.
    PreSign = 3,
}

impl SigningScheme {
    /// Returns the compact numeric encoding for the signing scheme.
    #[inline]
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    /// Returns whether the scheme produces an ECDSA signature locally.
    #[inline]
    #[must_use]
    pub const fn is_ecdsa(self) -> bool {
        matches!(self, Self::Eip712 | Self::EthSign)
    }
}

impl TryFrom<u8> for SigningScheme {
    type Error = ContractsError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Eip712),
            1 => Ok(Self::EthSign),
            2 => Ok(Self::Eip1271),
            3 => Ok(Self::PreSign),
            value => Err(ContractsError::UnsupportedSigningScheme(value)),
        }
    }
}

/// Decoded EIP-1271 verifier payload.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Eip1271SignatureData {
    /// Verifier contract address.
    pub verifier: Address,
    /// Encoded signature payload.
    ///
    /// Typed as [`HexData`] — the same shape as the sibling
    /// [`Eip1271VerificationRequest::signature`] — so the payload cannot hold
    /// non-hex bytes and the codec path needs no per-call re-parse.
    pub signature: HexData,
}

/// Input contract for EIP-1271 verification helpers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Eip1271VerificationRequest {
    /// Verifier contract address.
    pub verifier: Address,
    /// Digest being validated.
    pub digest: Hash32,
    /// Signature bytes.
    pub signature: HexData,
}

/// `CoW` signature union.
///
/// The enum is `#[non_exhaustive]` so future protocol-side signature forms can
/// extend the public surface without breaking existing consumers. Internal
/// matches remain exhaustive; downstream matches must include a wildcard arm.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Signature {
    /// Locally produced ECDSA signature plus scheme discriminator.
    Ecdsa {
        /// ECDSA signing scheme used to create `data`.
        scheme: SigningScheme,
        /// Signature bytes as a hex string.
        data: String,
    },
    /// EIP-1271 smart-account signature payload.
    Eip1271 {
        /// Verifier contract payload.
        data: Eip1271SignatureData,
    },
    /// Pre-sign owner address.
    PreSign {
        /// Owner address that pre-signed the order on-chain.
        owner: Address,
    },
}

impl Eip1271SignatureData {
    /// Creates an EIP-1271 verifier payload.
    #[must_use]
    pub const fn new(verifier: Address, signature: HexData) -> Self {
        Self {
            verifier,
            signature,
        }
    }
}

impl Eip1271VerificationRequest {
    /// Creates an EIP-1271 verification request.
    #[must_use]
    pub const fn new(verifier: Address, digest: Hash32, signature: HexData) -> Self {
        Self {
            verifier,
            digest,
            signature,
        }
    }
}

impl Signature {
    /// Returns the signing scheme represented by this signature.
    #[must_use]
    pub const fn scheme(&self) -> SigningScheme {
        match self {
            Self::Ecdsa { scheme, .. } => *scheme,
            Self::Eip1271 { .. } => SigningScheme::Eip1271,
            Self::PreSign { .. } => SigningScheme::PreSign,
        }
    }

    /// Returns the address declared directly by non-ECDSA signature variants.
    ///
    /// EIP-1271 signatures declare the verifier contract, and pre-sign
    /// signatures declare the owner. ECDSA variants return `None` because the
    /// owner is recovered cryptographically with
    /// [`Signature::recover_ecdsa_address`].
    #[must_use]
    pub const fn declared_address(&self) -> Option<&Address> {
        match self {
            Self::Ecdsa { .. } => None,
            Self::Eip1271 { data } => Some(&data.verifier),
            Self::PreSign { owner } => Some(owner),
        }
    }

    /// Recovers the signer address for an ECDSA signature and 32-byte digest.
    ///
    /// For [`SigningScheme::Eip712`], `digest` is the exact prehash supplied
    /// to the recovery backend. For [`SigningScheme::EthSign`], recovery uses
    /// the EIP-191 prehash `keccak256("\x19Ethereum Signed Message:\n32" ||
    /// digest_bytes)` because `CoW Protocol` signs the 32-byte order digest as
    /// the message body.
    ///
    /// Delegates to [`RecoverableSignature::recover`] after parsing through
    /// [`RecoverableSignature::parse_hex`], so the strict ADR 0022 input
    /// contract is enforced before any cryptographic operation runs.
    ///
    /// # Errors
    ///
    /// Returns [`ContractsError::SignatureSchemeNotEcdsa`] for EIP-1271 and
    /// pre-sign variants, and another [`ContractsError`] when the signature or
    /// digest cannot be decoded or recovered.
    pub fn recover_ecdsa_address(&self, digest: &Hash32) -> Result<Address, ContractsError> {
        let Self::Ecdsa { scheme, data } = self else {
            return Err(ContractsError::SignatureSchemeNotEcdsa);
        };
        RecoverableSignature::parse_hex(data)?.recover(digest, *scheme)
    }
}

/// A recoverable ECDSA signature that has cleared the contracts-boundary
/// canonicalization contract.
///
/// Construction is closed: the only paths are [`Self::parse_hex`] and
/// [`Self::parse_bytes`], both of which reject every trailing recovery
/// byte outside `{0, 1, 27, 28}` through
/// [`ContractsError::InvalidSignatureRecoveryByte`] before the value
/// exists. Holding a `RecoverableSignature` is therefore a typestate
/// proof of canonicalization on the legacy Solidity `{27, 28}` range
/// per [ADR 0022](../../docs/adr/0022-ecdsa-signature-v-normalization.md).
///
/// The internal representation is an [`alloy_primitives::Signature`],
/// stored by parity rather than by recovery byte. Canonical
/// serialization through [`Self::to_bytes`] / [`Self::to_hex_string`]
/// emits the legacy `r || s || (27 + y_parity)` byte layout.
/// ERC-2098 compact decoding and encoding ride through the same backing
/// value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RecoverableSignature {
    inner: AlloySignature,
}

impl RecoverableSignature {
    /// Parses a 65-byte `0x`-prefixed hex string.
    ///
    /// # Errors
    ///
    /// Returns [`ContractsError`] for hex envelope failures,
    /// [`ContractsError::InvalidSignatureLength`] for non-65-byte payloads,
    /// and [`ContractsError::InvalidSignatureRecoveryByte`] for trailing
    /// byte values outside `{0, 1, 27, 28}`.
    pub fn parse_hex(data: &str) -> Result<Self, ContractsError> {
        let bytes = decode_hex_field_bounded("signature", data, MAX_SIGNATURE_HEX_BYTES)?;
        Self::parse_bytes(&bytes)
    }

    /// Parses a 65-byte raw payload.
    ///
    /// The trailing recovery byte is validated against the ADR 0022 accept
    /// set `{0, 1, 27, 28}` and reduced to a parity bit. The parity bit is
    /// then handed to [`AlloySignature::from_bytes_and_parity`], which
    /// constructs the signature value directly without re-running the
    /// wider alloy parity-normalization path that would otherwise admit
    /// EIP-155 chain-id encoded `v` values.
    ///
    /// # Errors
    ///
    /// Returns [`ContractsError::InvalidSignatureLength`] for non-65-byte
    /// payloads, and [`ContractsError::InvalidSignatureRecoveryByte`]
    /// for trailing byte values outside `{0, 1, 27, 28}`.
    // DO NOT SWAP for alloy_primitives::Signature::from_raw.
    //
    // `Signature::from_raw` delegates to `normalize_v`, which accepts
    // v in {0, 1, 27, 28, 35..}. Values >= 35 are the EIP-155 legacy
    // transaction encoding that mixes the chain id into v. CoW off-chain
    // order signatures never carry an EIP-155 v value, so this surface
    // narrows the accept set to {0, 1, 27, 28} and rejects every other
    // value through the typed `InvalidSignatureRecoveryByte` variant
    // (ADR 0022).
    //
    // After the strict pre-validation produces a parity bool,
    // `from_bytes_and_parity` consumes the parity directly and skips
    // `normalize_v` entirely. The legacy `27 + y_parity` byte emerges
    // from `as_bytes()` by construction.
    //
    // ADR: docs/adr/0022-ecdsa-signature-v-normalization.md
    // Doctrine: docs/alloy-doctrine.md, Bucket 2 row for ECDSA `v` byte
    // canonicalization.
    // Enforced by cargo check-source-fences (xtask/src/policy/fences.rs).
    pub fn parse_bytes(bytes: &[u8]) -> Result<Self, ContractsError> {
        if bytes.len() != 65 {
            return Err(ContractsError::InvalidSignatureLength {
                actual: bytes.len(),
            });
        }
        let parity = match bytes[64] {
            0 | 27 => false,
            1 | 28 => true,
            value => return Err(ContractsError::InvalidSignatureRecoveryByte { value }),
        };
        Ok(Self {
            inner: AlloySignature::from_bytes_and_parity(&bytes[..64], parity),
        })
    }

    /// Returns the canonical 65-byte legacy form (`r || s || (27 + parity)`).
    #[must_use]
    pub fn to_bytes(&self) -> [u8; 65] {
        self.inner.as_bytes()
    }

    /// Returns the canonical lowercase `0x`-prefixed 65-byte hex form.
    #[must_use]
    pub fn to_hex_string(&self) -> String {
        alloy_primitives::hex::encode_prefixed(self.inner.as_bytes())
    }

    /// Returns a borrow of the underlying alloy primitive.
    ///
    /// The canonicalization contract is enforced by this wrapper; reading
    /// the inner value and re-serializing through alternative alloy
    /// representations (for example `Signature::as_rsy`, which writes
    /// the raw parity byte `{0, 1}` instead of the legacy `{27, 28}`
    /// form) is forbidden in the contracts and signing surfaces and
    /// guarded by the `ecdsa-v-normalization` source fence
    /// (`cargo check-source-fences`).
    #[must_use]
    pub const fn as_alloy(&self) -> &AlloySignature {
        &self.inner
    }

    /// Returns the low-s canonical form per BIP-62, or `self` if the
    /// signature is already low-s.
    ///
    /// This is opt-in defense in depth against `(r, s)` malleability.
    /// The orderbook accepts both low-s and high-s recoverable signatures
    /// today, so this canonicalization is not applied at parse time;
    /// callers opt in when their downstream invariants require a
    /// uniquely-shaped signature.
    #[must_use]
    pub fn canonicalized_low_s(self) -> Self {
        Self {
            inner: self.inner.normalized_s(),
        }
    }

    /// Returns the ERC-2098 compact 64-byte form.
    ///
    /// The `s` component is normalized to low-s first so the parity bit
    /// fits in the high bit of the packed `s` word.
    #[must_use]
    pub fn to_erc2098(&self) -> [u8; 64] {
        self.inner.as_erc2098()
    }

    /// Constructs from an ERC-2098 compact 64-byte payload.
    ///
    /// # Errors
    ///
    /// Returns [`ContractsError::InvalidSignatureLength`] for non-64-byte
    /// payloads.
    pub fn parse_erc2098(bytes: &[u8]) -> Result<Self, ContractsError> {
        if bytes.len() != 64 {
            return Err(ContractsError::InvalidSignatureLength {
                actual: bytes.len(),
            });
        }
        Ok(Self {
            inner: AlloySignature::from_erc2098(bytes),
        })
    }

    /// Recovers the signer address against a 32-byte digest under the
    /// given ECDSA scheme.
    ///
    /// [`SigningScheme::Eip712`] recovers against the supplied digest
    /// directly. [`SigningScheme::EthSign`] applies the canonical
    /// EIP-191 prehash `keccak256("\x19Ethereum Signed Message:\n32" ||
    /// digest_bytes)` internally before recovery.
    ///
    /// # Errors
    ///
    /// Returns [`ContractsError::SignatureSchemeNotEcdsa`] for non-ECDSA
    /// scheme variants, and [`ContractsError::SignatureRecovery`] when
    /// secp256k1 recovery fails.
    pub fn recover(
        &self,
        digest: &Hash32,
        scheme: SigningScheme,
    ) -> Result<Address, ContractsError> {
        let prehash_bytes = match scheme {
            SigningScheme::Eip712 => hash32_bytes(digest),
            SigningScheme::EthSign => eth_sign_digest_prehash(digest),
            _ => return Err(ContractsError::SignatureSchemeNotEcdsa),
        };
        let recovered = self
            .inner
            .recover_address_from_prehash(&B256::from(prehash_bytes))
            .map_err(|error| signature_recovery_error(&error))?;
        Ok(Address::from_bytes(recovered.into_array()))
    }
}

pub(crate) const fn hash32_bytes(digest: &Hash32) -> [u8; 32] {
    digest.into_alloy().0
}

fn eth_sign_digest_prehash(digest: &Hash32) -> [u8; 32] {
    alloy_primitives::eip191_hash_message(hash32_bytes(digest)).0
}

fn signature_recovery_error(error: &alloy_primitives::SignatureError) -> ContractsError {
    ContractsError::SignatureRecovery {
        message: error.to_string().into(),
    }
}

/// Encodes an EIP-1271 verifier payload as the `CoW` compact wire format.
///
/// # Errors
///
/// Returns [`ContractsError`] if the verifier or signature is not valid hex.
pub fn encode_eip1271_signature_data(
    data: &Eip1271SignatureData,
) -> Result<String, ContractsError> {
    // The payload is already typed bytes, so there is no hex to re-parse. Keep
    // the decoded-length budget as a cheap guard so a caller-built oversized
    // payload is refused before the verifier-prefixed buffer is materialized.
    let signature = data.signature.as_slice();
    if signature.len() > MAX_SIGNATURE_HEX_BYTES {
        return Err(ContractsError::FieldTooLarge {
            field: "signature",
            max_bytes: MAX_SIGNATURE_HEX_BYTES,
        });
    }
    let mut payload = Vec::with_capacity(20 + signature.len());
    payload.extend_from_slice(&data.verifier.into_alloy().0.0);
    payload.extend_from_slice(signature);
    Ok(alloy_primitives::hex::encode_prefixed(payload))
}

/// Decodes a compact EIP-1271 verifier payload.
///
/// # Errors
///
/// Returns [`ContractsError::InvalidEip1271SignatureData`] when the payload is
/// shorter than the verifier address, or another [`ContractsError`] when hex or
/// address validation fails.
///
/// # Panics
///
/// Cannot panic in practice. The function returns early with
/// [`ContractsError::InvalidEip1271SignatureData`] when the decoded
/// byte length is below 20; after that guard, the 20-byte
/// slice-to-array conversion for the verifier address is infallible
/// by construction. The `expect` call inside the body documents the
/// unreachability proof so a future contributor cannot accidentally
/// weaken the guard without removing the proof first.
pub fn decode_eip1271_signature_data(
    signature: &str,
) -> Result<Eip1271SignatureData, ContractsError> {
    let bytes = decode_hex_field_bounded("signature", signature, MAX_SIGNATURE_HEX_BYTES)?;
    if bytes.len() < 20 {
        return Err(ContractsError::InvalidEip1271SignatureData);
    }
    // SAFETY: the `bytes.len() < 20` guard above guarantees `bytes.len() >= 20`
    // here, so the `[..20]` slice is always 20 bytes and `try_into` cannot fail.
    let verifier = Address::from_bytes(
        bytes[..20]
            .try_into()
            .expect("slice length 20 is guaranteed by the bytes.len() < 20 check above"),
    );
    let signature = HexData::from_bytes(bytes[20..].to_vec());
    Ok(Eip1271SignatureData::new(verifier, signature))
}

/// Encodes a signing scheme into the compact trade-flag representation.
#[inline]
#[must_use]
pub const fn encode_signing_scheme(scheme: SigningScheme) -> u8 {
    scheme.as_u8()
}

/// Decodes a signing scheme from the compact trade-flag representation.
///
/// # Errors
///
/// Returns [`ContractsError::UnsupportedSigningScheme`] for unknown values.
#[inline]
pub fn decode_signing_scheme(flags: u8) -> Result<SigningScheme, ContractsError> {
    SigningScheme::try_from(flags)
}

/// Verifies an EIP-1271 signature using a synchronous provider.
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
/// Returns [`ContractsError`] if the verifier has no code, the provider call
/// fails, or the verifier response is malformed or does not match the expected
/// magic value.
pub async fn verify_eip1271_signature<P>(
    provider: &P,
    request: &Eip1271VerificationRequest,
) -> Result<(), ContractsError>
where
    P: Provider,
    P::Error: fmt::Display,
{
    // The uncached path is the cached path with the always-miss
    // [`NoopEip1271Cache`](crate::verify::NoopEip1271Cache), so the
    // `isValidSignature` dispatch lives in exactly one place.
    crate::verify::verify_eip1271_signature_cached(
        provider,
        request,
        &crate::verify::NoopEip1271Cache,
    )
    .await
}

pub(crate) async fn ensure_contract_code<P>(
    provider: &P,
    verifier: &Address,
) -> Result<(), ContractsError>
where
    P: Provider,
    P::Error: fmt::Display,
{
    let code =
        provider
            .get_code(verifier)
            .await
            .map_err(|error| ContractsError::Eip1271Provider {
                operation: "get_code",
                message: error.to_string().into(),
            })?;

    if has_contract_code(code.as_ref()) {
        Ok(())
    } else {
        Err(ContractsError::UnsupportedEip1271Verifier {
            verifier: *verifier,
        })
    }
}

fn has_contract_code(code: Option<&HexData>) -> bool {
    matches!(code, Some(code) if !code.is_empty())
}

pub(crate) fn ensure_magic_value(raw: &str) -> Result<(), ContractsError> {
    let actual = decode_magic_value_response(raw)?;
    if actual == IERC1271::isValidSignatureCall::SELECTOR {
        Ok(())
    } else {
        Err(ContractsError::Eip1271MagicValueMismatch {
            expected: IERC1271::isValidSignatureCall::SELECTOR,
            actual,
        })
    }
}

pub(crate) fn decode_magic_value_response(raw: &str) -> Result<[u8; 4], ContractsError> {
    let candidate = match serde_json::from_str::<serde_json::Value>(raw) {
        Ok(serde_json::Value::String(value)) => value,
        Ok(other) => {
            return Err(ContractsError::MalformedEip1271Response {
                response: other.to_string().into(),
            });
        }
        Err(_) => raw.to_owned(),
    };

    let bytes: [u8; 4] = decode_hex_field_exact::<4>("magicValue", &candidate).map_err(|_| {
        ContractsError::MalformedEip1271Response {
            response: raw.to_owned().into(),
        }
    })?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::keccak256;
    use alloy_sol_types::SolCall;

    /// Test-only runtime keccak fallback used as an independent parity
    /// oracle. Production code must reach the EIP-1271 success magic
    /// value through `IERC1271::isValidSignatureCall::SELECTOR` (the
    /// typed `[u8; 4]` constant emitted by the workspace `alloy::sol!`
    /// binding) per ADR 0012; this helper exists solely to assert that
    /// the canonical Solidity signature keccak-hashes to the same four
    /// bytes.
    fn function_magic_value(signature: &str) -> String {
        let hash = keccak256(signature.as_bytes());
        alloy_primitives::hex::encode_prefixed(&hash[..4])
    }

    #[test]
    fn keccak_of_canonical_signature_matches_the_typed_eip1271_selector() {
        let runtime = function_magic_value("isValidSignature(bytes32,bytes)");
        let typed = format!(
            "0x{}",
            alloy_primitives::hex::encode(IERC1271::isValidSignatureCall::SELECTOR)
        );
        assert_eq!(runtime, typed);
        assert_eq!(runtime, "0x1626ba7e");
    }
}
