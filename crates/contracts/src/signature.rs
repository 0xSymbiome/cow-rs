use std::fmt;

use serde::{Deserialize, Serialize};

use cow_sdk_core::{Address, AsyncProvider, Hash32, HexData, Provider};

use crate::{
    ContractsError,
    primitives::{function_selector, normalize_hex_payload, parse_hex, parse_hex_exact},
};

pub const EIP1271_MAGICVALUE: &str = "0x1626ba7e";
const EIP1271_IS_VALID_SIGNATURE_ABI_JSON: &str = r#"[{"type":"function","name":"isValidSignature","inputs":[{"name":"hash","type":"bytes32"},{"name":"signature","type":"bytes"}],"outputs":[{"name":"","type":"bytes4"}],"stateMutability":"view"}]"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum SigningScheme {
    Eip712 = 0,
    EthSign = 1,
    Eip1271 = 2,
    PreSign = 3,
}

impl SigningScheme {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn is_ecdsa(self) -> bool {
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Eip1271SignatureData {
    pub verifier: Address,
    pub signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Eip1271VerificationRequest {
    pub verifier: Address,
    pub digest: Hash32,
    pub signature: HexData,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Signature {
    Ecdsa { scheme: SigningScheme, data: String },
    Eip1271 { data: Eip1271SignatureData },
    PreSign { owner: Address },
}

impl Signature {
    pub fn scheme(&self) -> SigningScheme {
        match self {
            Signature::Ecdsa { scheme, .. } => *scheme,
            Signature::Eip1271 { .. } => SigningScheme::Eip1271,
            Signature::PreSign { .. } => SigningScheme::PreSign,
        }
    }
}

pub fn encode_eip1271_signature_data(
    data: &Eip1271SignatureData,
) -> Result<String, ContractsError> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&parse_hex_exact(data.verifier.as_str(), "verifier", 20)?);
    payload.extend_from_slice(&parse_hex(&data.signature, "signature")?);
    Ok(format!("0x{}", hex::encode(payload)))
}

pub fn decode_eip1271_signature_data(
    signature: &str,
) -> Result<Eip1271SignatureData, ContractsError> {
    let bytes = parse_hex(signature, "signature")?;
    if bytes.len() < 20 {
        return Err(ContractsError::InvalidEip1271SignatureData);
    }
    let verifier = Address::new(format!("0x{}", hex::encode(&bytes[..20])))?;
    let signature = format!("0x{}", hex::encode(&bytes[20..]));
    Ok(Eip1271SignatureData {
        verifier,
        signature,
    })
}

pub fn encode_signing_scheme(scheme: SigningScheme) -> u8 {
    scheme.as_u8()
}

pub fn decode_signing_scheme(flags: u8) -> Result<SigningScheme, ContractsError> {
    SigningScheme::try_from(flags)
}

pub fn normalized_ecdsa_signature(data: &str) -> Result<String, ContractsError> {
    normalize_hex_payload(data, "signature")
}

pub fn function_magic_value(signature: &str) -> String {
    let selector = function_selector(signature);
    format!("0x{}", hex::encode(selector))
}

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
            ))
            .map_err(|error| ContractsError::Serialization(error.to_string()))?,
        })
        .map_err(|error| ContractsError::Eip1271Provider {
            operation: "read_contract",
            message: error.to_string(),
        })?;

    ensure_magic_value(&raw)
}

pub async fn verify_eip1271_signature_async<P>(
    provider: &P,
    request: &Eip1271VerificationRequest,
) -> Result<(), ContractsError>
where
    P: AsyncProvider,
    P::Error: fmt::Display,
{
    ensure_contract_code_async(provider, &request.verifier).await?;
    let raw = provider
        .read_contract(&cow_sdk_core::ContractCall {
            address: request.verifier.clone(),
            method: "isValidSignature".to_owned(),
            abi_json: EIP1271_IS_VALID_SIGNATURE_ABI_JSON.to_owned(),
            args_json: serde_json::to_string(&(
                request.digest.as_str(),
                request.signature.as_str(),
            ))
            .map_err(|error| ContractsError::Serialization(error.to_string()))?,
        })
        .await
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

async fn ensure_contract_code_async<P>(
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
    if actual == EIP1271_MAGICVALUE {
        Ok(())
    } else {
        Err(ContractsError::Eip1271MagicValueMismatch {
            expected: EIP1271_MAGICVALUE.to_owned(),
            actual,
        })
    }
}

fn decode_magic_value_response(raw: &str) -> Result<String, ContractsError> {
    let candidate = match serde_json::from_str::<serde_json::Value>(raw) {
        Ok(serde_json::Value::String(value)) => value,
        Ok(other) => {
            return Err(ContractsError::MalformedEip1271Response {
                response: other.to_string(),
            });
        }
        Err(_) => raw.to_owned(),
    };

    parse_hex_exact(&candidate, "magicValue", 4)
        .map(|bytes| format!("0x{}", hex::encode(bytes)))
        .map_err(|_| ContractsError::MalformedEip1271Response {
            response: raw.to_owned(),
        })
}
