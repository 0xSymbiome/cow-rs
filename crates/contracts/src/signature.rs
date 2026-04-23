use std::fmt;

use serde::{Deserialize, Serialize};

use cow_sdk_core::{Address, AsyncProvider, Hash32, HexData, Provider};

use crate::{
    ContractsError,
    primitives::{function_selector, normalize_hex_payload, parse_hex, parse_hex_exact},
};

/// EIP-1271 success magic value as the canonical `0x`-prefixed hex string
/// form documented by the protocol.
#[doc(alias = "magic-value")]
pub const EIP1271_MAGICVALUE: &str = "0x1626ba7e";

/// EIP-1271 success magic value as the 4-byte function selector
/// (`isValidSignature(bytes32,bytes)`) the protocol uses on the wire.
pub(crate) const EIP1271_MAGICVALUE_BYTES: [u8; 4] = [0x16, 0x26, 0xba, 0x7e];
pub(crate) const EIP1271_IS_VALID_SIGNATURE_ABI_JSON: &str = r#"[{"type":"function","name":"isValidSignature","inputs":[{"name":"hash","type":"bytes32"},{"name":"signature","type":"bytes"}],"outputs":[{"name":"","type":"bytes4"}],"stateMutability":"view"}]"#;

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
    /// Encoded signature payload as hex.
    pub signature: String,
}

/// Input contract for EIP-1271 verification helpers.
#[non_exhaustive]
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
    pub const fn new(verifier: Address, signature: String) -> Self {
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
}

/// Encodes an EIP-1271 verifier payload as the `CoW` compact wire format.
///
/// # Errors
///
/// Returns [`ContractsError`] if the verifier or signature is not valid hex.
pub fn encode_eip1271_signature_data(
    data: &Eip1271SignatureData,
) -> Result<String, ContractsError> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&parse_hex_exact(data.verifier.as_str(), "verifier", 20)?);
    payload.extend_from_slice(&parse_hex(&data.signature, "signature")?);
    Ok(format!("0x{}", hex::encode(payload)))
}

/// Decodes a compact EIP-1271 verifier payload.
///
/// # Errors
///
/// Returns [`ContractsError::InvalidEip1271SignatureData`] when the payload is
/// shorter than the verifier address, or another [`ContractsError`] when hex or
/// address validation fails.
pub fn decode_eip1271_signature_data(
    signature: &str,
) -> Result<Eip1271SignatureData, ContractsError> {
    let bytes = parse_hex(signature, "signature")?;
    if bytes.len() < 20 {
        return Err(ContractsError::InvalidEip1271SignatureData);
    }
    let verifier = Address::new(format!("0x{}", hex::encode(&bytes[..20])))?;
    let signature = format!("0x{}", hex::encode(&bytes[20..]));
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

/// Normalizes an ECDSA signature into canonical hex form with a legacy-range
/// recovery byte (`v ∈ {27, 28}`).
///
/// The canonical on-chain form uses `v = 27` or `v = 28`. Modern EIP-2
/// signers emit `v = 0` or `v = 1`; this helper maps the modern form onto the
/// legacy form so on-chain `ecrecover` recovers a valid signer.
///
/// Accepts `v ∈ {0, 1, 27, 28}` and rejects every other byte.
///
/// # Errors
///
/// Returns [`ContractsError`] if the signature is not valid hex, is not
/// exactly 65 bytes, or carries an unsupported recovery byte.
pub fn normalized_ecdsa_signature(data: &str) -> Result<String, ContractsError> {
    let hex_normalized = normalize_hex_payload(data, "signature")?;
    let mut bytes = parse_hex(&hex_normalized, "signature")?;
    if bytes.len() != 65 {
        return Err(ContractsError::InvalidSignatureLength {
            actual: bytes.len(),
        });
    }
    bytes[64] = match bytes[64] {
        0 | 27 => 27,
        1 | 28 => 28,
        other => {
            return Err(ContractsError::InvalidSignatureRecoveryByte { value: other });
        }
    };
    Ok(format!("0x{}", hex::encode(bytes)))
}

/// Returns the 4-byte function selector for a Solidity signature.
#[must_use]
pub fn function_magic_value(signature: &str) -> String {
    let selector = function_selector(signature);
    format!("0x{}", hex::encode(selector))
}

/// Verifies an EIP-1271 signature using a synchronous provider.
///
/// # Errors
///
/// Returns [`ContractsError`] if the verifier has no code, the provider call
/// fails, or the verifier response is malformed or does not match the expected
/// magic value.
pub fn verify_eip1271_signature<P>(
    provider: &P,
    request: &Eip1271VerificationRequest,
) -> Result<(), ContractsError>
where
    P: Provider,
    P::Error: fmt::Display,
{
    ensure_contract_code(provider, &request.verifier)?;
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
        .map_err(|error| ContractsError::Eip1271Provider {
            operation: "read_contract",
            message: error.to_string(),
        })?;

    ensure_magic_value(&raw)
}

fn ensure_contract_code<P>(provider: &P, verifier: &Address) -> Result<(), ContractsError>
where
    P: Provider,
    P::Error: fmt::Display,
{
    let code = provider
        .get_code(verifier)
        .map_err(|error| ContractsError::Eip1271Provider {
            operation: "get_code",
            message: error.to_string(),
        })?;

    if has_contract_code(code.as_ref()) {
        Ok(())
    } else {
        Err(ContractsError::UnsupportedEip1271Verifier {
            verifier: verifier.clone(),
        })
    }
}

pub(crate) async fn ensure_contract_code_async<P>(
    provider: &P,
    verifier: &Address,
) -> Result<(), ContractsError>
where
    P: AsyncProvider,
    P::Error: fmt::Display,
{
    let code =
        provider
            .get_code(verifier)
            .await
            .map_err(|error| ContractsError::Eip1271Provider {
                operation: "get_code",
                message: error.to_string(),
            })?;

    if has_contract_code(code.as_ref()) {
        Ok(())
    } else {
        Err(ContractsError::UnsupportedEip1271Verifier {
            verifier: verifier.clone(),
        })
    }
}

fn has_contract_code(code: Option<&HexData>) -> bool {
    matches!(code, Some(code) if code.as_str() != "0x")
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

pub(crate) fn decode_magic_value_response(raw: &str) -> Result<[u8; 4], ContractsError> {
    let candidate = match serde_json::from_str::<serde_json::Value>(raw) {
        Ok(serde_json::Value::String(value)) => value,
        Ok(other) => {
            return Err(ContractsError::MalformedEip1271Response {
                response: other.to_string(),
            });
        }
        Err(_) => raw.to_owned(),
    };

    let bytes = parse_hex_exact(&candidate, "magicValue", 4).map_err(|_| {
        ContractsError::MalformedEip1271Response {
            response: raw.to_owned(),
        }
    })?;
    let mut out = [0u8; 4];
    out.copy_from_slice(&bytes);
    Ok(out)
}
