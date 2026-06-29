use cow_sdk_core::{Address, OrderData, OrderUid, TypedDataPayload};

use super::parse_chain;

/// The EIP-712 typed-data JSON (`domain`, `types`, `primaryType`, `message`)
/// for an order, ready for `eth_signTypedData_v4`.
pub fn order_typed_data(chain_id: u64, order_json: &str) -> Result<String, String> {
    let chain = parse_chain(chain_id)?;
    let order: OrderData = serde_json::from_str(order_json).map_err(|error| error.to_string())?;
    let payload = cow_sdk_signing::domain::order_typed_data_payload(chain, &order, None)
        .map_err(|error| error.to_string())?;
    typed_data_json(&payload)
}

/// The EIP-712 typed-data JSON for cancelling the given order UIDs.
pub fn cancellations_typed_data(chain_id: u64, uids: &[String]) -> Result<String, String> {
    let chain = parse_chain(chain_id)?;
    let order_uids = uids
        .iter()
        .map(|uid| OrderUid::new(uid).map_err(|error| error.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    let payload = cow_sdk_signing::cancellation::order_cancellations_typed_data_payload(
        &order_uids,
        chain,
        None,
    )
    .map_err(|error| error.to_string())?;
    typed_data_json(&payload)
}

/// The order UID (`0x` + 112 hex) and its EIP-712 digest (`0x` + 64 hex).
pub fn generate_order_id(
    chain_id: u64,
    owner: &str,
    order_json: &str,
) -> Result<(String, String), String> {
    let chain = parse_chain(chain_id)?;
    let owner = Address::new(owner).map_err(|error| error.to_string())?;
    let order: OrderData = serde_json::from_str(order_json).map_err(|error| error.to_string())?;
    let generated = cow_sdk_signing::generate_order_id(chain, &order, &owner, None)
        .map_err(|error| error.to_string())?;
    Ok((
        generated.order_id.to_hex_string(),
        format!(
            "0x{}",
            alloy_primitives::hex::encode(generated.order_digest.as_slice())
        ),
    ))
}

/// Wraps a host-produced 65-byte ECDSA signature into the EIP-1271 wire
/// payload (verifier-prefixed `abi.encode(order, signature)`).
pub fn eip1271_signature_payload(
    order_json: &str,
    ecdsa_signature: &str,
) -> Result<String, String> {
    let order: OrderData = serde_json::from_str(order_json).map_err(|error| error.to_string())?;
    cow_sdk_signing::eip1271_signature_payload(&order, ecdsa_signature)
        .map_err(|error| error.to_string())
}

/// Serializes a typed-data payload to the canonical EIP-712 JSON object.
fn typed_data_json(payload: &TypedDataPayload) -> Result<String, String> {
    let message: serde_json::Value = serde_json::from_str(payload.message_json())
        .map_err(|error| format!("typed-data message: {error}"))?;
    Ok(serde_json::json!({
        "domain": payload.domain,
        "types": payload.types,
        "primaryType": payload.primary_type,
        "message": message,
    })
    .to_string())
}
