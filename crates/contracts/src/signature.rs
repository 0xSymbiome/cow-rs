use serde::{Deserialize, Serialize};

use cow_sdk_core::Address;

use crate::{
    ContractsError,
    primitives::{function_selector, normalize_hex_payload, parse_hex, parse_hex_exact},
};

pub const EIP1271_MAGICVALUE: &str = "0x1626ba7e";

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
