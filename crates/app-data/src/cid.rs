use cid::Cid;
use multibase::Base;
use multihash::Multihash;

use crate::AppDataError;

const LATEST_CID_CODEC: u64 = 0x55;
const KECCAK_256_CODE: u64 = 0x1b;
const SHA2_256_CODE: u64 = 0x12;
const APP_DATA_HEX_LENGTH: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CidMode {
    Latest,
    Legacy,
}

pub fn app_data_hex_to_cid(app_data_hex: &str) -> Result<String, AppDataError> {
    let digest = parse_app_data_hex(app_data_hex)?;
    let hash = Multihash::<64>::wrap(KECCAK_256_CODE, &digest)
        .map_err(|err| AppDataError::Calculation(err.to_string()))?;
    let cid = Cid::new_v1(LATEST_CID_CODEC, hash);
    cid.to_string_of_base(Base::Base16Lower)
        .map_err(|err| AppDataError::Calculation(err.to_string()))
}

pub fn app_data_hex_to_cid_legacy(app_data_hex: &str) -> Result<String, AppDataError> {
    let digest = parse_app_data_hex(app_data_hex)?;
    let hash = Multihash::<64>::wrap(SHA2_256_CODE, &digest)
        .map_err(|err| AppDataError::Calculation(err.to_string()))?;
    let cid = Cid::new_v0(hash).map_err(|_| AppDataError::InvalidCid)?;
    Ok(cid.to_string())
}

pub fn app_data_hex_to_cid_with_mode(
    app_data_hex: &str,
    mode: CidMode,
) -> Result<String, AppDataError> {
    match mode {
        CidMode::Latest => app_data_hex_to_cid(app_data_hex),
        CidMode::Legacy => app_data_hex_to_cid_legacy(app_data_hex),
    }
}

pub fn cid_to_app_data_hex(cid: &str) -> Result<String, AppDataError> {
    let cid = Cid::try_from(cid).map_err(|_| AppDataError::InvalidCid)?;
    let digest = cid.hash().digest();
    if digest.len() != APP_DATA_HEX_LENGTH {
        return Err(AppDataError::InvalidCid);
    }
    Ok(format!("0x{}", hex::encode(digest)))
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
