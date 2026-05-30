//! DTOs for the pure on-chain event-log decoding exports.
//!
//! [`EventLogInput`] is the JavaScript-facing log shape (`topics` + `data` as
//! hex strings) that the decode exports reconstruct into borrowed
//! [`alloy_primitives::LogData`]; [`SettlementEventDto`] and [`EthFlowEventDto`]
//! mirror the typed `cow_sdk_contracts::SettlementEvent` /
//! `cow_sdk_contracts::EthFlowEvent` results as tagged unions. Reconstruction
//! is fail-closed: malformed hex returns a typed [`WasmError`] and the
//! underlying decoders never panic.

use alloy_primitives::{B256, Bytes, LogData};
use cow_sdk_contracts::{EthFlowEvent, OnchainSigningScheme, SettlementEvent};
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use super::OrderInput;
use crate::exports::errors::WasmError;

/// Raw EVM event log accepted by the on-chain event decoders.
///
/// `topics` carries the indexed log topics as `0x`-prefixed 32-byte hex
/// strings with topic-0 (the event signature hash) first; `data` is the
/// ABI-encoded non-indexed payload as a `0x`-prefixed hex string (`"0x"` for an
/// empty payload).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct EventLogInput {
    /// Indexed log topics as 0x-prefixed 32-byte hex strings (topic-0 first).
    pub topics: Vec<String>,
    /// ABI-encoded non-indexed log data as a 0x-prefixed hex string.
    pub data: String,
}

impl EventLogInput {
    /// Reconstructs borrowed [`LogData`] from the hex topic and data fields.
    ///
    /// Topic-count and indexed arity are intentionally not validated here; the
    /// downstream fail-closed decoder is the single authority on the expected
    /// topic set and rejects a mismatch with a typed error.
    ///
    /// # Errors
    ///
    /// Returns [`WasmError::InvalidInput`] when a topic is not a 0x-prefixed
    /// 32-byte hex string or when `data` is not a 0x-prefixed hex string.
    pub(crate) fn to_log_data(&self) -> Result<LogData, WasmError> {
        let topics = self
            .topics
            .iter()
            .enumerate()
            .map(|(index, topic)| parse_topic(index, topic))
            .collect::<Result<Vec<B256>, WasmError>>()?;
        let data = parse_data(&self.data)?;
        Ok(LogData::new_unchecked(topics, data))
    }
}

/// Parses one indexed topic into a 32-byte [`B256`].
fn parse_topic(index: usize, value: &str) -> Result<B256, WasmError> {
    let field = format!("topics[{index}]");
    let stripped = value
        .strip_prefix("0x")
        .ok_or_else(|| WasmError::invalid(field.clone(), "expected a 0x-prefixed hex string"))?;
    let bytes = alloy_primitives::hex::decode(stripped)
        .map_err(|_| WasmError::invalid(field.clone(), "expected hex-encoded bytes"))?;
    let array: [u8; 32] = bytes.as_slice().try_into().map_err(|_| {
        WasmError::invalid(
            field,
            format!("expected a 32-byte topic, got {} bytes", bytes.len()),
        )
    })?;
    Ok(B256::from(array))
}

/// Parses the non-indexed data payload into borrowed [`Bytes`].
fn parse_data(value: &str) -> Result<Bytes, WasmError> {
    let stripped = value
        .strip_prefix("0x")
        .ok_or_else(|| WasmError::invalid("data", "expected a 0x-prefixed hex string"))?;
    let bytes = alloy_primitives::hex::decode(stripped)
        .map_err(|_| WasmError::invalid("data", "expected hex-encoded bytes"))?;
    Ok(Bytes::from(bytes))
}

/// A decoded `GPv2Settlement` (or inherited `GPv2Signing`) event.
///
/// Mirrors `cow_sdk_contracts::SettlementEvent`. Addresses and the order UID
/// are lowercase `0x`-prefixed hex; amounts are base-10 atom strings; the
/// interaction `selector` is a `0x`-prefixed 4-byte hex string. The `kind`
/// discriminator distinguishes the variants.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[non_exhaustive]
pub enum SettlementEventDto {
    /// A user order was executed in a settlement.
    Trade {
        /// Order owner.
        owner: String,
        /// Sell token traded.
        sell_token: String,
        /// Buy token traded.
        buy_token: String,
        /// Executed sell amount (base-10 atoms).
        sell_amount: String,
        /// Executed buy amount (base-10 atoms).
        buy_amount: String,
        /// Executed fee amount (base-10 atoms).
        fee_amount: String,
        /// 56-byte order UID of the filled order.
        order_uid: String,
    },
    /// A solver interaction was executed during a settlement.
    Interaction {
        /// Interaction target contract.
        target: String,
        /// Native value forwarded with the interaction (base-10 atoms).
        value: String,
        /// First four bytes of the interaction calldata (the function selector).
        selector: String,
    },
    /// A settlement batch completed.
    Settlement {
        /// Authorized solver that submitted the batch.
        solver: String,
    },
    /// An off-chain signed order was invalidated on-chain by its owner.
    OrderInvalidated {
        /// Owner that invalidated the order.
        owner: String,
        /// 56-byte order UID that was invalidated.
        order_uid: String,
    },
    /// An order pre-signature was set or revoked.
    PreSignature {
        /// Owner whose pre-signature changed.
        owner: String,
        /// 56-byte order UID affected.
        order_uid: String,
        /// `true` when the order is now pre-signed, `false` when revoked.
        signed: bool,
    },
}

