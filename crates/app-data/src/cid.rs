use cid::{Cid, Version};
use multibase::Base;
use multihash::Multihash;
use sha2::{Digest as Sha2Digest, Sha256};

use crate::AppDataError;

const LATEST_CID_CODEC: u64 = 0x55;
const LEGACY_CID_CODEC: u64 = 0x70;
const KECCAK_256_CODE: u64 = 0x1b;
const SHA2_256_CODE: u64 = 0x12;
const APP_DATA_HEX_LENGTH: usize = 32;

/// Supported CID derivation modes for app-data documents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CidMode {
    /// CIDv1 over the existing keccak256 app-data digest.
    Latest,
    /// Legacy CIDv0 over the JSON document bytes.
    Legacy,
}

/// Converts an app-data hex digest into the latest supported CID representation.
///
/// # Errors
///
/// Returns [`AppDataError`] if the digest is not valid 32-byte hex or if CID conversion fails.
pub fn app_data_hex_to_cid(app_data_hex: &str) -> Result<String, AppDataError> {
    let digest = parse_app_data_hex(app_data_hex)?;
    let cid = latest_cid_from_digest(&digest)?;
    cid.to_string_of_base(Base::Base16Lower)
        .map_err(|err| AppDataError::Calculation(err.to_string()))
}

/// Converts an app-data hex digest into the legacy CIDv0 representation.
///
/// # Errors
///
/// Returns [`AppDataError`] if the digest is not valid 32-byte hex or if CID conversion fails.
pub fn app_data_hex_to_cid_legacy(app_data_hex: &str) -> Result<String, AppDataError> {
    let digest = parse_app_data_hex(app_data_hex)?;
    let cid = legacy_cid_from_digest(&digest)?;
    Ok(cid.to_string())
}

/// Converts an app-data hex digest using the requested CID mode.
///
/// # Errors
///
/// Returns any error from the selected conversion mode.
pub fn app_data_hex_to_cid_with_mode(
    app_data_hex: &str,
    mode: CidMode,
) -> Result<String, AppDataError> {
    match mode {
        CidMode::Latest => app_data_hex_to_cid(app_data_hex),
        CidMode::Legacy => app_data_hex_to_cid_legacy(app_data_hex),
    }
}

/// Converts a supported CID back into the app-data hex digest.
///
/// # Errors
///
/// Returns [`AppDataError::InvalidCid`] if the CID is malformed or uses an
/// unsupported codec or hash function.
pub fn cid_to_app_data_hex(cid: &str) -> Result<String, AppDataError> {
    let cid = Cid::try_from(cid).map_err(|_| AppDataError::InvalidCid)?;
    ensure_supported_cid(&cid)?;
    let digest = cid.hash().digest();
    Ok(format!("0x{}", hex::encode(digest)))
}

pub(crate) fn app_data_bytes_to_legacy_cid(content: &[u8]) -> Result<String, AppDataError> {
    let digest = Sha256::digest(content);
    let cid = legacy_cid_from_digest(digest.as_ref())?;
    Ok(cid.to_string())
}

fn parse_app_data_hex(value: &str) -> Result<Vec<u8>, AppDataError> {
    let hex = value
        .strip_prefix("0x")
        .ok_or(AppDataError::InvalidAppDataHex)?;
    let bytes = hex::decode(hex).map_err(|_| AppDataError::InvalidAppDataHex)?;
    if bytes.len() != APP_DATA_HEX_LENGTH {
        return Err(AppDataError::InvalidAppDataHex);
    }
    Ok(bytes)
}

fn latest_cid_from_digest(digest: &[u8]) -> Result<Cid, AppDataError> {
    let hash = Multihash::<64>::wrap(KECCAK_256_CODE, digest)
        .map_err(|err| AppDataError::Calculation(err.to_string()))?;
    Ok(Cid::new_v1(LATEST_CID_CODEC, hash))
}

fn legacy_cid_from_digest(digest: &[u8]) -> Result<Cid, AppDataError> {
    let hash = Multihash::<64>::wrap(SHA2_256_CODE, digest)
        .map_err(|err| AppDataError::Calculation(err.to_string()))?;
    Cid::new_v0(hash).map_err(|_| AppDataError::InvalidCid)
}

fn ensure_supported_cid(cid: &Cid) -> Result<(), AppDataError> {
    let digest = cid.hash().digest();
    if digest.len() != APP_DATA_HEX_LENGTH {
        return Err(AppDataError::InvalidCid);
    }

    match cid.version() {
        Version::V0 if cid.codec() == LEGACY_CID_CODEC && cid.hash().code() == SHA2_256_CODE => {
            Ok(())
        }
        Version::V1 if cid.codec() == LATEST_CID_CODEC && cid.hash().code() == KECCAK_256_CODE => {
            Ok(())
        }
        _ => Err(AppDataError::InvalidCid),
    }
}
