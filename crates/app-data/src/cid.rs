//! CID conversion helpers for app-data documents.
//!
//! `cow-sdk-app-data` supports exactly one CID shape: `CIDv1` with the raw
//! multicodec (`0x55`) over a keccak-256 multihash (`0x1b`). This is the
//! shape the cow-protocol services backend emits for every app-data
//! document, so the conversion helpers below accept no other codec or
//! hash combination. Historical `CIDv0` values (dag-pb + sha2-256) are
//! rejected at the decoder boundary with a typed
//! [`AppDataError::InvalidCid`]; consumers that need to parse such
//! values use a general-purpose `cid` crate directly.

use cid::{Cid, Version};
use multibase::Base;
use multihash::Multihash;

use crate::AppDataError;

const LATEST_CID_CODEC: u64 = 0x55;
const KECCAK_256_CODE: u64 = 0x1b;
const APP_DATA_HEX_LENGTH: usize = 32;

/// Converts an app-data hex digest into the supported CID representation.
///
/// # Errors
///
/// Returns [`AppDataError`] if the digest is not valid 32-byte hex or if
/// CID conversion fails.
pub fn app_data_hex_to_cid(app_data_hex: &str) -> Result<String, AppDataError> {
    let digest = parse_app_data_hex(app_data_hex)?;
    let cid = latest_cid_from_digest(&digest)?;
    cid.to_string_of_base(Base::Base16Lower)
        .map_err(|err| AppDataError::Calculation {
            source: Box::new(err),
        })
}

/// Converts a supported CID back into the app-data hex digest.
///
/// # Errors
///
/// Returns [`AppDataError::InvalidCid`] if the CID is malformed or uses an
/// unsupported version, codec, or hash function.
pub fn cid_to_app_data_hex(cid: &str) -> Result<String, AppDataError> {
    let cid = Cid::try_from(cid).map_err(|_| AppDataError::InvalidCid)?;
    ensure_supported_cid(&cid)?;
    let digest = cid.hash().digest();
    Ok(alloy_primitives::hex::encode_prefixed(digest))
}

fn parse_app_data_hex(value: &str) -> Result<Vec<u8>, AppDataError> {
    let hex = value
        .strip_prefix("0x")
        .ok_or(AppDataError::InvalidAppDataHex)?;
    let bytes = alloy_primitives::hex::decode(hex).map_err(|_| AppDataError::InvalidAppDataHex)?;
    if bytes.len() != APP_DATA_HEX_LENGTH {
        return Err(AppDataError::InvalidAppDataHex);
    }
    Ok(bytes)
}

fn latest_cid_from_digest(digest: &[u8]) -> Result<Cid, AppDataError> {
    let hash = Multihash::<64>::wrap(KECCAK_256_CODE, digest).map_err(|err| {
        AppDataError::Calculation {
            source: Box::new(err),
        }
    })?;
    Ok(Cid::new_v1(LATEST_CID_CODEC, hash))
}

fn ensure_supported_cid(cid: &Cid) -> Result<(), AppDataError> {
    let digest = cid.hash().digest();
    if digest.len() != APP_DATA_HEX_LENGTH {
        return Err(AppDataError::InvalidCid);
    }

    match cid.version() {
        Version::V1 if cid.codec() == LATEST_CID_CODEC && cid.hash().code() == KECCAK_256_CODE => {
            Ok(())
        }
        _ => Err(AppDataError::InvalidCid),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_supported_cid_accepts_only_the_documented_codec_and_hash_pairs() {
        const SHA2_256_CODE: u64 = 0x12;

        let latest = Cid::new_v1(
            LATEST_CID_CODEC,
            Multihash::<64>::wrap(KECCAK_256_CODE, &[0x11; APP_DATA_HEX_LENGTH]).unwrap(),
        );
        let wrong_latest_codec = Cid::new_v1(
            0x70,
            Multihash::<64>::wrap(KECCAK_256_CODE, &[0x22; APP_DATA_HEX_LENGTH]).unwrap(),
        );
        let wrong_latest_hash = Cid::new_v1(
            LATEST_CID_CODEC,
            Multihash::<64>::wrap(SHA2_256_CODE, &[0x33; APP_DATA_HEX_LENGTH]).unwrap(),
        );

        assert!(ensure_supported_cid(&latest).is_ok());
        assert!(matches!(
            ensure_supported_cid(&wrong_latest_codec),
            Err(AppDataError::InvalidCid)
        ));
        assert!(matches!(
            ensure_supported_cid(&wrong_latest_hash),
            Err(AppDataError::InvalidCid)
        ));
    }

    #[test]
    fn ensure_supported_cid_rejects_v0_inputs() {
        const SHA2_256_CODE: u64 = 0x12;

        let v0 = Cid::new_v0(
            Multihash::<64>::wrap(SHA2_256_CODE, &[0x44; APP_DATA_HEX_LENGTH]).unwrap(),
        )
        .unwrap();

        assert!(matches!(
            ensure_supported_cid(&v0),
            Err(AppDataError::InvalidCid)
        ));
    }

    #[test]
    fn ensure_supported_cid_rejects_non_32_byte_digests() {
        let short_latest = Cid::new_v1(
            LATEST_CID_CODEC,
            Multihash::<64>::wrap(KECCAK_256_CODE, &[0x11; APP_DATA_HEX_LENGTH - 1]).unwrap(),
        );

        assert!(matches!(
            ensure_supported_cid(&short_latest),
            Err(AppDataError::InvalidCid)
        ));
    }
}
