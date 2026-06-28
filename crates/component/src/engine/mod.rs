//! The deterministic engine lane: order identity, the gas-free on-chain
//! transaction builders, the EIP-712 / EIP-1271 signing payloads, and on-chain
//! event-log decoding — all pure, with no host imports. Built for the component
//! target and the native golden test.

use cow_sdk_core::{Address, OrderData, SupportedChainId};

// Gas-free on-chain transaction builders for the engine world's `tx` interface.
mod tx;
// EIP-712 / EIP-1271 / order-id / cancellation signing payloads for the
// `order-signing` interface.
mod signing;
// On-chain settlement and eth-flow event-log decoding for the `events` interface.
mod events;
// ComposableCoW / TWAP conditional-order encoding for the `composable` interface.
mod composable;

// The engine golden test: pins the deterministic entry points (uid/digest, the
// tx-builder targets and selectors, the signing payloads, and fail-closed event
// decode) on the native target.
#[cfg(test)]
mod tests;

// ===== world: order-engine (deterministic, no host imports) =====
#[cfg(all(
    target_arch = "wasm32",
    feature = "world-engine",
    not(any(feature = "world-client-sync", feature = "world-client-async"))
))]
#[allow(
    unsafe_code,
    clippy::same_length_and_capacity,
    reason = "wit-bindgen's export! generates the Canonical ABI glue (#[export_name], unsafe, and raw Vec reconstruction); this crate writes none of its own"
)]
mod world;

/// Resolves a numeric chain id to a supported chain.
fn parse_chain(chain_id: u64) -> Result<SupportedChainId, String> {
    SupportedChainId::try_from(chain_id).map_err(|error| error.to_string())
}

/// Computes the order UID (`0x` + 112 hex) for a camelCase order JSON.
fn compute_uid(chain_id: u64, owner: &str, order_json: &str) -> Result<String, String> {
    let chain = parse_chain(chain_id)?;
    let owner = Address::new(owner).map_err(|error| error.to_string())?;
    let order: OrderData = serde_json::from_str(order_json).map_err(|error| error.to_string())?;
    let generated = cow_sdk_signing::generate_order_id(chain, &order, &owner, None)
        .map_err(|error| error.to_string())?;
    Ok(generated.order_id.to_hex_string())
}

/// Computes the EIP-712 order digest (`0x` + 64 hex) for a camelCase order JSON.
///
/// The digest is independent of the owner — only the UID packs it — so a zero
/// owner is used here.
fn compute_digest(chain_id: u64, order_json: &str) -> Result<String, String> {
    let chain = parse_chain(chain_id)?;
    let order: OrderData = serde_json::from_str(order_json).map_err(|error| error.to_string())?;
    let owner = Address::new("0x0000000000000000000000000000000000000000")
        .map_err(|error| error.to_string())?;
    let generated = cow_sdk_signing::generate_order_id(chain, &order, &owner, None)
        .map_err(|error| error.to_string())?;
    Ok(format!(
        "0x{}",
        alloy_primitives::hex::encode(generated.order_digest.as_slice())
    ))
}