impl SettlementEventDto {
    /// Maps a decoded `cow_sdk_contracts::SettlementEvent` into the DTO.
    ///
    /// # Errors
    ///
    /// Returns [`WasmError::Internal`] only if a future settlement-event variant
    /// is decoded that this wasm build does not yet model.
    pub(crate) fn from_event(event: SettlementEvent) -> Result<Self, WasmError> {
        Ok(match event {
            SettlementEvent::Trade {
                owner,
                sell_token,
                buy_token,
                sell_amount,
                buy_amount,
                fee_amount,
                order_uid,
            } => Self::Trade {
                owner: owner.to_hex_string(),
                sell_token: sell_token.to_hex_string(),
                buy_token: buy_token.to_hex_string(),
                sell_amount: sell_amount.to_string(),
                buy_amount: buy_amount.to_string(),
                fee_amount: fee_amount.to_string(),
                order_uid: order_uid.to_hex_string(),
            },
            SettlementEvent::Interaction {
                target,
                value,
                selector,
            } => Self::Interaction {
                target: target.to_hex_string(),
                value: value.to_string(),
                selector: alloy_primitives::hex::encode_prefixed(selector),
            },
            SettlementEvent::Settlement { solver } => Self::Settlement {
                solver: solver.to_hex_string(),
            },
            SettlementEvent::OrderInvalidated { owner, order_uid } => Self::OrderInvalidated {
                owner: owner.to_hex_string(),
                order_uid: order_uid.to_hex_string(),
            },
            SettlementEvent::PreSignature {
                owner,
                order_uid,
                signed,
            } => Self::PreSignature {
                owner: owner.to_hex_string(),
                order_uid: order_uid.to_hex_string(),
                signed,
            },
            _ => {
                return Err(WasmError::internal(
                    "decoded settlement event variant is not representable by this wasm build",
                ));
            }
        })
    }
}

/// A decoded eth-flow on-chain order lifecycle event.
///
/// Mirrors `cow_sdk_contracts::EthFlowEvent`. The placement `order` reuses the
/// canonical [`OrderInput`] shape (its `validTo` is the on-chain clamped value;
/// the trader's real expiry travels in the opaque `data` trailer). `signature`
/// and `data` are `0x`-prefixed hex strings carrying the raw on-chain signature
/// payload and the opaque trailing data field; addresses and the order UID are
/// lowercase `0x`-prefixed hex. The `kind` discriminator distinguishes the
/// variants.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[non_exhaustive]
pub enum EthFlowEventDto {
    /// An eth-flow order was broadcast on-chain.
    OrderPlacement {
        /// Account that triggered the on-chain order creation (not necessarily
        /// the order owner).
        sender: String,
        /// The reconstructed on-chain order.
        order: OrderInput,
        /// On-chain signing scheme: `"eip1271"` or `"presign"`.
        signing_scheme: String,
        /// Raw on-chain signature payload as 0x-prefixed hex.
        signature: String,
        /// Opaque trailing data field as 0x-prefixed hex.
        data: String,
    },
    /// A still-tradeable eth-flow order was invalidated on-chain.
    OrderInvalidation {
        /// 56-byte UID of the order being invalidated.
        order_uid: String,
    },
    /// Unspent native value was refunded for an expired eth-flow order.
    OrderRefund {
        /// 56-byte UID of the refunded order.
        order_uid: String,
        /// Account that triggered the refund.
        refunder: String,
    },
}

impl EthFlowEventDto {
    /// Maps a decoded `cow_sdk_contracts::EthFlowEvent` into the DTO.
    ///
    /// # Errors
    ///
    /// Returns [`WasmError::Internal`] only if a future eth-flow event variant
    /// or on-chain signing scheme is decoded that this wasm build does not yet
    /// model.
    pub(crate) fn from_event(event: EthFlowEvent) -> Result<Self, WasmError> {
        Ok(match event {
            EthFlowEvent::OrderPlacement(placement) => {
                let signing_scheme = match placement.signing_scheme {
                    OnchainSigningScheme::Eip1271 => "eip1271",
                    OnchainSigningScheme::PreSign => "presign",
                    _ => {
                        return Err(WasmError::internal(
                            "decoded on-chain signing scheme is not representable by this wasm build",
                        ));
                    }
                };
                Self::OrderPlacement {
                    sender: placement.sender.to_hex_string(),
                    order: OrderInput::from(&placement.order),
                    signing_scheme: signing_scheme.to_owned(),
                    signature: alloy_primitives::hex::encode_prefixed(
                        placement.signature_data.as_ref(),
                    ),
                    data: alloy_primitives::hex::encode_prefixed(placement.data.as_ref()),
                }
            }
            EthFlowEvent::OrderInvalidation(invalidation) => Self::OrderInvalidation {
                order_uid: invalidation.order_uid.to_hex_string(),
            },
            EthFlowEvent::OrderRefund(refund) => Self::OrderRefund {
                order_uid: refund.order_uid.to_hex_string(),
                refunder: refund.refunder.to_hex_string(),
            },
            _ => {
                return Err(WasmError::internal(
                    "decoded eth-flow event variant is not representable by this wasm build",
                ));
            }
        })
    }
}
