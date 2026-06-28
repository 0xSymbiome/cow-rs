use alloy_primitives::{B256, Bytes, LogData, hex};
use cow_sdk_contracts::{EthFlowEvent, SettlementEvent};

/// Decodes a `GPv2Settlement` event log.
pub fn settlement(topics: &[String], data: &str) -> Result<SettlementEvent, String> {
    cow_sdk_contracts::decode_settlement_log(&log_data(topics, data)?)
        .map_err(|error| error.to_string())
}

/// Decodes an eth-flow on-chain order lifecycle event log.
pub fn eth_flow(topics: &[String], data: &str) -> Result<EthFlowEvent, String> {
    cow_sdk_contracts::decode_eth_flow_log(&log_data(topics, data)?)
        .map_err(|error| error.to_string())
}

fn log_data(topics: &[String], data: &str) -> Result<LogData, String> {
    let topics = topics
        .iter()
        .map(|topic| parse_b256(topic))
        .collect::<Result<Vec<B256>, String>>()?;
    Ok(LogData::new_unchecked(
        topics,
        Bytes::from(decode_hex(data)?),
    ))
}

fn parse_b256(value: &str) -> Result<B256, String> {
    B256::try_from(decode_hex(value)?.as_slice())
        .map_err(|_| format!("topic must be 32 bytes: {value}"))
}

fn decode_hex(value: &str) -> Result<Vec<u8>, String> {
    hex::decode(value.strip_prefix("0x").unwrap_or(value)).map_err(|error| error.to_string())
}
